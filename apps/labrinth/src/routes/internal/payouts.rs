use std::cmp::Reverse;
use std::collections::HashSet;

use actix_web::{HttpRequest, get, web};
use chrono::{Months, Utc};

use crate::auth::get_user_from_headers;
use crate::database::models::DBUserId;
use crate::database::redis::RedisPool;
use crate::database::{PgPool, ReadOnlyPgPool};
use crate::models::pats::Scopes;
use crate::models::payout_runs::{Adjustment, PayoutRun, PayoutRunStatus};
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::error::Context;
use crate::util::time::YearMonth;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get);
}

/// List creator payout runs.
#[utoipa::path(
    tag = "payouts",
    responses((status = OK, body = inline(Vec<PayoutRun>)))
)]
#[get("/payout-runs")]
pub async fn get(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<Vec<PayoutRun>>, ApiError> {
    let is_admin = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::empty(),
    )
    .await
    .is_ok_and(|(_, user)| user.role.is_admin());

    let stored_runs = sqlx::query!(
        r#"
        SELECT
            period_start,
            started_at,
            started_by,
            completed_at,
            adjustments AS "adjustments!: sqlx::types::Json<Vec<Adjustment>>"
        FROM payouts_runs
        ORDER BY period_start DESC
        "#,
    )
    .fetch_all(&***ro_pool)
    .await
    .wrap_internal_err("failed to fetch payout runs")?;

    let newest_created = sqlx::query_scalar!(
        r#"
        SELECT created
        FROM payouts_values
        ORDER BY created DESC
        LIMIT 1
        "#,
    )
    .fetch_optional(&***ro_pool)
    .await
    .wrap_internal_err("failed to fetch newest payout value")?;

    let mut stored_periods = HashSet::with_capacity(stored_runs.len());
    let mut runs = Vec::with_capacity(stored_runs.len());
    for run in stored_runs {
        let period_start = YearMonth::from_day1(run.period_start.date_naive());
        let status = if run.completed_at.is_some() {
            PayoutRunStatus::Done
        } else {
            PayoutRunStatus::Running
        };

        stored_periods.insert(period_start);
        runs.push(PayoutRun {
            period_start,
            status,
            started_at: is_admin.then_some(run.started_at),
            started_by: is_admin
                .then_some(run.started_by.map(|id| DBUserId(id).into()))
                .flatten(),
            completed_at: is_admin.then_some(run.completed_at).flatten(),
            adjustments: is_admin.then_some(run.adjustments.0),
        });
    }

    if let Some(newest_created) = newest_created {
        let now = Utc::now();
        let newest_period = YearMonth::from_day1(newest_created.date_naive());
        let mut period = YearMonth::from_day1(now.date_naive());

        while period <= newest_period {
            if !stored_periods.contains(&period) {
                runs.push(PayoutRun {
                    period_start: period,
                    status: PayoutRunStatus::Pending,
                    started_at: None,
                    started_by: None,
                    completed_at: None,
                    adjustments: None,
                });
            }

            if period == newest_period {
                break;
            }

            let next_month = period
                .date()
                .checked_add_months(Months::new(1))
                .wrap_internal_err(
                "failed to calculate next payout month",
            )?;
            period = YearMonth::from_day1(next_month);
        }
    }

    runs.sort_by_key(|run| Reverse(run.period_start));

    Ok(web::Json(runs))
}
