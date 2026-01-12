use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::database::models::{DBProjectId, DBUserId};

const LOCK_EXPIRY_MINUTES: i64 = 15;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBModerationLock {
    pub project_id: DBProjectId,
    pub moderator_id: DBUserId,
    pub locked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationLockWithUser {
    pub project_id: DBProjectId,
    pub moderator_id: DBUserId,
    pub moderator_username: String,
    pub moderator_avatar_url: Option<String>,
    pub locked_at: DateTime<Utc>,
    pub expired: bool,
}

impl DBModerationLock {
    /// Check if a lock is expired (older than 15 minutes)
    pub fn is_expired(&self) -> bool {
        Utc::now()
            .signed_duration_since(self.locked_at)
            .num_minutes()
            >= LOCK_EXPIRY_MINUTES
    }

    /// Try to acquire or refresh a lock for a project.
    /// Returns Ok(Ok(())) if lock acquired/refreshed, Ok(Err(lock)) if blocked by another moderator.
    pub async fn acquire(
        project_id: DBProjectId,
        moderator_id: DBUserId,
        pool: &PgPool,
    ) -> Result<Result<(), ModerationLockWithUser>, sqlx::Error> {
        // First check if there's an existing lock
        let existing = Self::get_with_user(project_id, pool).await?;

        if let Some(lock) = existing {
            // Same moderator - refresh the lock
            if lock.moderator_id == moderator_id {
                sqlx::query!(
					"UPDATE moderation_locks SET locked_at = NOW() WHERE project_id = $1",
					project_id as DBProjectId
				)
				.execute(pool)
				.await?;
                return Ok(Ok(()));
            }

            // Different moderator but lock expired - take over
            if lock.expired {
                sqlx::query!(
					"UPDATE moderation_locks SET moderator_id = $1, locked_at = NOW() WHERE project_id = $2",
					moderator_id as DBUserId,
					project_id as DBProjectId
				)
				.execute(pool)
				.await?;
                return Ok(Ok(()));
            }

            // Different moderator, not expired - blocked
            return Ok(Err(lock));
        }

        // No existing lock - create new one
        sqlx::query!(
            "INSERT INTO moderation_locks (project_id, moderator_id, locked_at)
			VALUES ($1, $2, NOW())
			ON CONFLICT (project_id) DO UPDATE
			SET moderator_id = EXCLUDED.moderator_id, locked_at = EXCLUDED.locked_at",
            project_id as DBProjectId,
            moderator_id as DBUserId
        )
        .execute(pool)
        .await?;

        Ok(Ok(()))
    }

    /// Get lock status for a project, including moderator username
    pub async fn get_with_user(
        project_id: DBProjectId,
        pool: &PgPool,
    ) -> Result<Option<ModerationLockWithUser>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
			SELECT
				ml.project_id,
				ml.moderator_id,
				u.username as moderator_username,
				u.avatar_url as moderator_avatar_url,
				ml.locked_at
			FROM moderation_locks ml
			INNER JOIN users u ON u.id = ml.moderator_id
			WHERE ml.project_id = $1
			"#,
            project_id as DBProjectId
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| {
            let locked_at: DateTime<Utc> = r.locked_at;
            let expired =
                Utc::now().signed_duration_since(locked_at).num_minutes()
                    >= LOCK_EXPIRY_MINUTES;

            ModerationLockWithUser {
                project_id: DBProjectId(r.project_id),
                moderator_id: DBUserId(r.moderator_id),
                moderator_username: r.moderator_username,
                moderator_avatar_url: r.moderator_avatar_url,
                locked_at,
                expired,
            }
        }))
    }

    /// Release a lock (only if held by the specified moderator)
    pub async fn release(
        project_id: DBProjectId,
        moderator_id: DBUserId,
        pool: &PgPool,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
			"DELETE FROM moderation_locks WHERE project_id = $1 AND moderator_id = $2",
			project_id as DBProjectId,
			moderator_id as DBUserId
		)
		.execute(pool)
		.await?;

        Ok(result.rows_affected() > 0)
    }

    /// Clean up expired locks (can be called periodically)
    pub async fn cleanup_expired(pool: &PgPool) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
			"DELETE FROM moderation_locks WHERE locked_at < NOW() - INTERVAL '15 minutes'"
		)
		.execute(pool)
		.await?;

        Ok(result.rows_affected())
    }

    /// Delete all moderation locks (admin only)
    pub async fn delete_all(pool: &PgPool) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!("DELETE FROM moderation_locks")
            .execute(pool)
            .await?;

        Ok(result.rows_affected())
    }
}
