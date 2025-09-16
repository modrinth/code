use crate::database::models::DatabaseError;

pub async fn synchronize_future_payout_values(
    exec: impl sqlx::PgExecutor<'_>,
) -> Result<(), DatabaseError> {
    sqlx::query!(
        "
		INSERT INTO payouts_values_notifications (date_available, user_id, notified)
		SELECT DISTINCT date_available, user_id, false notified
		FROM payouts_values
		WHERE date_available > NOW()
		ON CONFLICT (date_available, user_id) DO NOTHING
		",
    )
    .execute(exec)
    .await?;

    Ok(())
}
