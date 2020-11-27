use super::ids::{ModId, UserId};

pub struct User {
    pub id: UserId,
    pub github_id: Option<i64>,
    pub username: String,
    pub name: Option<String>,
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
            self.name.as_ref(),
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

    pub async fn get_from_username<'a, 'b, E>(
        username: String,
        executor: E,
    ) -> Result<Option<Self>, sqlx::error::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT u.id, u.github_id, u.name, u.email,
                u.avatar_url, u.bio,
                u.created, u.role
            FROM users u
            WHERE u.username = $1
            ",
            username
        )
        .fetch_optional(executor)
        .await?;

        if let Some(row) = result {
            Ok(Some(User {
                id: UserId(row.id),
                github_id: row.github_id,
                name: row.name,
                email: row.email,
                avatar_url: row.avatar_url,
                username,
                bio: row.bio,
                created: row.created,
                role: row.role,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_many<'a, E>(user_ids: Vec<UserId>, exec: E) -> Result<Vec<User>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        use futures::stream::TryStreamExt;

        let user_ids_parsed: Vec<i64> = user_ids.into_iter().map(|x| x.0).collect();
        let users = sqlx::query!(
            "
            SELECT u.id, u.github_id, u.name, u.email,
                u.avatar_url, u.username, u.bio,
                u.created, u.role FROM users u
            WHERE u.id IN (SELECT * FROM UNNEST($1::bigint[]))
            ",
            &user_ids_parsed
        )
        .fetch_many(exec)
        .try_filter_map(|e| async {
            Ok(e.right().map(|u| User {
                id: UserId(u.id),
                github_id: u.github_id,
                name: u.name,
                email: u.email,
                avatar_url: u.avatar_url,
                username: u.username,
                bio: u.bio,
                created: u.created,
                role: u.role,
            }))
        })
        .try_collect::<Vec<User>>()
        .await?;

        Ok(users)
    }

    pub async fn get_mods<'a, E>(user_id: UserId, exec: E) -> Result<Vec<ModId>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        use futures::stream::TryStreamExt;

        let mods = sqlx::query!(
            "
            SELECT m.id FROM mods m
            INNER JOIN team_members tm ON tm.team_id = m.team_id
            WHERE tm.user_id = $1
            ",
            user_id as UserId,
        )
        .fetch_many(exec)
        .try_filter_map(|e| async { Ok(e.right().map(|m| ModId(m.id))) })
        .try_collect::<Vec<ModId>>()
        .await?;

        Ok(mods)
    }
}
