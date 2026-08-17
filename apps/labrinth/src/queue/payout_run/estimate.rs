//! Logic for fetching and caching revenue estimations from our ad provider.

use std::collections::HashMap;

use chrono::{Datelike, Months};
use dashmap::DashMap;
use eyre::{Result, eyre};
use rust_decimal::Decimal;
use xredis::RedisPool;

use crate::{
    routes::ApiError,
    util::{error::Context, time::YearMonth},
};

const REDIS_KEY: &str = "aditude_month_estimate_v1";

#[derive(Debug)]
pub struct PeriodEstimate {
    pub period: YearMonth,
    pub days: Vec<DayEstimate>,
}

#[derive(Debug)]
pub struct DayEstimate {
    pub day: u32,
    pub raw_estimated_revenue_usd: Decimal,
    pub impressions: u128,
}

pub async fn estimate(
    aditude: &aditude::Client,
    redis: &RedisPool,
    periods: &[YearMonth],
) -> Result<Vec<PeriodEstimate>> {
    redis
        .get_cached_keys(REDIS_KEY, periods, |periods| async {
            fetch_estimates(aditude, redis, &periods)
                .await
                .map_err(ApiError::Internal)
        })
        .await
}

async fn fetch_estimates(
    aditude: &aditude::Client,
    redis: &RedisPool,
    periods: &[YearMonth],
) -> Result<DashMap<YearMonth, PeriodEstimate>> {
    let mut periods = periods.iter();
    let first_period = periods.next().wrap_err("no first period")?;
    let last_period = periods.last().unwrap_or(first_period);

    let range_start = first_period
        .date()
        .and_hms_opt(0, 0, 0)
        .wrap_err("calculating payout period start")?
        .and_utc();
    let range_end = last_period
        .date()
        .checked_add_months(Months::new(1))
        .wrap_err("calculating month after payout period end")?
        .and_hms_opt(0, 0, 0)
        .wrap_err("calculating payout period end")?
        .and_utc();

    let metrics = aditude
        .get_metrics_v2(aditude::v2::GetMetrics {
            metrics: &[
                aditude::v2::MetricKind::Revenue,
                aditude::v2::MetricKind::Impressions,
            ],
            range: aditude::v2::Range::Custom {
                start: range_start,
                end: range_end,
            },
            interval: aditude::v2::Interval::OneDay,
        })
        .await
        .wrap_err("fetching metrics from Aditude")?;

    let mut map = HashMap::<YearMonth, PeriodEstimate>::new();
    for response in metrics.responses {
        for row in response.rows {
            let date = row.time.date_naive();
            let period = YearMonth::from_day1(date);

            let period_estimate = map.entry(period).or_insert(PeriodEstimate {
                period,
                days: Vec::new(),
            });
            let days = &mut period_estimate.days;

            let day = date.day();
            days.push(DayEstimate {
                day,
                raw_estimated_revenue_usd: row
                    .revenue
                    .wrap_err_with(|| eyre!("no revenue data for day {day}"))?,
                impressions: row.impressions.wrap_err_with(|| {
                    eyre!("no impressions data for day {day}")
                })?,
            });
        }
    }

    // we have no clue if the Aditude row return order is stable,
    // so for safety sort the days here
    for period in map.values_mut() {
        period.days.sort_unstable_by_key(|day| day.day);
    }

    Ok(map.into_iter().collect::<DashMap<_, _>>())
}
