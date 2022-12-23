use super::ids::*;
use super::DatabaseError;
use chrono::DateTime;
use chrono::Utc;
use futures::TryStreamExt;
use serde::Deserialize;

pub struct ProjectType {
    pub id: ProjectTypeId,
    pub name: String,
}

pub struct Loader {
    pub id: LoaderId,
    pub loader: String,
    pub icon: String,
    pub supported_project_types: Vec<String>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct GameVersion {
    pub id: GameVersionId,
    pub version: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub created: DateTime<Utc>,
    pub major: bool,
}

pub struct Category {
    pub id: CategoryId,
    pub category: String,
    pub project_type: String,
    pub icon: String,
    pub header: String,
}

pub struct ReportType {
    pub id: ReportTypeId,
    pub report_type: String,
}

pub struct DonationPlatform {
    pub id: DonationPlatformId,
    pub short: String,
    pub name: String,
}

pub struct CategoryBuilder<'a> {
    pub name: Option<&'a str>,
    pub project_type: Option<&'a ProjectTypeId>,
    pub icon: Option<&'a str>,
    pub header: Option<&'a str>,
}

impl Category {
    pub fn builder() -> CategoryBuilder<'static> {
        CategoryBuilder {
            name: None,
            project_type: None,
            icon: None,
            header: None,
        }
    }

    pub async fn get_id<'a, E>(
        name: &str,
        exec: E,
    ) -> Result<Option<CategoryId>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT id FROM categories
            WHERE category = $1
            ",
            name,
        )
        .fetch_optional(exec)
        .await?;

        Ok(result.map(|r| CategoryId(r.id)))
    }

    pub async fn get_id_project<'a, E>(
        name: &str,
        project_type: ProjectTypeId,
        exec: E,
    ) -> Result<Option<CategoryId>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT id FROM categories
            WHERE category = $1 AND project_type = $2
            ",
            name,
            project_type as ProjectTypeId
        )
        .fetch_optional(exec)
        .await?;

        Ok(result.map(|r| CategoryId(r.id)))
    }

    pub async fn get_name<'a, E>(
        id: CategoryId,
        exec: E,
    ) -> Result<String, DatabaseError>
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

    pub async fn list<'a, E>(exec: E) -> Result<Vec<Category>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT c.id id, c.category category, c.icon icon, c.header category_header, pt.name project_type
            FROM categories c
            INNER JOIN project_types pt ON c.project_type = pt.id
            ORDER BY c.ordering, c.category
            "
        )
        .fetch_many(exec)
        .try_filter_map(|e| async {
            Ok(e.right().map(|c| Category {
                id: CategoryId(c.id),
                category: c.category,
                project_type: c.project_type,
                icon: c.icon,
                header: c.category_header
            }))
        })
        .try_collect::<Vec<Category>>()
        .await?;

        Ok(result)
    }

    pub async fn remove<'a, E>(
        name: &str,
        exec: E,
    ) -> Result<Option<()>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
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
    pub fn name(
        self,
        name: &'a str,
    ) -> Result<CategoryBuilder<'a>, DatabaseError> {
        Ok(Self {
            name: Some(name),
            ..self
        })
    }

    pub fn header(
        self,
        header: &'a str,
    ) -> Result<CategoryBuilder<'a>, DatabaseError> {
        Ok(Self {
            header: Some(header),
            ..self
        })
    }

    pub fn project_type(
        self,
        project_type: &'a ProjectTypeId,
    ) -> Result<CategoryBuilder<'a>, DatabaseError> {
        Ok(Self {
            project_type: Some(project_type),
            ..self
        })
    }

    pub fn icon(
        self,
        icon: &'a str,
    ) -> Result<CategoryBuilder<'a>, DatabaseError> {
        Ok(Self {
            icon: Some(icon),
            ..self
        })
    }

    pub async fn insert<'b, E>(
        self,
        exec: E,
    ) -> Result<CategoryId, DatabaseError>
    where
        E: sqlx::Executor<'b, Database = sqlx::Postgres>,
    {
        let id = *self.project_type.ok_or_else(|| {
            DatabaseError::Other("No project type specified.".to_string())
        })?;
        let result = sqlx::query!(
            "
            INSERT INTO categories (category, project_type, icon, header)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            ",
            self.name,
            id as ProjectTypeId,
            self.icon,
            self.header
        )
        .fetch_one(exec)
        .await?;

        Ok(CategoryId(result.id))
    }
}

pub struct LoaderBuilder<'a> {
    pub name: Option<&'a str>,
    pub icon: Option<&'a str>,
    pub supported_project_types: Option<&'a [ProjectTypeId]>,
}

impl Loader {
    pub fn builder() -> LoaderBuilder<'static> {
        LoaderBuilder {
            name: None,
            icon: None,
            supported_project_types: None,
        }
    }

    pub async fn get_id<'a, E>(
        name: &str,
        exec: E,
    ) -> Result<Option<LoaderId>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
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

    pub async fn get_name<'a, E>(
        id: LoaderId,
        exec: E,
    ) -> Result<String, DatabaseError>
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

    pub async fn list<'a, E>(exec: E) -> Result<Vec<Loader>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT l.id id, l.loader loader, l.icon icon,
            ARRAY_AGG(DISTINCT pt.name) filter (where pt.name is not null) project_types
            FROM loaders l
            LEFT OUTER JOIN loaders_project_types lpt ON joining_loader_id = l.id
            LEFT OUTER JOIN project_types pt ON lpt.joining_project_type_id = pt.id
            GROUP BY l.id;
            "
        )
        .fetch_many(exec)
        .try_filter_map(|e| async {
            Ok(e.right().map(|x| Loader {
                id: LoaderId(x.id),
                loader: x.loader,
                icon: x.icon,
                supported_project_types: x
                    .project_types
                    .unwrap_or_default()
                    .iter()
                    .map(|x| x.to_string())
                    .collect(),
            }))
        })
        .try_collect::<Vec<_>>()
        .await?;

        Ok(result)
    }

    // TODO: remove loaders with projects using them
    pub async fn remove<'a, E>(
        name: &str,
        exec: E,
    ) -> Result<Option<()>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
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
    pub fn name(
        self,
        name: &'a str,
    ) -> Result<LoaderBuilder<'a>, DatabaseError> {
        Ok(Self {
            name: Some(name),
            ..self
        })
    }

    pub fn icon(
        self,
        icon: &'a str,
    ) -> Result<LoaderBuilder<'a>, DatabaseError> {
        Ok(Self {
            icon: Some(icon),
            ..self
        })
    }

    pub fn supported_project_types(
        self,
        supported_project_types: &'a [ProjectTypeId],
    ) -> Result<LoaderBuilder<'a>, DatabaseError> {
        Ok(Self {
            supported_project_types: Some(supported_project_types),
            ..self
        })
    }

    pub async fn insert(
        self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<LoaderId, super::DatabaseError> {
        let result = sqlx::query!(
            "
            INSERT INTO loaders (loader, icon)
            VALUES ($1, $2)
            ON CONFLICT (loader) DO NOTHING
            RETURNING id
            ",
            self.name,
            self.icon
        )
        .fetch_one(&mut *transaction)
        .await?;

        if let Some(project_types) = self.supported_project_types {
            sqlx::query!(
                "
                    DELETE FROM loaders_project_types
                    WHERE joining_loader_id = $1
                    ",
                result.id
            )
            .execute(&mut *transaction)
            .await?;

            for project_type in project_types {
                sqlx::query!(
                    "
                    INSERT INTO loaders_project_types (joining_loader_id, joining_project_type_id)
                    VALUES ($1, $2)
                    ",
                    result.id,
                    project_type.0,
                )
                .execute(&mut *transaction)
                .await?;
            }
        }

        Ok(LoaderId(result.id))
    }
}

#[derive(Default)]
pub struct GameVersionBuilder<'a> {
    pub version: Option<&'a str>,
    pub version_type: Option<&'a str>,
    pub date: Option<&'a DateTime<Utc>>,
}

impl GameVersion {
    pub fn builder() -> GameVersionBuilder<'static> {
        GameVersionBuilder::default()
    }

    pub async fn get_id<'a, E>(
        version: &str,
        exec: E,
    ) -> Result<Option<GameVersionId>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
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

    pub async fn get_name<'a, E>(
        id: GameVersionId,
        exec: E,
    ) -> Result<String, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT version FROM game_versions
            WHERE id = $1
            ",
            id as GameVersionId
        )
        .fetch_one(exec)
        .await?;

        Ok(result.version)
    }

    pub async fn list<'a, E>(exec: E) -> Result<Vec<GameVersion>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT gv.id id, gv.version version_, gv.type type_, gv.created created, gv.major FROM game_versions gv
            ORDER BY created DESC
            "
        )
        .fetch_many(exec)
        .try_filter_map(|e| async { Ok(e.right().map(|c| GameVersion {
            id: GameVersionId(c.id),
            version: c.version_,
            type_: c.type_,
            created: c.created,
            major: c.major
        })) })
        .try_collect::<Vec<GameVersion>>()
        .await?;

        Ok(result)
    }

    pub async fn list_filter<'a, E>(
        version_type_option: Option<&str>,
        major_option: Option<bool>,
        exec: E,
    ) -> Result<Vec<GameVersion>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result;

        if let Some(version_type) = version_type_option {
            if let Some(major) = major_option {
                result = sqlx::query!(
                    "
                    SELECT gv.id id, gv.version version_, gv.type type_, gv.created created, gv.major major FROM game_versions gv
                    WHERE major = $1 AND type = $2
                    ORDER BY created DESC
                    ",
                    major,
                    version_type
                )
                .fetch_many(exec)
                    .try_filter_map(|e| async { Ok(e.right().map(|c| GameVersion {
                        id: GameVersionId(c.id),
                        version: c.version_,
                        type_: c.type_,
                        created: c.created,
                        major: c.major,
                    })) })
                .try_collect::<Vec<GameVersion>>()
                .await?;
            } else {
                result = sqlx::query!(
                    "
                    SELECT gv.id id, gv.version version_, gv.type type_, gv.created created, gv.major major FROM game_versions gv
                    WHERE type = $1
                    ORDER BY created DESC
                    ",
                    version_type
                )
                .fetch_many(exec)
                    .try_filter_map(|e| async { Ok(e.right().map(|c| GameVersion {
                        id: GameVersionId(c.id),
                        version: c.version_,
                        type_: c.type_,
                        created: c.created,
                        major: c.major,
                    })) })
                .try_collect::<Vec<GameVersion>>()
                .await?;
            }
        } else if let Some(major) = major_option {
            result = sqlx::query!(
                "
                SELECT gv.id id, gv.version version_, gv.type type_, gv.created created, gv.major major FROM game_versions gv
                WHERE major = $1
                ORDER BY created DESC
                ",
                major
            )
            .fetch_many(exec)
                .try_filter_map(|e| async { Ok(e.right().map(|c| GameVersion {
                    id: GameVersionId(c.id),
                    version: c.version_,
                    type_: c.type_,
                    created: c.created,
                    major: c.major,
                })) })
            .try_collect::<Vec<GameVersion>>()
            .await?;
        } else {
            result = Vec::new();
        }

        Ok(result)
    }

    pub async fn remove<'a, E>(
        name: &str,
        exec: E,
    ) -> Result<Option<()>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
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
    pub fn version(
        self,
        version: &'a str,
    ) -> Result<GameVersionBuilder<'a>, DatabaseError> {
        Ok(Self {
            version: Some(version),
            ..self
        })
    }

    pub fn version_type(
        self,
        version_type: &'a str,
    ) -> Result<GameVersionBuilder<'a>, DatabaseError> {
        Ok(Self {
            version_type: Some(version_type),
            ..self
        })
    }

    pub fn created(self, created: &'a DateTime<Utc>) -> GameVersionBuilder<'a> {
        Self {
            date: Some(created),
            ..self
        }
    }

    pub async fn insert<'b, E>(
        self,
        exec: E,
    ) -> Result<GameVersionId, DatabaseError>
    where
        E: sqlx::Executor<'b, Database = sqlx::Postgres>,
    {
        // This looks like a mess, but it *should* work
        // This allows game versions to be partially updated without
        // replacing the unspecified fields with defaults.
        let result = sqlx::query!(
            "
            INSERT INTO game_versions (version, type, created)
            VALUES ($1, COALESCE($2, 'other'), COALESCE($3, timezone('utc', now())))
            ON CONFLICT (version) DO UPDATE
                SET type = COALESCE($2, game_versions.type),
                    created = COALESCE($3, game_versions.created)
            RETURNING id
            ",
            self.version,
            self.version_type,
            self.date.map(chrono::DateTime::naive_utc),
        )
        .fetch_one(exec)
        .await?;

        Ok(GameVersionId(result.id))
    }
}

#[derive(Default)]
pub struct DonationPlatformBuilder<'a> {
    pub short: Option<&'a str>,
    pub name: Option<&'a str>,
}

impl DonationPlatform {
    pub fn builder() -> DonationPlatformBuilder<'static> {
        DonationPlatformBuilder::default()
    }

    pub async fn get_id<'a, E>(
        id: &str,
        exec: E,
    ) -> Result<Option<DonationPlatformId>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT id FROM donation_platforms
            WHERE short = $1
            ",
            id
        )
        .fetch_optional(exec)
        .await?;

        Ok(result.map(|r| DonationPlatformId(r.id)))
    }

    pub async fn get<'a, E>(
        id: DonationPlatformId,
        exec: E,
    ) -> Result<DonationPlatform, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT short, name FROM donation_platforms
            WHERE id = $1
            ",
            id as DonationPlatformId
        )
        .fetch_one(exec)
        .await?;

        Ok(DonationPlatform {
            id,
            short: result.short,
            name: result.name,
        })
    }

    pub async fn list<'a, E>(
        exec: E,
    ) -> Result<Vec<DonationPlatform>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT id, short, name FROM donation_platforms
            "
        )
        .fetch_many(exec)
        .try_filter_map(|e| async {
            Ok(e.right().map(|c| DonationPlatform {
                id: DonationPlatformId(c.id),
                short: c.short,
                name: c.name,
            }))
        })
        .try_collect::<Vec<DonationPlatform>>()
        .await?;

        Ok(result)
    }

    pub async fn remove<'a, E>(
        short: &str,
        exec: E,
    ) -> Result<Option<()>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            DELETE FROM donation_platforms
            WHERE short = $1
            ",
            short
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

impl<'a> DonationPlatformBuilder<'a> {
    /// The donation platform short name.  Spaces must be replaced with '_' for it to be valid
    pub fn short(
        self,
        short: &'a str,
    ) -> Result<DonationPlatformBuilder<'a>, DatabaseError> {
        Ok(Self {
            short: Some(short),
            ..self
        })
    }

    /// The donation platform long name
    pub fn name(
        self,
        name: &'a str,
    ) -> Result<DonationPlatformBuilder<'a>, DatabaseError> {
        Ok(Self {
            name: Some(name),
            ..self
        })
    }

    pub async fn insert<'b, E>(
        self,
        exec: E,
    ) -> Result<DonationPlatformId, DatabaseError>
    where
        E: sqlx::Executor<'b, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            INSERT INTO donation_platforms (short, name)
            VALUES ($1, $2)
            ON CONFLICT (short) DO NOTHING
            RETURNING id
            ",
            self.short,
            self.name,
        )
        .fetch_one(exec)
        .await?;

        Ok(DonationPlatformId(result.id))
    }
}

pub struct ReportTypeBuilder<'a> {
    pub name: Option<&'a str>,
}

impl ReportType {
    pub fn builder() -> ReportTypeBuilder<'static> {
        ReportTypeBuilder { name: None }
    }

    pub async fn get_id<'a, E>(
        name: &str,
        exec: E,
    ) -> Result<Option<ReportTypeId>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT id FROM report_types
            WHERE name = $1
            ",
            name
        )
        .fetch_optional(exec)
        .await?;

        Ok(result.map(|r| ReportTypeId(r.id)))
    }

    pub async fn get_name<'a, E>(
        id: ReportTypeId,
        exec: E,
    ) -> Result<String, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT name FROM report_types
            WHERE id = $1
            ",
            id as ReportTypeId
        )
        .fetch_one(exec)
        .await?;

        Ok(result.name)
    }

    pub async fn list<'a, E>(exec: E) -> Result<Vec<String>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT name FROM report_types
            "
        )
        .fetch_many(exec)
        .try_filter_map(|e| async { Ok(e.right().map(|c| c.name)) })
        .try_collect::<Vec<String>>()
        .await?;

        Ok(result)
    }

    pub async fn remove<'a, E>(
        name: &str,
        exec: E,
    ) -> Result<Option<()>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            DELETE FROM report_types
            WHERE name = $1
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

impl<'a> ReportTypeBuilder<'a> {
    /// The name of the report type.  Must be ASCII alphanumeric or `-`/`_`
    pub fn name(
        self,
        name: &'a str,
    ) -> Result<ReportTypeBuilder<'a>, DatabaseError> {
        Ok(Self { name: Some(name) })
    }

    pub async fn insert<'b, E>(
        self,
        exec: E,
    ) -> Result<ReportTypeId, DatabaseError>
    where
        E: sqlx::Executor<'b, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            INSERT INTO report_types (name)
            VALUES ($1)
            ON CONFLICT (name) DO NOTHING
            RETURNING id
            ",
            self.name
        )
        .fetch_one(exec)
        .await?;

        Ok(ReportTypeId(result.id))
    }
}

pub struct ProjectTypeBuilder<'a> {
    pub name: Option<&'a str>,
}

impl ProjectType {
    pub fn builder() -> ProjectTypeBuilder<'static> {
        ProjectTypeBuilder { name: None }
    }

    pub async fn get_id<'a, E>(
        name: &str,
        exec: E,
    ) -> Result<Option<ProjectTypeId>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT id FROM project_types
            WHERE name = $1
            ",
            name
        )
        .fetch_optional(exec)
        .await?;

        Ok(result.map(|r| ProjectTypeId(r.id)))
    }

    pub async fn get_many_id<'a, E>(
        names: &[String],
        exec: E,
    ) -> Result<Vec<ProjectType>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let project_types = sqlx::query!(
            "
            SELECT id, name FROM project_types
            WHERE name = ANY($1)
            ",
            names
        )
        .fetch_many(exec)
        .try_filter_map(|e| async {
            Ok(e.right().map(|x| ProjectType {
                id: ProjectTypeId(x.id),
                name: x.name,
            }))
        })
        .try_collect::<Vec<ProjectType>>()
        .await?;

        Ok(project_types)
    }

    pub async fn get_name<'a, E>(
        id: ProjectTypeId,
        exec: E,
    ) -> Result<String, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT name FROM project_types
            WHERE id = $1
            ",
            id as ProjectTypeId
        )
        .fetch_one(exec)
        .await?;

        Ok(result.name)
    }

    pub async fn list<'a, E>(exec: E) -> Result<Vec<String>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT name FROM project_types
            "
        )
        .fetch_many(exec)
        .try_filter_map(|e| async { Ok(e.right().map(|c| c.name)) })
        .try_collect::<Vec<String>>()
        .await?;

        Ok(result)
    }

    // TODO: remove loaders with mods using them
    pub async fn remove<'a, E>(
        name: &str,
        exec: E,
    ) -> Result<Option<()>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            DELETE FROM project_types
            WHERE name = $1
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

impl<'a> ProjectTypeBuilder<'a> {
    /// The name of the project type.  Must be ASCII alphanumeric or `-`/`_`
    pub fn name(
        self,
        name: &'a str,
    ) -> Result<ProjectTypeBuilder<'a>, DatabaseError> {
        Ok(Self { name: Some(name) })
    }

    pub async fn insert<'b, E>(
        self,
        exec: E,
    ) -> Result<ProjectTypeId, DatabaseError>
    where
        E: sqlx::Executor<'b, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            INSERT INTO project_types (name)
            VALUES ($1)
            ON CONFLICT (name) DO NOTHING
            RETURNING id
            ",
            self.name
        )
        .fetch_one(exec)
        .await?;

        Ok(ProjectTypeId(result.id))
    }
}
