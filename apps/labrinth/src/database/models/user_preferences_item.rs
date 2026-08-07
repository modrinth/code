use sqlx::types::Json;
use crate::database::Executor;
use crate::database::models::DBUserId;
use crate::models::v3::preferences::UserPreferences;

pub struct DBUserPreferences;

impl DBUserPreferences {
    pub async fn get<'a, E>(
        user_id: DBUserId,
        exec: E,
    ) -> Result<Option<UserPreferences>, sqlx::Error>
    where
        E: Executor<'a, Database = sqlx::Postgres>,
    {
        let row = sqlx::query!(
            r#"
            SELECT preferences AS "preferences: Json<UserPreferences>"
            FROM user_preferences
            WHERE user_id = $1
            "#,
            user_id.0,
        )
        .fetch_optional(exec)
        .await?;

        Ok(row.map(|row| row.preferences.0))
    }

    pub async fn upsert<'a, E>(
        user_id: DBUserId,
        preferences: &UserPreferences,
        exec: E,
    ) -> Result<(), sqlx::Error>
    where
        E: Executor<'a, Database = sqlx::Postgres>,
    {
        sqlx::query!(
            r#"
            INSERT INTO user_preferences (user_id, preferences)
            VALUES ($1, $2)
            ON CONFLICT (user_id) DO UPDATE
                SET preferences = EXCLUDED.preferences
            "#,
            user_id.0,
            Json(preferences) as Json<&UserPreferences>,
        )
        .execute(exec)
        .await?;

        Ok(())
    }
}
