use std::collections::HashMap;

use chrono::NaiveDate;
use rust_decimal::Decimal;

use super::DatabaseError;

#[derive(Debug, Clone)]
pub struct DBPayoutPeriod {
    pub period: NaiveDate,
    pub raw_actual_aditude_revenue_usd: Decimal,
    pub adjustments: serde_json::Value,
    pub days: Vec<DBPayoutPeriodDay>,
    pub has_scheduled_run: bool,
    pub has_running_run: bool,
    pub has_succeeded_run: bool,
}

#[derive(Debug, Clone)]
pub struct DBPayoutPeriodDay {
    pub period: NaiveDate,
    pub date: NaiveDate,
    pub raw_estimated_aditude_revenue_usd: Decimal,
    pub aditude_impressions: i64,
}

impl DBPayoutPeriod {
    pub async fn get_many<'a, E>(
        periods: &[NaiveDate],
        exec: E,
    ) -> Result<Vec<Self>, DatabaseError>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let period_rows = sqlx::query!(
            r#"
			SELECT
				payout_periods.period,
				payout_periods.raw_actual_aditude_revenue_usd,
				payout_periods.adjustments,
				EXISTS (
					SELECT 1
					FROM payout_runs
					WHERE payout_runs.period = payout_periods.period
						AND payout_runs.status = 'scheduled'
				) AS "has_scheduled_run!",
				EXISTS (
					SELECT 1
					FROM payout_runs
					WHERE payout_runs.period = payout_periods.period
						AND payout_runs.status = 'running'
				) AS "has_running_run!",
				EXISTS (
					SELECT 1
					FROM payout_runs
					WHERE payout_runs.period = payout_periods.period
						AND payout_runs.status = 'succeeded'
				) AS "has_succeeded_run!"
			FROM payout_periods
			WHERE payout_periods.period = ANY($1)
			"#,
            periods,
        )
        .fetch_all(exec)
        .await?;

        let mut periods = period_rows
            .into_iter()
            .map(|row| {
                (
                    row.period,
                    Self {
                        period: row.period,
                        raw_actual_aditude_revenue_usd: row
                            .raw_actual_aditude_revenue_usd,
                        adjustments: row.adjustments,
                        days: Vec::new(),
                        has_scheduled_run: row.has_scheduled_run,
                        has_running_run: row.has_running_run,
                        has_succeeded_run: row.has_succeeded_run,
                    },
                )
            })
            .collect::<HashMap<_, _>>();

        let stored_period_dates = periods.keys().copied().collect::<Vec<_>>();
        let days = sqlx::query!(
            r#"
			SELECT
				period,
				date,
				raw_estimated_aditude_revenue_usd,
				aditude_impressions
			FROM payout_period_days
			WHERE period = ANY($1)
			ORDER BY period, date
			"#,
            &stored_period_dates,
        )
        .fetch_all(exec)
        .await?;

        for day in days {
            if let Some(period) = periods.get_mut(&day.period) {
                period.days.push(DBPayoutPeriodDay {
                    period: day.period,
                    date: day.date,
                    raw_estimated_aditude_revenue_usd: day
                        .raw_estimated_aditude_revenue_usd,
                    aditude_impressions: day.aditude_impressions,
                });
            }
        }

        let mut periods = periods.into_values().collect::<Vec<_>>();
        periods.sort_unstable_by_key(|period| period.period);
        Ok(periods)
    }
}
