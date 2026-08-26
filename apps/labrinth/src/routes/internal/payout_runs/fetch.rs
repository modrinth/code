use std::collections::HashMap;

use actix_web::{HttpRequest, get, web};
use chrono::{DateTime, Months, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use xredis::RedisPool;

use super::RevenueAdjustment;
use crate::{
    auth::get_user_from_headers,
    database::{
        PgPool,
        models::{
            payout_period_item::DBPayoutPeriod,
            payout_variance_item::DBPayoutVariance,
        },
    },
    models::pats::Scopes,
    queue::{
        payout_run::{
            DayDistribution, PayoutVariance, PayoutVariances,
            compute_actual_distribution_flow, distribution_for_day, estimate,
        },
        session::AuthQueue,
    },
    routes::ApiError,
    util::{
        error::Context,
        time::{YearMonth, net_60_payout_available_at},
    },
};

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PayoutRuns {
    pub periods: Vec<PayoutRunPeriod>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PayoutRunPeriod {
    pub period: YearMonth,
    pub status: PayoutPeriodStatus,
    /// When the active payout run is scheduled to begin executing.
    pub runs_at: Option<DateTime<Utc>>,
    pub days: Vec<PayoutRunDay>,
    /// Sum of all revenue adjustments applied on top of actual revenue.
    #[serde(with = "rust_decimal::serde::float")]
    pub total_revenue_adjustment_usd: Decimal,
    /// Individual revenue adjustments, including their admin-provided
    /// descriptions.
    ///
    /// Only visible to admins.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revenue_adjustments: Option<Vec<RevenueAdjustment>>,
}

/// Has revenue been distributed for a specific payout period month yet?
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum PayoutPeriodStatus {
    /// We are still waiting on the NET 60 cycle to complete for this month;
    /// revenue has not been received by the platform yet.
    Open,
    /// Revenue should have been received for the platform by now; waiting for
    /// an admin to manually execute the payout run.
    InReview,
    /// A payout run is waiting for its cancellation window to expire.
    Scheduled,
    /// Payout run has been paid out to creators.
    Paid,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PayoutRunDay {
    pub date: NaiveDate,
    pub estimated: DayDistribution,
    pub actual: Option<DayDistribution>,
}

/// Fetch all payout runs.
#[utoipa::path(
    tag = "payout runs",
    responses((status = OK, body = PayoutRuns)),
)]
#[get("")]
pub async fn get_runs(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    aditude: web::Data<aditude::Client>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<PayoutRuns>, ApiError> {
    let is_admin = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await
    .is_ok_and(|(_, user)| user.role.is_admin());
    let now = Utc::now();
    let current_date = aditude::phoenix_date(now);
    let first_period_date = sqlx::query_scalar!(
        r#"
        SELECT LEAST(
            COALESCE(
                (SELECT MAX(created)::date FROM payouts_values),
                $1
            ),
            COALESCE(
                (SELECT MIN(period) FROM payout_periods),
                $1
            ),
            $1
        ) AS "first_period!"
        "#,
        current_date,
    )
    .fetch_one(&**pool)
    .await
    .wrap_internal_err("fetching first payout period")?;

    let current_period = YearMonth::from_day1(current_date);
    let mut period = YearMonth::from_day1(first_period_date);
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
    let live_periods = requested_periods
        .iter()
        .copied()
        .filter(|period| {
            stored_periods
                .get(&period.date())
                .is_none_or(|stored| stored.days.is_empty())
        })
        .collect::<Vec<_>>();
    let mut live_estimates = if let Some(first_requested_period) =
        requested_periods.first()
        && !live_periods.is_empty()
    {
        estimate(
            aditude.get_ref(),
            redis.get_ref(),
            &live_periods,
            first_requested_period.date(),
            current_date,
        )
        .await
        .wrap_internal_err("fetching payout estimates")?
        .into_iter()
        .map(|estimate| (estimate.period, estimate))
        .collect::<HashMap<_, _>>()
    } else {
        HashMap::new()
    };

    let periods = requested_periods
        .into_iter()
        .map(|requested_period| -> Result<_, ApiError> {
            if let Some(period) = stored_periods
                .get(&requested_period.date())
                .filter(|period| !period.days.is_empty())
            {
                let period_revenue_adjustments =
                    if let Some(payload) = &period.active_run_payload {
                        &payload.revenue_adjustments
                    } else {
                        &period.revenue_adjustments
                    };
                let total_revenue_adjustment_usd = period_revenue_adjustments
                    .iter()
                    .map(|adjustment| adjustment.amount_usd)
                    .sum();
                let revenue_adjustments =
                    is_admin.then(|| period_revenue_adjustments.clone());
                let total_estimated_revenue_usd = period
                    .days
                    .iter()
                    .map(|day| day.raw_estimated_aditude_revenue_usd)
                    .sum();
                let actual_flow = compute_actual_distribution_flow(
                    requested_period,
                    total_estimated_revenue_usd,
                    period.raw_actual_aditude_revenue_usd,
                    total_revenue_adjustment_usd,
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
                                Decimal::ZERO,
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
                } else if period.active_run_execute_at.is_some() {
                    PayoutPeriodStatus::Scheduled
                } else {
                    PayoutPeriodStatus::InReview
                };

                Ok(PayoutRunPeriod {
                    period: requested_period,
                    status,
                    runs_at: period.active_run_execute_at,
                    days,
                    total_revenue_adjustment_usd,
                    revenue_adjustments,
                })
            } else {
                let estimate = live_estimates
                    .remove(&requested_period)
                    .wrap_internal_err("missing live payout estimate")?;
                let status = if let Some(available_at) =
                    net_60_payout_available_at(requested_period)
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
                            &variances,
                        ),
                        actual: None,
                    })
                    .collect();

                Ok(PayoutRunPeriod {
                    period: requested_period,
                    status,
                    runs_at: None,
                    days,
                    total_revenue_adjustment_usd: Decimal::ZERO,
                    revenue_adjustments: is_admin.then(Vec::new),
                })
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(web::Json(PayoutRuns { periods }))
}
