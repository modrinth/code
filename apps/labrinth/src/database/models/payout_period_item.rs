use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::types::Json;

use super::DatabaseError;
use crate::queue::payout_run::{Adjustment, PayoutRunPayload};

#[derive(Debug, Clone)]
pub struct DBPayoutPeriod {
    pub period: NaiveDate,
    pub raw_actual_aditude_revenue_usd: Decimal,
    pub adjustments: Vec<Adjustment>,
    pub active_run_payload: Option<PayoutRunPayload>,
    pub active_run_execute_at: Option<DateTime<Utc>>,
    pub days: Vec<DBPayoutPeriodDay>,
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
				payout_periods.adjustments AS "adjustments: Json<Vec<Adjustment>>",
				active_run.payload AS "active_run_payload: Json<PayoutRunPayload>",
				active_run.execute_at AS active_run_execute_at,
				EXISTS (
					SELECT 1
					FROM payout_runs
					WHERE payout_runs.period = payout_periods.period
						AND payout_runs.status = 'succeeded'
				) AS "has_succeeded_run!"
			FROM payout_periods
			LEFT JOIN payout_runs active_run
				ON active_run.period = payout_periods.period
				AND active_run.status IN ('scheduled', 'running')
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
                        adjustments: row.adjustments.0,
                        active_run_payload: row
                            .active_run_payload
                            .map(|payload| payload.0),
                        active_run_execute_at: row.active_run_execute_at,
                        days: Vec::new(),
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
