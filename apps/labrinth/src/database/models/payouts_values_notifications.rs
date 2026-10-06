use crate::database::models::DBUserId;
use chrono::{DateTime, Utc};
use eyre::{Result, WrapErr};

pub struct PayoutsValuesNotification {
    pub id: i32,
    pub user_id: DBUserId,
    pub date_available: DateTime<Utc>,
}

impl PayoutsValuesNotification {
    pub async fn unnotified_users_with_available_payouts_with_limit(
        exec: impl sqlx::PgExecutor<'_>,
        limit: i64,
    ) -> Result<Vec<PayoutsValuesNotification>> {
        Ok(sqlx::query_as!(
            QueryResult,
            "
			SELECT
			  id,
			  user_id,
			  date_available
			FROM payouts_values_notifications
			WHERE
			  notified = FALSE
			  AND date_available <= NOW()
			FOR UPDATE SKIP LOCKED
			LIMIT $1
			",
            limit,
        )
        .fetch_all(exec)
        .await
        .wrap_err("fetching users with unnotified available payouts")?
        .into_iter()
        .map(Into::into)
        .collect())
    }

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
        .wrap_err("marking payout value notifications as notified")?;

        Ok(())
    }
}

pub async fn synchronize_future_payout_values(
    exec: impl sqlx::PgExecutor<'_>,
    limit: i64,
) -> Result<()> {
    sqlx::query!(
        "
		INSERT INTO payouts_values_notifications (date_available, user_id, notified)
		SELECT DISTINCT date_available, user_id, false notified
		FROM payouts_values
		WHERE date_available > NOW()
		LIMIT $1
		ON CONFLICT (date_available, user_id) DO NOTHING
		",
        limit,
    )
    .execute(exec)
    .await
    .wrap_err("synchronizing future payout value notifications")?;

    Ok(())
}

struct QueryResult {
    id: i32,
    user_id: i64,
    date_available: DateTime<Utc>,
}

impl From<QueryResult> for PayoutsValuesNotification {
    fn from(result: QueryResult) -> Self {
        PayoutsValuesNotification {
            id: result.id,
            user_id: DBUserId(result.user_id),
            date_available: result.date_available,
        }
    }
}
