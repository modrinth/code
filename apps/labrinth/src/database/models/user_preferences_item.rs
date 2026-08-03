use sqlx::types::Json;

use crate::{
    database::{Executor, PgTransaction},
    models::v3::user_preferences::UserPreferences,
};

use super::DBUserId;

pub struct DBUserPreferences;

impl DBUserPreferences {
    pub async fn get<'a, E>(
        user_id: DBUserId,
        exec: E,
    ) -> Result<Option<UserPreferences>, sqlx::Error>
    where
        E: Executor<'a, Database = sqlx::Postgres>,
    {
        let preferences = sqlx::query_scalar!(
            r#"
			SELECT preferences AS "preferences: Json<UserPreferences>"
			FROM user_preferences
			WHERE user_id = $1
			"#,
            user_id.0,
        )
        .fetch_optional(exec)
        .await?
        .map(|preferences| preferences.0);

        Ok(preferences)
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

    pub async fn insert_if_absent(
        user_id: DBUserId,
        preferences: &UserPreferences,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
			INSERT INTO user_preferences (user_id, preferences)
			VALUES ($1, $2)
			ON CONFLICT (user_id) DO NOTHING
			"#,
            user_id.0,
            Json(preferences) as Json<&UserPreferences>,
        )
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }

    pub async fn get_for_update(
        user_id: DBUserId,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<UserPreferences, sqlx::Error> {
        let preferences = sqlx::query_scalar!(
            r#"
			SELECT preferences AS "preferences: Json<UserPreferences>"
			FROM user_preferences
			WHERE user_id = $1
			FOR UPDATE
			"#,
            user_id.0,
        )
        .fetch_one(&mut *transaction)
        .await?;

        Ok(preferences.0)
    }
}
