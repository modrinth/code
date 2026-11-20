use crate::database::models::DBUserId;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};

pub struct PayoutsValuesNotification {
    pub id: i32,
    pub user_id: DBUserId,
    pub date_available: DateTime<Utc>,
}

impl PayoutsValuesNotification {
    pub async fn set_notified_many(
        ids: &[i32],
        exec: impl sqlx::PgExecutor<'_>,
    ) -> Result<()> {
        sqlx::query!(
            "
			UPDATE payouts_values_notifications
			SET notified = TRUE
			WHERE id = ANY($1)
			",
            &ids[..],
        )
        .execute(exec)
        .await
        .context("marking payout value notifications as notified")?;

        Ok(())
    }
}
