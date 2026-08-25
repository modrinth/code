//! Logic for fetching and caching revenue estimations from our ad provider.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Days, NaiveDate, Utc};
use dashmap::DashMap;
use eyre::Result;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use xredis::RedisPool;

use crate::{
    routes::ApiError,
    util::{error::Context, time::YearMonth},
};

const REDIS_KEY: &str = "aditude_month_estimate:v1";

#[derive(Debug, Serialize, Deserialize)]
pub struct PeriodEstimate {
    pub period: YearMonth,
    pub days: Vec<DayEstimate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DayEstimate {
    pub date: NaiveDate,
    pub raw_estimated_revenue_usd: Decimal,
    pub impressions: u128,
}

/// Get per-month and per-day estimated ad provider info for an inclusive date
/// range.
///
/// Dates are interpreted as Phoenix calendar dates. They are converted to UTC
/// instants only when constructing the Aditude API request.
pub async fn estimate(
    aditude: &aditude::Client,
    redis: &RedisPool,
    periods: &[YearMonth],
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<PeriodEstimate>> {
    if start_date > end_date {
        return Err(eyre::eyre!("estimate start date is after end date"));
    }

    let mut periods = redis
        .get_cached_keys(REDIS_KEY, periods, |periods| async move {
            fetch_estimates(aditude, &periods, start_date, end_date)
                .await
                .map_err(ApiError::Internal)
        })
        .await?;
    periods.sort_unstable_by_key(|p| p.period);
    Ok(periods)
}

async fn fetch_estimates(
    aditude: &aditude::Client,
    periods: &[YearMonth],
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<DashMap<YearMonth, PeriodEstimate>> {
    let range_start = aditude::phoenix_midnight(start_date);
    let range_end_date = end_date
        .checked_add_days(Days::new(1))
        .wrap_err("calculating day after estimate range end")?;
    let range_end = aditude::phoenix_midnight(range_end_date);

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

    Ok(period_estimates(metrics, periods))
}

#[derive(Default)]
struct PartialDayEstimate {
    raw_estimated_revenue_usd: Option<Decimal>,
    impressions: Option<u128>,
}

fn period_estimates(
    metrics: aditude::v2::Metrics,
    periods: &[YearMonth],
) -> DashMap<YearMonth, PeriodEstimate> {
    let requested_periods = periods.iter().copied().collect::<HashSet<_>>();
    let mut partial_days = HashMap::<DateTime<Utc>, PartialDayEstimate>::new();
    for response in metrics.responses {
        for row in response.rows {
            let day = partial_days.entry(row.time).or_default();
            if let Some(raw_estimated_revenue_usd) = row.revenue {
                day.raw_estimated_revenue_usd = Some(raw_estimated_revenue_usd);
            }
            if let Some(impressions) = row.impressions {
                day.impressions = Some(impressions);
            }
        }
    }

    let mut map = HashMap::<YearMonth, PeriodEstimate>::new();
    for (time, day) in partial_days {
        let Some(raw_estimated_revenue_usd) = day.raw_estimated_revenue_usd
        else {
            continue;
        };
        let Some(impressions) = day.impressions else {
            continue;
        };

        let date = aditude::phoenix_date(time);
        let period = YearMonth::from_day1(date);
        if !requested_periods.contains(&period) {
            continue;
        }
        let period_estimate = map.entry(period).or_insert(PeriodEstimate {
            period,
            days: Vec::new(),
        });
        period_estimate.days.push(DayEstimate {
            date,
            raw_estimated_revenue_usd,
            impressions,
        });
    }

    // we have no clue if the Aditude row return order is stable,
    // so for safety sort the days here
    for period in map.values_mut() {
        period.days.sort_unstable_by_key(|day| day.date);
    }

    // for any `periods` for which we don't have data yet,
    // give them an empty estimate dataset
    for &period in periods {
        map.entry(period).or_insert(PeriodEstimate {
            period,
            days: Vec::new(),
        });
    }

    map.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use rust_decimal::dec;

    use super::*;

    #[test]
    fn combines_metrics_from_separate_responses() {
        let time = Utc.with_ymd_and_hms(2026, 8, 1, 7, 0, 0).unwrap();
        let metrics = aditude::v2::Metrics {
            responses: vec![
                aditude::v2::Response {
                    rows: vec![aditude::v2::Row {
                        impressions: None,
                        revenue: Some(dec!(12.34)),
                        time,
                    }],
                },
                aditude::v2::Response {
                    rows: vec![aditude::v2::Row {
                        impressions: Some(5678),
                        revenue: None,
                        time,
                    }],
                },
            ],
        };
        let period =
            YearMonth::from_day1(NaiveDate::from_ymd_opt(2026, 8, 1).unwrap());

        let estimates = period_estimates(metrics, &[period]);
        let estimate = estimates.get(&period).unwrap();

        assert_eq!(estimate.days.len(), 1);
        assert_eq!(estimate.days[0].date, period.date());
        assert_eq!(estimate.days[0].raw_estimated_revenue_usd, dec!(12.34));
        assert_eq!(estimate.days[0].impressions, 5678);
    }
}
