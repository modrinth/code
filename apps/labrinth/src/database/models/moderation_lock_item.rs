use crate::database::PgPool;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::database::models::{DBProjectId, DBUserId};

pub const LOCK_EXPIRY_MINUTES: i64 = 15;

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
    pub expires_at: DateTime<Utc>,
    pub expired: bool,
}

impl DBModerationLock {
    /// Try to acquire or refresh a lock for a project atomically.
    /// Returns Ok(Ok(())) if lock acquired/refreshed, Ok(Err(lock)) if blocked by another moderator.
    pub async fn acquire(
        project_id: DBProjectId,
        moderator_id: DBUserId,
        pool: &PgPool,
    ) -> Result<Result<(), ModerationLockWithUser>, sqlx::Error> {
        // Atomic upsert that always returns the post-operation row. When the lock is held by
        // another moderator and is still valid, the CASE branches write the existing values
        // back (a harmless self-update), so `RETURNING` always yields a row describing the
        // current holder. We cannot rely on a bare `DO UPDATE ... WHERE` because:
        //   * `WHERE` that evaluates false suppresses the update *and* `RETURNING`, and
        //   * data-modifying CTEs share a snapshot with the enclosing SELECT, so a plain
        //     `SELECT ... FROM moderation_locks` in the same statement cannot see a row
        //     inserted by the CTE above it.
        let row = sqlx::query!(
            r#"
            WITH upsert AS (
                INSERT INTO moderation_locks (project_id, moderator_id, locked_at)
                VALUES ($1, $2, NOW())
                ON CONFLICT (project_id) DO UPDATE SET
                    moderator_id = CASE
                        WHEN moderation_locks.moderator_id = EXCLUDED.moderator_id
                          OR moderation_locks.locked_at < NOW() - ($3::bigint * INTERVAL '1 minute')
                        THEN EXCLUDED.moderator_id
                        ELSE moderation_locks.moderator_id
                    END,
                    locked_at = CASE
                        WHEN moderation_locks.moderator_id = EXCLUDED.moderator_id
                          OR moderation_locks.locked_at < NOW() - ($3::bigint * INTERVAL '1 minute')
                        THEN EXCLUDED.locked_at
                        ELSE moderation_locks.locked_at
                    END
                RETURNING moderator_id, locked_at
            )
            SELECT
                upsert.moderator_id,
                upsert.locked_at,
                u.username   AS moderator_username,
                u.avatar_url AS moderator_avatar_url
            FROM upsert
            INNER JOIN users u ON u.id = upsert.moderator_id
            "#,
            project_id as DBProjectId,
            moderator_id as DBUserId,
            LOCK_EXPIRY_MINUTES,
        )
        .fetch_one(pool)
        .await?;

        let locked_at: DateTime<Utc> = row.locked_at;
        let expires_at = locked_at + Duration::minutes(LOCK_EXPIRY_MINUTES);
        let expired = Utc::now() >= expires_at;

        if row.moderator_id == moderator_id.0 {
            Ok(Ok(()))
        } else {
            Ok(Err(ModerationLockWithUser {
                project_id,
                moderator_id: DBUserId(row.moderator_id),
                moderator_username: row.moderator_username,
                moderator_avatar_url: row.moderator_avatar_url,
                locked_at,
                expires_at,
                expired,
            }))
        }
    }

    /// Reassign the lock to `moderator_id`, even when another moderator holds an active lock.
    /// Used only after explicit client confirmation (override flow).
    pub async fn force_acquire(
        project_id: DBProjectId,
        moderator_id: DBUserId,
        pool: &PgPool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO moderation_locks (project_id, moderator_id, locked_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (project_id) DO UPDATE SET
                moderator_id = EXCLUDED.moderator_id,
                locked_at = EXCLUDED.locked_at
            "#,
        )
        .bind(project_id)
        .bind(moderator_id)
        .execute(pool)
        .await?;

        Ok(())
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
            let expires_at = locked_at + Duration::minutes(LOCK_EXPIRY_MINUTES);
            let expired = Utc::now() >= expires_at;

            ModerationLockWithUser {
                project_id: DBProjectId(r.project_id),
                moderator_id: DBUserId(r.moderator_id),
                moderator_username: r.moderator_username,
                moderator_avatar_url: r.moderator_avatar_url,
                locked_at,
                expires_at,
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
            "DELETE FROM moderation_locks WHERE locked_at < NOW() - ($1::bigint * INTERVAL '1 minute')",
            LOCK_EXPIRY_MINUTES,
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
