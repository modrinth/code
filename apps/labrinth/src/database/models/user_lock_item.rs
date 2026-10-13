use chrono::{DateTime, Utc};
use eyre::{Result, WrapErr};
use serde::{Deserialize, Serialize};

use crate::database::models::DBUserId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBUserLock {
    pub user_id: DBUserId,
    pub locked_by: DBUserId,
    pub reason: String,
    pub created: DateTime<Utc>,
}

impl DBUserLock {
    pub async fn upsert<'a, E>(
        user_id: DBUserId,
        locked_by: DBUserId,
        reason: &str,
        exec: E,
    ) -> Result<()>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        sqlx::query!(
            r#"
            INSERT INTO user_locks (user_id, locked_by, reason)
            VALUES ($1, $2, $3)
            ON CONFLICT (user_id) DO UPDATE
            SET locked_by = EXCLUDED.locked_by,
                reason = EXCLUDED.reason,
                created = NOW()
            "#,
            user_id as DBUserId,
            locked_by as DBUserId,
            reason,
        )
        .execute(exec)
        .await
        .wrap_err("upserting user lock")?;

        Ok(())
    }

    pub async fn exists<'a, E>(user_id: DBUserId, exec: E) -> Result<bool>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        let exists = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM user_locks WHERE user_id = $1) AS "exists!""#,
            user_id as DBUserId,
        )
        .fetch_one(exec)
        .await
        .wrap_err("checking user lock")?;

        Ok(exists)
    }

    /// Returns `false` if the user was not locked.
    pub async fn delete<'a, E>(user_id: DBUserId, exec: E) -> Result<bool>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "DELETE FROM user_locks WHERE user_id = $1",
            user_id as DBUserId,
        )
        .execute(exec)
        .await
        .wrap_err("deleting user lock")?;

        Ok(result.rows_affected() > 0)
    }
}
