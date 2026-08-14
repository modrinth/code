use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};

use actix_web::{HttpRequest, get, web};
use chrono::{Months, Utc};
use rust_decimal::Decimal;

use crate::auth::get_user_from_headers;
use crate::database::models::DBUserId;
use crate::database::{PgPool, ReadOnlyPgPool};
use crate::models::pats::Scopes;
use crate::models::payout_runs::{
	Adjustment, DayRevenue, PayoutRun, PayoutRunCompletion, PayoutRunReport,
	PayoutRunStatus,
};
use crate::queue::payouts::get_cached_aditude_month_estimates;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::error::Context;
use crate::util::time::{YearMonth, net_60_payout_available_at};
use xredis::RedisPool;

#[derive(Debug, Clone, Copy)]
enum DayRevenueEstimate {
	Raw,
	AdjustedToActual { actual_revenue_usd: Decimal },
}

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
	let mut revenue_estimates = HashMap::new();
	let mut runs = Vec::with_capacity(stored_runs.len());
	for run in stored_runs {
		let period_start = YearMonth::from_day1(run.period_start.date_naive());
		let (status, revenue_estimate) = if run.completed_at.is_some() {
			let amount_usd = run
				.completed_result
                .map(|completion| completion.revenue_usd)
                .wrap_internal_err(
                    "paid payout run is missing its completion result",
                )?;
			(
				PayoutRunStatus::Paid,
				DayRevenueEstimate::AdjustedToActual {
					actual_revenue_usd: amount_usd,
				},
			)
		} else {
			(PayoutRunStatus::Review, DayRevenueEstimate::Raw)
		};

		stored_periods.insert(period_start);
		revenue_estimates.insert(period_start, revenue_estimate);
		runs.push(PayoutRun {
			period_start,
			status,
			report: PayoutRunReport {
				days: Vec::new(),
				fees_deducted_usd: Decimal::ZERO, // TODO: calculate deducted fees
				variance_adjustment_usd: Decimal::ZERO, // TODO: calculate variance adjustment
				net_estimated_revenue_usd: Decimal::ZERO, // TODO: calculate net revenue
				creator_net_estimated_revenue_usd: Decimal::ZERO, // TODO: calculate creator share
				modrinth_net_estimated_revenue_usd: Decimal::ZERO, // TODO: calculate Modrinth share
			},
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

				revenue_estimates.insert(period, DayRevenueEstimate::Raw);

                runs.push(PayoutRun {
                    period_start: period,
                    status,
					report: PayoutRunReport {
						days: Vec::new(),
						fees_deducted_usd: Decimal::ZERO, // TODO: calculate deducted fees
						variance_adjustment_usd: Decimal::ZERO, // TODO: calculate variance adjustment
						net_estimated_revenue_usd: Decimal::ZERO, // TODO: calculate net revenue
						creator_net_estimated_revenue_usd: Decimal::ZERO, // TODO: calculate creator share
						modrinth_net_estimated_revenue_usd: Decimal::ZERO, // TODO: calculate Modrinth share
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

	let estimate_periods = revenue_estimates.keys().copied().collect::<Vec<_>>();
	let estimates =
		get_cached_aditude_month_estimates(&estimate_periods, &redis).await?;
	for run in &mut runs {
		if let Some(days) = estimates.get(&run.period_start) {
			let days = match revenue_estimates.get(&run.period_start) {
				Some(DayRevenueEstimate::Raw) | None => days.clone(),
				Some(DayRevenueEstimate::AdjustedToActual {
					actual_revenue_usd,
				}) => adjust_estimates_to_actual(days, *actual_revenue_usd)?,
			};
			run.report.net_estimated_revenue_usd =
				days.iter().map(|day| day.amount_usd).sum();
			run.report.days = days;
		}
    }

    runs.sort_by_key(|run| Reverse(run.period_start));

	Ok(web::Json(runs))
}

fn adjust_estimates_to_actual(
	days: &[DayRevenue],
	actual_revenue_usd: Decimal,
) -> Result<Vec<DayRevenue>, ApiError> {
	if days.is_empty() {
		return Ok(Vec::new());
	}

	let estimated_revenue_usd =
		days.iter().map(|day| day.amount_usd).sum::<Decimal>();
	let day_count = u64::try_from(days.len())
		.wrap_internal_err("failed to calculate payout period day count")?;
	let mut allocated_revenue_usd = Decimal::ZERO;
	let last_day = days.len() - 1;

	Ok(days
		.iter()
		.enumerate()
		.map(|(index, day)| {
			let amount_usd = if index == last_day {
				actual_revenue_usd - allocated_revenue_usd
			} else if estimated_revenue_usd.is_zero() {
				actual_revenue_usd / Decimal::from(day_count)
			} else {
				day.amount_usd * actual_revenue_usd / estimated_revenue_usd
			};
			allocated_revenue_usd += amount_usd;

			DayRevenue {
				date: day.date,
				amount_usd,
			}
		})
		.collect())
}
