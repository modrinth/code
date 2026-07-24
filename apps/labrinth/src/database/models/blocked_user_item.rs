use crate::database::models::DBUserId;

pub struct DBBlockedUser {
    pub user_id: DBUserId,
    pub blocked_id: DBUserId,
}

impl DBBlockedUser {
    pub async fn insert<'a, E>(&self, exec: E) -> Result<(), sqlx::Error>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        sqlx::query!(
            "
            INSERT INTO blocked_users (user_id, blocked_id)
            VALUES ($1, $2)
            ON CONFLICT (user_id, blocked_id) DO NOTHING
            ",
            self.user_id.0,
            self.blocked_id.0,
        )
        .execute(exec)
        .await?;

        Ok(())
    }

    pub async fn is_blocked<'a, E>(
        user_id: DBUserId,
        blocked_id: DBUserId,
        exec: E,
    ) -> Result<bool, sqlx::Error>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        let blocked = sqlx::query_scalar!(
            "
            SELECT 1 FROM blocked_users
            WHERE user_id = $1 AND blocked_id = $2
            ",
            user_id.0,
            blocked_id.0,
        )
        .fetch_optional(exec)
        .await?;

        Ok(blocked.is_some())
    }

    pub async fn remove<'a, E>(
        user_id: DBUserId,
        blocked_id: DBUserId,
        exec: E,
    ) -> Result<(), sqlx::Error>
    where
        E: crate::database::Executor<'a, Database = sqlx::Postgres>,
    {
        sqlx::query!(
            "
            DELETE FROM blocked_users
            WHERE user_id = $1 AND blocked_id = $2
            ",
            user_id.0,
            blocked_id.0,
        )
        .execute(exec)
        .await?;

        Ok(())
    }
}
