use super::ids::UserId;

pub struct User {
    pub id: UserId,
    pub github_id: Option<i64>,
    pub username: String,
    pub name: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub created: chrono::DateTime<chrono::Utc>,
    pub role: String,
}

impl User {
    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::error::Error> {
        sqlx::query!(
            "
            INSERT INTO users (
                id, github_id, username, name, email,
                avatar_url, bio, created
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8
            )
            ",
            self.id as UserId,
            self.github_id,
            &self.username,
            &self.name,
            self.email.as_ref(),
            self.avatar_url.as_ref(),
            self.bio.as_ref(),
            self.created,
        )
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }
    pub async fn get<'a, 'b, E>(id: UserId, executor: E) -> Result<Option<Self>, sqlx::error::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT u.github_id, u.name, u.email,
                u.avatar_url, u.username, u.bio,
                u.created, u.role
            FROM users u
            WHERE u.id = $1
            ",
            id as UserId,
        )
        .fetch_optional(executor)
        .await?;

        if let Some(row) = result {
            Ok(Some(User {
                id,
                github_id: row.github_id,
                name: row.name,
                email: row.email,
                avatar_url: row.avatar_url,
                username: row.username,
                bio: row.bio,
                created: row.created,
                role: row.role,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_from_github_id<'a, 'b, E>(
        github_id: u64,
        executor: E,
    ) -> Result<Option<Self>, sqlx::error::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT u.id, u.name, u.email,
                u.avatar_url, u.username, u.bio,
                u.created, u.role
            FROM users u
            WHERE u.github_id = $1
            ",
            github_id as i64,
        )
        .fetch_optional(executor)
        .await?;

        if let Some(row) = result {
            Ok(Some(User {
                id: UserId(row.id),
                github_id: Some(github_id as i64),
                name: row.name,
                email: row.email,
                avatar_url: row.avatar_url,
                username: row.username,
                bio: row.bio,
                created: row.created,
                role: row.role,
            }))
        } else {
            Ok(None)
        }
    }
}
