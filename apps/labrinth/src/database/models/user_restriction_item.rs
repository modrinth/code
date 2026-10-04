use chrono::{DateTime, Utc};
use eyre::{Result, WrapErr};
use serde::{Deserialize, Serialize};

use crate::database::models::DBUserId;
use crate::models::pats::Scopes;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBUserRestriction {
    pub user_id: DBUserId,
    pub removed_perms: Scopes,
    pub reason: Option<String>,
    pub private_reason: Option<String>,
    pub restricted_by: DBUserId,
    pub updated: DateTime<Utc>,
}

impl DBUserRestriction {
    pub async fn upsert<'a, E>(
        user_id: DBUserId,
        removed_perms: Scopes,
        reason: Option<&str>,
        private_reason: Option<&str>,
        restricted_by: DBUserId,
        exec: E,
    ) -> Result<()>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        sqlx::query!(
            r#"
            INSERT INTO user_restrictions (user_id, removed_perms, reason, private_reason, restricted_by)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (user_id) DO UPDATE
            SET removed_perms = EXCLUDED.removed_perms,
                reason = EXCLUDED.reason,
                private_reason = EXCLUDED.private_reason,
                restricted_by = EXCLUDED.restricted_by,
                updated = NOW()
            "#,
            user_id as DBUserId,
            removed_perms.to_postgres(),
            reason,
            private_reason,
            restricted_by as DBUserId,
        )
        .execute(exec)
        .await
        .wrap_err("upserting user restriction")?;

        Ok(())
    }

    /// Returns `false` if the user was not restricted.
    pub async fn delete<'a, E>(user_id: DBUserId, exec: E) -> Result<bool>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "DELETE FROM user_restrictions WHERE user_id = $1",
            user_id as DBUserId,
        )
        .execute(exec)
        .await
        .wrap_err("deleting user restriction")?;

        Ok(result.rows_affected() > 0)
    }
}
