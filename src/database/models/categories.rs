use super::ids::*;
use super::DatabaseError;
use futures::TryStreamExt;

pub struct Loader {
    pub id: LoaderId,
    pub loader: String,
}

pub struct GameVersion {
    pub id: GameVersionId,
    pub version: String,
}

pub struct Category {
    pub id: CategoryId,
    pub category: String,
}

pub struct CategoryBuilder<'a> {
    pub name: Option<&'a str>,
}

impl Category {
    pub fn builder() -> CategoryBuilder<'static> {
        CategoryBuilder { name: None }
    }

    pub async fn get_id<'a, E>(name: &str, exec: E) -> Result<Option<CategoryId>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(DatabaseError::InvalidIdentifier(name.to_string()));
        }

        let result = sqlx::query!(
            "
            SELECT id FROM categories
            WHERE category = $1
            ",
            name
        )
        .fetch_optional(exec)
        .await?;

        Ok(result.map(|r| CategoryId(r.id)))
    }

    pub async fn get_name<'a, E>(id: CategoryId, exec: E) -> Result<String, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT category FROM categories
            WHERE id = $1
            ",
            id as CategoryId
        )
        .fetch_one(exec)
        .await?;

        Ok(result.category)
    }

    pub async fn list<'a, E>(exec: E) -> Result<Vec<String>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT category FROM categories
            "
        )
        .fetch_many(exec)
        .try_filter_map(|e| async { Ok(e.right().map(|c| c.category)) })
        .try_collect::<Vec<String>>()
        .await?;

        Ok(result)
    }

    pub async fn remove<'a, E>(name: &str, exec: E) -> Result<Option<()>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        use sqlx::Done;

        let result = sqlx::query!(
            "
            DELETE FROM categories
            WHERE category = $1
            ",
            name
        )
        .execute(exec)
        .await?;

        if result.rows_affected() == 0 {
            // Nothing was deleted
            Ok(None)
        } else {
            Ok(Some(()))
        }
    }
}

impl<'a> CategoryBuilder<'a> {
    /// The name of the category.  Must be ASCII alphanumeric or `-`/`_`
    pub fn name(self, name: &'a str) -> Result<CategoryBuilder<'a>, DatabaseError> {
        if name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            Ok(Self { name: Some(name) })
        } else {
            Err(DatabaseError::InvalidIdentifier(name.to_string()))
        }
    }

    pub async fn insert<'b, E>(self, exec: E) -> Result<CategoryId, DatabaseError>
    where
        E: sqlx::Executor<'b, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            INSERT INTO categories (category)
            VALUES ($1)
            RETURNING id
            ",
            self.name
        )
        .fetch_one(exec)
        .await?;

        Ok(CategoryId(result.id))
    }
}

pub struct LoaderBuilder<'a> {
    pub name: Option<&'a str>,
}

impl Loader {
    pub fn builder() -> LoaderBuilder<'static> {
        LoaderBuilder { name: None }
    }

    pub async fn get_id<'a, E>(name: &str, exec: E) -> Result<Option<LoaderId>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(DatabaseError::InvalidIdentifier(name.to_string()));
        }

        let result = sqlx::query!(
            "
            SELECT id FROM loaders
            WHERE loader = $1
            ",
            name
        )
        .fetch_optional(exec)
        .await?;

        Ok(result.map(|r| LoaderId(r.id)))
    }

    pub async fn get_name<'a, E>(id: LoaderId, exec: E) -> Result<String, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT loader FROM loaders
            WHERE id = $1
            ",
            id as LoaderId
        )
        .fetch_one(exec)
        .await?;

        Ok(result.loader)
    }

    pub async fn list<'a, E>(exec: E) -> Result<Vec<String>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT loader FROM loaders
            "
        )
        .fetch_many(exec)
        .try_filter_map(|e| async { Ok(e.right().map(|c| c.loader)) })
        .try_collect::<Vec<String>>()
        .await?;

        Ok(result)
    }

    // TODO: remove loaders with mods using them
    pub async fn remove<'a, E>(name: &str, exec: E) -> Result<Option<()>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        use sqlx::Done;

        let result = sqlx::query!(
            "
            DELETE FROM loaders
            WHERE loader = $1
            ",
            name
        )
        .execute(exec)
        .await?;

        if result.rows_affected() == 0 {
            // Nothing was deleted
            Ok(None)
        } else {
            Ok(Some(()))
        }
    }
}

impl<'a> LoaderBuilder<'a> {
    /// The name of the loader.  Must be ASCII alphanumeric or `-`/`_`
    pub fn name(self, name: &'a str) -> Result<LoaderBuilder<'a>, DatabaseError> {
        if name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            Ok(Self { name: Some(name) })
        } else {
            Err(DatabaseError::InvalidIdentifier(name.to_string()))
        }
    }

    pub async fn insert<'b, E>(self, exec: E) -> Result<LoaderId, DatabaseError>
    where
        E: sqlx::Executor<'b, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            INSERT INTO loaders (loader)
            VALUES ($1)
            RETURNING id
            ",
            self.name
        )
        .fetch_one(exec)
        .await?;

        Ok(LoaderId(result.id))
    }
}

pub struct GameVersionBuilder<'a> {
    pub version: Option<&'a str>,
}

impl GameVersion {
    pub fn builder() -> GameVersionBuilder<'static> {
        GameVersionBuilder { version: None }
    }

    pub async fn get_id<'a, E>(
        version: &str,
        exec: E,
    ) -> Result<Option<GameVersionId>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        if !version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
        {
            return Err(DatabaseError::InvalidIdentifier(version.to_string()));
        }

        let result = sqlx::query!(
            "
            SELECT id FROM game_versions
            WHERE version = $1
            ",
            version
        )
        .fetch_optional(exec)
        .await?;

        Ok(result.map(|r| GameVersionId(r.id)))
    }

    pub async fn get_name<'a, E>(id: VersionId, exec: E) -> Result<String, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT version FROM game_versions
            WHERE id = $1
            ",
            id as VersionId
        )
        .fetch_one(exec)
        .await?;

        Ok(result.version)
    }

    pub async fn list<'a, E>(exec: E) -> Result<Vec<String>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT version FROM game_versions
            "
        )
        .fetch_many(exec)
        .try_filter_map(|e| async { Ok(e.right().map(|c| c.version)) })
        .try_collect::<Vec<String>>()
        .await?;

        Ok(result)
    }

    pub async fn remove<'a, E>(name: &str, exec: E) -> Result<Option<()>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        use sqlx::Done;

        let result = sqlx::query!(
            "
            DELETE FROM game_versions
            WHERE version = $1
            ",
            name
        )
        .execute(exec)
        .await?;

        if result.rows_affected() == 0 {
            // Nothing was deleted
            Ok(None)
        } else {
            Ok(Some(()))
        }
    }
}

impl<'a> GameVersionBuilder<'a> {
    /// The game version.  Spaces must be replaced with '_' for it to be valid
    pub fn version(self, version: &'a str) -> Result<GameVersionBuilder<'a>, DatabaseError> {
        if version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
        {
            Ok(Self {
                version: Some(version),
            })
        } else {
            Err(DatabaseError::InvalidIdentifier(version.to_string()))
        }
    }

    pub async fn insert<'b, E>(self, exec: E) -> Result<GameVersionId, DatabaseError>
    where
        E: sqlx::Executor<'b, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            INSERT INTO game_versions (version)
            VALUES ($1)
            RETURNING id
            ",
            self.version
        )
        .fetch_one(exec)
        .await?;

        Ok(GameVersionId(result.id))
    }
}
