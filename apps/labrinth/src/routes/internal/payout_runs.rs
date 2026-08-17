use actix_web::{get, web};
use chrono::{Months, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use xredis::RedisPool;

use crate::{
    database::PgPool,
    queue::payout_run::{
        DayDistribution, PayoutVariances, distribution_for_day, estimate,
    },
    routes::ApiError,
    util::{
        error::Context,
        time::{YearMonth, net_60_payout_available_at},
    },
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_runs);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayoutRuns {
    pub periods: Vec<PayoutRunPeriod>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayoutRunPeriod {
    pub period: YearMonth,
    pub status: PayoutPeriodStatus,
    pub days: Vec<PayoutRunDay>,
    pub adjustments: Vec<PayoutRunAdjustment>,
}

/// Has revenue been distributed for a specific payout period month yet?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayoutPeriodStatus {
    /// We are still waiting on the NET 60 cycle to complete for this month;
    /// revenue has not been received by the platform yet.
    Open,
    /// Revenue should have been received for the platform by now; waiting for
    /// an admin to manually execute the payout run.
    InReview,
    /// Payout run is currently executing.
    Running,
    /// Payout run has been paid out to creators.
    Paid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayoutRunDay {
    pub date: NaiveDate,
    pub estimated: DayDistribution,
    pub actual: Option<DayDistribution>,
}

/// Manual admin-input adjustment to a [`PayoutRunPeriod`].
#[derive(Debug, Serialize, Deserialize)]
pub struct PayoutRunAdjustment {
    /// Total value of the adjustment.
    pub amount_usd: Decimal,
    /// Why this adjustment was applied.
    ///
    /// Only visible to admins.
    pub description: Option<String>,
}

#[get("")]
pub async fn get_runs(
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    aditude: web::Data<aditude::Client>,
) -> Result<web::Json<PayoutRuns>, ApiError> {
    let now = Utc::now();
    let latest_payout_value = sqlx::query_scalar!(
        r#"
        SELECT MAX(created)
        FROM payouts_values
        "#,
    )
    .fetch_one(&**pool)
    .await
    .wrap_internal_err("fetching latest payout value")?
    .unwrap_or(now)
    .max(now);

    let current_period = YearMonth::from_day1(now.date_naive());
    let mut period = YearMonth::from_day1(latest_payout_value.date_naive());
    let mut requested_periods = Vec::new();

    while period <= current_period {
        requested_periods.push(period);
        period = YearMonth::from_day1(
            period
                .date()
                .checked_add_months(Months::new(1))
                .wrap_internal_err("calculating next payout period")?,
        );
    }

    let estimates =
        estimate(aditude.get_ref(), redis.get_ref(), &requested_periods)
            .await
            .wrap_internal_err("fetching payout estimates")?;

    let periods = estimates
        .into_iter()
        .map(|estimate| {
            let status = if let Some(available_at) =
                net_60_payout_available_at(estimate.period)
                && now >= available_at
            {
                PayoutPeriodStatus::InReview
            } else {
                PayoutPeriodStatus::Open
            };
            let days = estimate
                .days
                .into_iter()
                .map(|day| PayoutRunDay {
                    date: day.date,
                    estimated: distribution_for_day(
                        day.date,
                        day.raw_estimated_revenue_usd,
                        day.impressions,
                        Decimal::ZERO,
                        &PayoutVariances::ZERO,
                    ),
                    actual: None,
                })
                .collect();

            PayoutRunPeriod {
                period: estimate.period,
                status,
                days,
                adjustments: Vec::new(),
            }
        })
        .collect::<Vec<_>>();

    Ok(web::Json(PayoutRuns { periods }))
}
