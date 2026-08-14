use std::cmp::Reverse;
use std::collections::HashSet;

use actix_web::{HttpRequest, get, web};
use chrono::{Months, Utc};

use crate::auth::get_user_from_headers;
use crate::database::models::DBUserId;
use crate::database::{PgPool, ReadOnlyPgPool};
use crate::models::pats::Scopes;
use crate::models::payout_runs::{
    Adjustment, PayoutRun, PayoutRunCompletion, PayoutRunReport,
    PayoutRunRevenue, PayoutRunStatus,
};
use crate::queue::payouts::get_cached_aditude_month_estimates;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::error::Context;
use crate::util::time::{YearMonth, net_60_payout_available_at};
use xredis::RedisPool;

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
            completed_result AS "completed_result?: sqlx::types::Json<PayoutRunCompletion>",
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
    let mut estimate_periods = HashSet::new();
    let mut runs = Vec::with_capacity(stored_runs.len());
    for run in stored_runs {
        let period_start = YearMonth::from_day1(run.period_start.date_naive());
        let (status, report) = if run.completed_at.is_some() {
            let amount_usd = run
                .completed_result
                .map(|completion| completion.revenue_usd)
                .wrap_internal_err(
                    "paid payout run is missing its completion result",
                )?;
            (
                PayoutRunStatus::Paid,
                PayoutRunReport {
                    revenue: PayoutRunRevenue::Actual { amount_usd },
					fees_deducted_usd: todo!(),
					variance_adjustment_usd: todo!(),
					net_estimated_revenue_usd: todo!(),
					creator_net_estimated_revenue_usd: todo!(),
					modrinth_net_estimated_revenue_usd: todo!(),
                },
            )
        } else {
            estimate_periods.insert(period_start);
            (
                PayoutRunStatus::Review,
                PayoutRunReport {
                    revenue: PayoutRunRevenue::Estimated { days: Vec::new() },
					fees_deducted_usd: todo!(),
					variance_adjustment_usd: todo!(),
					net_estimated_revenue_usd: todo!(),
					creator_net_estimated_revenue_usd: todo!(),
					modrinth_net_estimated_revenue_usd: todo!(),
                },
            )
        };

        stored_periods.insert(period_start);
        runs.push(PayoutRun {
            period_start,
            status,
            report,
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
                let status = if period == newest_period {
                    PayoutRunStatus::Open
                } else if net_60_payout_available_at(period).wrap_internal_err(
                    "failed to calculate payout review date",
                )? <= newest_created
                {
                    PayoutRunStatus::Review
                } else {
                    PayoutRunStatus::Pending
                };

                estimate_periods.insert(period);

                runs.push(PayoutRun {
                    period_start: period,
                    status,
                    report: PayoutRunReport {
                        revenue: PayoutRunRevenue::Estimated {
                            days: Vec::new(),
                        },
						fees_deducted_usd: todo!(),
						variance_adjustment_usd: todo!(),
						net_estimated_revenue_usd: todo!(),
						creator_net_estimated_revenue_usd: todo!(),
						modrinth_net_estimated_revenue_usd: todo!(),
                    },
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

    let estimate_periods = estimate_periods.into_iter().collect::<Vec<_>>();
    let estimates =
        get_cached_aditude_month_estimates(&estimate_periods, &redis).await?;
    for run in &mut runs {
        if let Some(days) = estimates.get(&run.period_start) {
            run.report.net_estimated_revenue_usd =
                days.iter().map(|day| day.amount_usd).sum();
            run.report.revenue =
                PayoutRunRevenue::Estimated { days: days.clone() };
        }
    }

    runs.sort_by_key(|run| Reverse(run.period_start));

    Ok(web::Json(runs))
}
