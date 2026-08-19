use std::collections::HashMap;

use actix_web::{get, web};
use chrono::{Months, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use xredis::RedisPool;

use crate::{
    database::{
        PgPool,
        models::{
            payout_period_item::DBPayoutPeriod,
            payout_variance_item::DBPayoutVariance,
        },
    },
    queue::payout_run::{
        DayDistribution, PayoutVariance, PayoutVariances,
        compute_actual_distribution_flow, distribution_for_day, estimate,
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

    let requested_period_dates = requested_periods
        .iter()
        .map(|period| period.date())
        .collect::<Vec<_>>();
    let stored_periods =
        DBPayoutPeriod::get_many(&requested_period_dates, &**pool)
            .await
            .wrap_internal_err("fetching stored payout periods")?
            .into_iter()
            .map(|period| (period.period, period))
            .collect::<HashMap<_, _>>();
    let stored_variances = DBPayoutVariance::get_all(&**pool)
        .await
        .wrap_internal_err("fetching payout variances")?;
    let default_variance = stored_variances
        .first()
        .wrap_internal_err("no payout variance configured")?
        .variance;
    let variances = PayoutVariances {
        default_frac: default_variance,
        fracs: stored_variances
            .into_iter()
            .map(|variance| PayoutVariance {
                starts_at: variance.applied_on,
                frac: variance.variance,
            })
            .collect(),
    };
    let estimates =
        estimate(aditude.get_ref(), redis.get_ref(), &requested_periods)
            .await
            .wrap_internal_err("fetching payout estimates")?;

    let periods = estimates
        .into_iter()
        .map(|estimate| -> Result<_, ApiError> {
            if let Some(period) = stored_periods.get(&estimate.period.date()) {
                let adjustments = serde_json::from_value::<
                    Vec<PayoutRunAdjustment>,
                >(period.adjustments.clone())
                .wrap_internal_err("deserializing payout adjustments")?;
                let total_estimated_revenue_usd = period
                    .days
                    .iter()
                    .map(|day| day.raw_estimated_aditude_revenue_usd)
                    .sum::<Decimal>();
                let actual_flow = compute_actual_distribution_flow(
                    total_estimated_revenue_usd,
                    period.raw_actual_aditude_revenue_usd,
                );
                let days = period
                    .days
                    .iter()
                    .map(|day| -> Result<_, ApiError> {
                        let impressions =
                            u128::try_from(day.aditude_impressions)
                                .wrap_internal_err(
                                    "converting stored Aditude impressions",
                                )?;
                        Ok(PayoutRunDay {
                            date: day.date,
                            estimated: distribution_for_day(
                                day.date,
                                day.raw_estimated_aditude_revenue_usd,
                                impressions,
                                &variances,
                            ),
                            actual: Some(actual_flow.distribution_for_day(
                                day.date,
                                day.raw_estimated_aditude_revenue_usd,
                                impressions,
                            )),
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let status = if period.has_succeeded_run {
                    PayoutPeriodStatus::Paid
                } else if period.has_active_run {
                    PayoutPeriodStatus::Running
                } else {
                    PayoutPeriodStatus::InReview
                };

                Ok(PayoutRunPeriod {
                    period: estimate.period,
                    status,
                    days,
                    adjustments,
                })
            } else {
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
                            &variances,
                        ),
                        actual: None,
                    })
                    .collect();

                Ok(PayoutRunPeriod {
                    period: estimate.period,
                    status,
                    days,
                    adjustments: Vec::new(),
                })
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(web::Json(PayoutRuns { periods }))
}
