use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::DatabaseError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBPayoutVariance {
    pub applied_on: NaiveDate,
    pub variance: Decimal,
}

impl DBPayoutVariance {
    pub async fn get_all(
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<Vec<Self>, DatabaseError> {
        let variances = sqlx::query_as!(
            Self,
            r#"
			SELECT applied_on, variance
			FROM payouts_variance
			ORDER BY applied_on
			"#,
        )
        .fetch_all(exec)
        .await?;

        Ok(variances)
    }

    pub async fn upsert(
        &self,
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            r#"
			INSERT INTO payouts_variance (applied_on, variance)
			VALUES ($1, $2)
			ON CONFLICT (applied_on) DO UPDATE
			SET variance = EXCLUDED.variance
			"#,
            self.applied_on,
            self.variance,
        )
        .execute(exec)
        .await?;

        Ok(())
    }
}
