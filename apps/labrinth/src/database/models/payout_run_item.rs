use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use strum::{EnumString, IntoStaticStr};

use super::{DBPayoutRunId, DBUserId, DatabaseError};
use crate::queue::payout_run::PayoutRunPayload;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    EnumString,
    IntoStaticStr,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum PayoutRunStatus {
    Scheduled,
    Running,
    Cancelled,
    Failed,
    Succeeded,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayoutRunError {
    pub error: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl From<crate::models::error::ApiError<'_>> for PayoutRunError {
    fn from(error: crate::models::error::ApiError<'_>) -> Self {
        Self {
            error: error.error.to_owned(),
            description: error.description,
            details: error.details,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBPayoutRun {
    pub id: DBPayoutRunId,
    pub period: NaiveDate,
    pub payload: PayoutRunPayload,
    pub status: PayoutRunStatus,
    pub started_at: DateTime<Utc>,
    pub started_by: DBUserId,
    pub execute_at: DateTime<Utc>,
    pub processing_started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancelled_by: Option<DBUserId>,
    pub error: Option<PayoutRunError>,
}

impl DBPayoutRun {
    pub async fn upsert(
        &self,
        exec: impl crate::database::Executor<'_, Database = sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            r#"
			INSERT INTO payout_runs (
				id,
				period,
				payload,
				status,
				started_at,
				started_by,
				execute_at,
				processing_started_at,
				finished_at,
				cancelled_at,
				cancelled_by,
				error
			)
			VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
			ON CONFLICT (id) DO UPDATE SET
				period = EXCLUDED.period,
				payload = EXCLUDED.payload,
				status = EXCLUDED.status,
				started_at = EXCLUDED.started_at,
				started_by = EXCLUDED.started_by,
				execute_at = EXCLUDED.execute_at,
				processing_started_at = EXCLUDED.processing_started_at,
				finished_at = EXCLUDED.finished_at,
				cancelled_at = EXCLUDED.cancelled_at,
				cancelled_by = EXCLUDED.cancelled_by,
				error = EXCLUDED.error
			"#,
            self.id.0,
            self.period,
            Json(&self.payload) as Json<&PayoutRunPayload>,
            <&'static str>::from(self.status),
            self.started_at,
            self.started_by.0,
            self.execute_at,
            self.processing_started_at,
            self.finished_at,
            self.cancelled_at,
            self.cancelled_by.map(|id| id.0),
            self.error.as_ref().map(Json) as Option<Json<&PayoutRunError>>,
        )
        .execute(exec)
        .await?;

        Ok(())
    }
}
