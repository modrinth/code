use super::ids::*;
use super::DatabaseError;
use crate::models::ids::base62_impl::parse_base62;
use crate::models::projects::{FileType, VersionStatus, VersionType};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::cmp::Ordering;
use std::collections::HashMap;

pub struct VersionBuilder {
    pub version_id: VersionId,
    pub project_id: ProjectId,
    pub author_id: UserId,
    pub name: String,
    pub version_number: String,
    pub changelog: String,
    pub files: Vec<VersionFileBuilder>,
    pub dependencies: Vec<DependencyBuilder>,
    pub game_versions: Vec<GameVersionId>,
    pub loaders: Vec<LoaderId>,
    pub version_type: String,
    pub featured: bool,
    pub status: VersionStatus,
    pub requested_status: Option<VersionStatus>,
}

pub struct DependencyBuilder {
    pub project_id: Option<ProjectId>,
    pub version_id: Option<VersionId>,
    pub file_name: Option<String>,
    pub dependency_type: String,
}

impl DependencyBuilder {
    pub async fn insert(
        self,
        version_id: VersionId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        let (version_dependency_id, project_dependency_id): (
            Option<VersionId>,
            Option<ProjectId>,
        ) = if self.version_id.is_some() {
            (self.version_id, None)
        } else if let Some(project_id) = self.project_id {
            let version_id = sqlx::query!(
                    "
                    SELECT version.id id FROM (
                        SELECT DISTINCT ON(v.id) v.id, v.date_published FROM versions v
                        INNER JOIN game_versions_versions gvv ON gvv.joining_version_id = v.id AND gvv.game_version_id IN (SELECT game_version_id FROM game_versions_versions WHERE joining_version_id = $2)
                        INNER JOIN loaders_versions lv ON lv.version_id = v.id AND lv.loader_id IN (SELECT loader_id FROM loaders_versions WHERE version_id = $2)
                        WHERE v.mod_id = $1
                    ) AS version
                    ORDER BY version.date_published DESC
                    LIMIT 1
                    ",
                    project_id as ProjectId,
                    version_id as VersionId,
                )
                .fetch_optional(&mut *transaction).await?.map(|x| VersionId(x.id));

            (version_id, Some(project_id))
        } else {
            (None, None)
        };

        sqlx::query!(
            "
            INSERT INTO dependencies (dependent_id, dependency_type, dependency_id, mod_dependency_id, dependency_file_name)
            VALUES ($1, $2, $3, $4, $5)
            ",
            version_id as VersionId,
            self.dependency_type,
            version_dependency_id.map(|x| x.0),
            project_dependency_id.map(|x| x.0),
            self.file_name,
        )
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }
}

pub struct VersionFileBuilder {
    pub url: String,
    pub filename: String,
    pub hashes: Vec<HashBuilder>,
    pub primary: bool,
    pub size: u32,
    pub file_type: Option<FileType>,
}

impl VersionFileBuilder {
    pub async fn insert(
        self,
        version_id: VersionId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<FileId, DatabaseError> {
        let file_id = generate_file_id(&mut *transaction).await?;

        sqlx::query!(
            "
            INSERT INTO files (id, version_id, url, filename, is_primary, size, file_type)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ",
            file_id as FileId,
            version_id as VersionId,
            self.url,
            self.filename,
            self.primary,
            self.size as i32,
            self.file_type.map(|x| x.as_str()),
        )
        .execute(&mut *transaction)
        .await?;

        for hash in self.hashes {
            sqlx::query!(
                "
                INSERT INTO hashes (file_id, algorithm, hash)
                VALUES ($1, $2, $3)
                ",
                file_id as FileId,
                hash.algorithm,
                hash.hash,
            )
            .execute(&mut *transaction)
            .await?;
        }

        Ok(file_id)
    }
}

pub struct HashBuilder {
    pub algorithm: String,
    pub hash: Vec<u8>,
}

impl VersionBuilder {
    pub async fn insert(
        self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<VersionId, DatabaseError> {
        let version = Version {
            id: self.version_id,
            project_id: self.project_id,
            author_id: self.author_id,
            name: self.name,
            version_number: self.version_number,
            changelog: self.changelog,
            changelog_url: None,
            date_published: Utc::now(),
            downloads: 0,
            featured: self.featured,
            version_type: self.version_type,
            status: self.status,
            requested_status: self.requested_status,
        };

        version.insert(&mut *transaction).await?;

        sqlx::query!(
            "
            UPDATE mods
            SET updated = NOW()
            WHERE id = $1
            ",
            self.project_id as ProjectId,
        )
        .execute(&mut *transaction)
        .await?;

        for file in self.files {
            file.insert(self.version_id, transaction).await?;
        }

        for dependency in self.dependencies {
            dependency.insert(self.version_id, transaction).await?;
        }

        for loader in self.loaders.clone() {
            sqlx::query!(
                "
                INSERT INTO loaders_versions (loader_id, version_id)
                VALUES ($1, $2)
                ",
                loader as LoaderId,
                self.version_id as VersionId,
            )
            .execute(&mut *transaction)
            .await?;
        }

        for game_version in self.game_versions.clone() {
            sqlx::query!(
                "
                INSERT INTO game_versions_versions (game_version_id, joining_version_id)
                VALUES ($1, $2)
                ",
                game_version as GameVersionId,
                self.version_id as VersionId,
            )
            .execute(&mut *transaction)
            .await?;
        }

        // Sync dependencies

        use futures::stream::TryStreamExt;

        let dependencies = sqlx::query!(
            "
            SELECT d.id id
            FROM dependencies d
            INNER JOIN game_versions_versions gvv ON gvv.joining_version_id = d.dependent_id AND gvv.game_version_id = ANY($2)
            INNER JOIN loaders_versions lv ON lv.version_id = d.dependent_id AND lv.loader_id = ANY($3)
            WHERE d.mod_dependency_id = $1
            ",
            self.project_id as ProjectId,
            &self.game_versions.iter().map(|x| x.0).collect::<Vec<i32>>(),
            &self.loaders.iter().map(|x| x.0).collect::<Vec<i32>>(),
        )
            .fetch_many(&mut *transaction)
            .try_filter_map(|e| async {
                Ok(e.right().map(|d| d.id as i64))
            })
            .try_collect::<Vec<i64>>()
            .await?;

        sqlx::query!(
            "
            UPDATE dependencies
            SET dependency_id = $2
            WHERE id = ANY($1::bigint[])
            ",
            dependencies.as_slice(),
            self.version_id as VersionId,
        )
        .execute(&mut *transaction)
        .await?;

        Ok(self.version_id)
    }
}

#[derive(Clone)]
pub struct Version {
    pub id: VersionId,
    pub project_id: ProjectId,
    pub author_id: UserId,
    pub name: String,
    pub version_number: String,
    pub changelog: String,
    pub changelog_url: Option<String>,
    pub date_published: DateTime<Utc>,
    pub downloads: i32,
    pub version_type: String,
    pub featured: bool,
    pub status: VersionStatus,
    pub requested_status: Option<VersionStatus>,
}

impl Version {
    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::error::Error> {
        sqlx::query!(
            "
            INSERT INTO versions (
                id, mod_id, author_id, name, version_number,
                changelog, date_published, downloads,
                version_type, featured, status
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8,
                $9, $10, $11
            )
            ",
            self.id as VersionId,
            self.project_id as ProjectId,
            self.author_id as UserId,
            &self.name,
            &self.version_number,
            self.changelog,
            self.date_published,
            self.downloads,
            &self.version_type,
            self.featured,
            self.status.as_str()
        )
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }

    pub async fn remove_full(
        id: VersionId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Option<()>, sqlx::Error> {
        let result = sqlx::query!(
            "
            SELECT EXISTS(SELECT 1 FROM versions WHERE id = $1)
            ",
            id as VersionId,
        )
        .fetch_one(&mut *transaction)
        .await?;

        if !result.exists.unwrap_or(false) {
            return Ok(None);
        }

        sqlx::query!(
            "
            DELETE FROM reports
            WHERE version_id = $1
            ",
            id as VersionId,
        )
        .execute(&mut *transaction)
        .await?;

        use futures::TryStreamExt;

        let game_versions: Vec<i32> = sqlx::query!(
            "
                SELECT game_version_id id FROM game_versions_versions
                WHERE joining_version_id = $1
                ",
            id as VersionId,
        )
        .fetch_many(&mut *transaction)
        .try_filter_map(|e| async { Ok(e.right().map(|c| c.id)) })
        .try_collect::<Vec<i32>>()
        .await?;

        let loaders: Vec<i32> = sqlx::query!(
            "
                SELECT loader_id id FROM loaders_versions
                WHERE version_id = $1
                ",
            id as VersionId,
        )
        .fetch_many(&mut *transaction)
        .try_filter_map(|e| async { Ok(e.right().map(|c| c.id)) })
        .try_collect::<Vec<i32>>()
        .await?;

        sqlx::query!(
            "
            DELETE FROM game_versions_versions gvv
            WHERE gvv.joining_version_id = $1
            ",
            id as VersionId,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM loaders_versions
            WHERE loaders_versions.version_id = $1
            ",
            id as VersionId,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM hashes
            WHERE EXISTS(
                SELECT 1 FROM files WHERE
                    (files.version_id = $1) AND
                    (hashes.file_id = files.id)
            )
            ",
            id as VersionId
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM files
            WHERE files.version_id = $1
            ",
            id as VersionId,
        )
        .execute(&mut *transaction)
        .await?;

        // Sync dependencies

        let project_id = sqlx::query!(
            "
            SELECT mod_id FROM versions WHERE id = $1
            ",
            id as VersionId,
        )
        .fetch_one(&mut *transaction)
        .await?;

        let new_version_id = sqlx::query!(
            "
            SELECT v.id id
            FROM versions v
            INNER JOIN game_versions_versions gvv ON gvv.joining_version_id = v.id AND gvv.game_version_id = ANY($2)
            INNER JOIN loaders_versions lv ON lv.version_id = v.id AND lv.loader_id = ANY($3)
            WHERE v.mod_id = $1
            ORDER BY v.date_published DESC
            LIMIT 1
            ",
            project_id.mod_id,
            &game_versions,
            &loaders,
        )
            .fetch_optional(&mut *transaction)
            .await?
            .map(|x| x.id);

        sqlx::query!(
            "
            UPDATE dependencies
            SET dependency_id = $2
            WHERE dependency_id = $1
            ",
            id as VersionId,
            new_version_id,
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM dependencies WHERE mod_dependency_id = NULL AND dependency_id = NULL AND dependency_file_name = NULL
            ",
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM dependencies WHERE dependent_id = $1
            ",
            id as VersionId,
        )
        .execute(&mut *transaction)
        .await?;

        // delete version

        sqlx::query!(
            "
            DELETE FROM versions WHERE id = $1
            ",
            id as VersionId,
        )
        .execute(&mut *transaction)
        .await?;

        crate::database::models::Project::update_game_versions(
            ProjectId(project_id.mod_id),
            &mut *transaction,
        )
        .await?;
        crate::database::models::Project::update_loaders(
            ProjectId(project_id.mod_id),
            &mut *transaction,
        )
        .await?;

        Ok(Some(()))
    }

    pub async fn get_project_versions<'a, E>(
        project_id: ProjectId,
        game_versions: Option<Vec<String>>,
        loaders: Option<Vec<String>>,
        version_type: Option<VersionType>,
        limit: Option<u32>,
        offset: Option<u32>,
        exec: E,
    ) -> Result<Vec<VersionId>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        use futures::stream::TryStreamExt;

        let vec = sqlx::query!(
            "
            SELECT DISTINCT ON(v.date_published, v.id) version_id, v.date_published FROM versions v
            INNER JOIN game_versions_versions gvv ON gvv.joining_version_id = v.id
            INNER JOIN game_versions gv on gvv.game_version_id = gv.id AND (cardinality($2::varchar[]) = 0 OR gv.version = ANY($2::varchar[]))
            INNER JOIN loaders_versions lv ON lv.version_id = v.id
            INNER JOIN loaders l on lv.loader_id = l.id AND (cardinality($3::varchar[]) = 0 OR l.loader = ANY($3::varchar[]))
            WHERE v.mod_id = $1 AND ($4::varchar IS NULL OR v.version_type = $4)
            ORDER BY v.date_published DESC, v.id
            LIMIT $5 OFFSET $6
            ",
            project_id as ProjectId,
            &game_versions.unwrap_or_default(),
            &loaders.unwrap_or_default(),
            version_type.map(|x| x.as_str()),
            limit.map(|x| x as i64),
            offset.map(|x| x as i64),
        )
        .fetch_many(exec)
        .try_filter_map(|e| async { Ok(e.right().map(|v| VersionId(v.version_id))) })
        .try_collect::<Vec<VersionId>>()
        .await?;

        Ok(vec)
    }

    pub async fn get_projects_versions<'a, E>(
        project_ids: Vec<ProjectId>,
        game_versions: Option<Vec<String>>,
        loaders: Option<Vec<String>>,
        version_type: Option<VersionType>,
        limit: Option<u32>,
        offset: Option<u32>,
        exec: E,
    ) -> Result<HashMap<ProjectId, Vec<VersionId>>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        use futures::stream::TryStreamExt;

        let vec = sqlx::query!(
            "
            SELECT DISTINCT ON(v.date_published, v.id) version_id, v.mod_id, v.date_published FROM versions v
            INNER JOIN game_versions_versions gvv ON gvv.joining_version_id = v.id
            INNER JOIN game_versions gv on gvv.game_version_id = gv.id AND (cardinality($2::varchar[]) = 0 OR gv.version = ANY($2::varchar[]))
            INNER JOIN loaders_versions lv ON lv.version_id = v.id
            INNER JOIN loaders l on lv.loader_id = l.id AND (cardinality($3::varchar[]) = 0 OR l.loader = ANY($3::varchar[]))
            WHERE v.mod_id = ANY($1) AND ($4::varchar IS NULL OR v.version_type = $4)
            ORDER BY v.date_published, v.id ASC
            LIMIT $5 OFFSET $6
            ",
            &project_ids.into_iter().map(|x| x.0).collect::<Vec<i64>>(),
            &game_versions.unwrap_or_default(),
            &loaders.unwrap_or_default(),
            version_type.map(|x| x.as_str()),
            limit.map(|x| x as i64),
            offset.map(|x| x as i64),
        )
            .fetch_many(exec)
            .try_filter_map(|e| async { Ok(e.right().map(|v| (ProjectId(v.mod_id), VersionId(v.version_id)))) })
            .try_collect::<Vec<(ProjectId, VersionId)>>()
            .await?;

        let mut map: HashMap<ProjectId, Vec<VersionId>> = HashMap::new();

        for (project_id, version_id) in vec {
            if let Some(value) = map.get_mut(&project_id) {
                value.push(version_id);
            } else {
                map.insert(project_id, vec![version_id]);
            }
        }

        Ok(map)
    }

    pub async fn get<'a, 'b, E>(
        id: VersionId,
        executor: E,
    ) -> Result<Option<Self>, sqlx::error::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT v.mod_id, v.author_id, v.name, v.version_number,
                v.changelog, v.date_published, v.downloads,
                v.version_type, v.featured, v.status, v.requested_status
            FROM versions v
            WHERE v.id = $1
            ",
            id as VersionId,
        )
        .fetch_optional(executor)
        .await?;

        if let Some(row) = result {
            Ok(Some(Version {
                id,
                project_id: ProjectId(row.mod_id),
                author_id: UserId(row.author_id),
                name: row.name,
                version_number: row.version_number,
                changelog: row.changelog,
                changelog_url: None,
                date_published: row.date_published,
                downloads: row.downloads,
                version_type: row.version_type,
                featured: row.featured,
                status: VersionStatus::from_str(&row.status),
                requested_status: row
                    .requested_status
                    .map(|x| VersionStatus::from_str(&x)),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_many<'a, E>(
        version_ids: Vec<VersionId>,
        exec: E,
    ) -> Result<Vec<Version>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        use futures::stream::TryStreamExt;

        let version_ids_parsed: Vec<i64> =
            version_ids.into_iter().map(|x| x.0).collect();
        let versions = sqlx::query!(
            "
            SELECT v.id, v.mod_id, v.author_id, v.name, v.version_number,
                v.changelog, v.date_published, v.downloads,
                v.version_type, v.featured, v.status, v.requested_status
            FROM versions v
            WHERE v.id = ANY($1)
            ORDER BY v.date_published ASC
            ",
            &version_ids_parsed
        )
        .fetch_many(exec)
        .try_filter_map(|e| async {
            Ok(e.right().map(|v| Version {
                id: VersionId(v.id),
                project_id: ProjectId(v.mod_id),
                author_id: UserId(v.author_id),
                name: v.name,
                version_number: v.version_number,
                changelog: v.changelog,
                changelog_url: None,
                date_published: v.date_published,
                downloads: v.downloads,
                featured: v.featured,
                version_type: v.version_type,
                status: VersionStatus::from_str(&v.status),
                requested_status: v
                    .requested_status
                    .map(|x| VersionStatus::from_str(&x)),
            }))
        })
        .try_collect::<Vec<Version>>()
        .await?;

        Ok(versions)
    }

    pub async fn get_full<'a, 'b, E>(
        id: VersionId,
        executor: E,
    ) -> Result<Option<QueryVersion>, sqlx::error::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let result = sqlx::query!(
            "
            SELECT v.id id, v.mod_id mod_id, v.author_id author_id, v.name version_name, v.version_number version_number,
            v.changelog changelog, v.date_published date_published, v.downloads downloads,
            v.version_type version_type, v.featured featured, v.status status, v.requested_status requested_status,
            JSONB_AGG(DISTINCT jsonb_build_object('version', gv.version, 'created', gv.created)) filter (where gv.version is not null) game_versions,
            ARRAY_AGG(DISTINCT l.loader) filter (where l.loader is not null) loaders,
            JSONB_AGG(DISTINCT jsonb_build_object('id', f.id, 'url', f.url, 'filename', f.filename, 'primary', f.is_primary, 'size', f.size, 'file_type', f.file_type))  filter (where f.id is not null) files,
            JSONB_AGG(DISTINCT jsonb_build_object('algorithm', h.algorithm, 'hash', encode(h.hash, 'escape'), 'file_id', h.file_id)) filter (where h.hash is not null) hashes,
            JSONB_AGG(DISTINCT jsonb_build_object('project_id', d.mod_dependency_id, 'version_id', d.dependency_id, 'dependency_type', d.dependency_type,'file_name', dependency_file_name)) filter (where d.dependency_type is not null) dependencies
            FROM versions v
            LEFT OUTER JOIN game_versions_versions gvv on v.id = gvv.joining_version_id
            LEFT OUTER JOIN game_versions gv on gvv.game_version_id = gv.id
            LEFT OUTER JOIN loaders_versions lv on v.id = lv.version_id
            LEFT OUTER JOIN loaders l on lv.loader_id = l.id
            LEFT OUTER JOIN files f on v.id = f.version_id
            LEFT OUTER JOIN hashes h on f.id = h.file_id
            LEFT OUTER JOIN dependencies d on v.id = d.dependent_id
            WHERE v.id = $1
            GROUP BY v.id;
            ",
            id as VersionId,
        )
            .fetch_optional(executor)
            .await?;

        if let Some(v) = result {
            Ok(Some(QueryVersion {
                inner: Version {
                    id: VersionId(v.id),
                    project_id: ProjectId(v.mod_id),
                    author_id: UserId(v.author_id),
                    name: v.version_name,
                    version_number: v.version_number,
                    changelog: v.changelog,
                    changelog_url: None,
                    date_published: v.date_published,
                    downloads: v.downloads,
                    version_type: v.version_type,
                    featured: v.featured,
                    status: VersionStatus::from_str(&v.status),
                    requested_status: v
                        .requested_status
                        .map(|x| VersionStatus::from_str(&x)),
                },
                files: {
                    #[derive(Deserialize, Debug)]
                    struct Hash {
                        pub file_id: FileId,
                        pub algorithm: String,
                        pub hash: String,
                    }

                    #[derive(Deserialize, Debug)]
                    struct File {
                        pub id: FileId,
                        pub url: String,
                        pub filename: String,
                        pub primary: bool,
                        pub size: u32,
                        pub file_type: Option<FileType>,
                    }

                    let hashes: Vec<Hash> =
                        serde_json::from_value(v.hashes.unwrap_or_default())
                            .ok()
                            .unwrap_or_default();

                    let files: Vec<File> =
                        serde_json::from_value(v.files.unwrap_or_default())
                            .ok()
                            .unwrap_or_default();

                    let mut files = files
                        .into_iter()
                        .map(|x| {
                            let mut file_hashes = HashMap::new();

                            for hash in &hashes {
                                if hash.file_id == x.id {
                                    file_hashes.insert(
                                        hash.algorithm.clone(),
                                        hash.hash.clone(),
                                    );
                                }
                            }

                            QueryFile {
                                id: x.id,
                                url: x.url,
                                filename: x.filename,
                                hashes: file_hashes,
                                primary: x.primary,
                                size: x.size,
                                file_type: x.file_type,
                            }
                        })
                        .collect::<Vec<_>>();

                    files.sort_by(|a, b| {
                        if a.primary {
                            Ordering::Less
                        } else if b.primary {
                            Ordering::Greater
                        } else {
                            a.filename.cmp(&b.filename)
                        }
                    });

                    files
                },
                game_versions: {
                    #[derive(Deserialize)]
                    struct GameVersion {
                        pub version: String,
                        pub created: DateTime<Utc>,
                    }

                    let mut game_versions: Vec<GameVersion> =
                        serde_json::from_value(
                            v.game_versions.unwrap_or_default(),
                        )
                        .ok()
                        .unwrap_or_default();

                    game_versions.sort_by(|a, b| a.created.cmp(&b.created));

                    game_versions.into_iter().map(|x| x.version).collect()
                },
                loaders: v.loaders.unwrap_or_default(),
                dependencies: serde_json::from_value(
                    v.dependencies.unwrap_or_default(),
                )
                .ok()
                .unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_many_full<'a, E>(
        version_ids: Vec<VersionId>,
        exec: E,
    ) -> Result<Vec<QueryVersion>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        use futures::stream::TryStreamExt;

        let version_ids_parsed: Vec<i64> =
            version_ids.into_iter().map(|x| x.0).collect();
        sqlx::query!(
            "
            SELECT v.id id, v.mod_id mod_id, v.author_id author_id, v.name version_name, v.version_number version_number,
            v.changelog changelog, v.date_published date_published, v.downloads downloads,
            v.version_type version_type, v.featured featured, v.status status, v.requested_status requested_status,
            JSONB_AGG(DISTINCT jsonb_build_object('version', gv.version, 'created', gv.created)) filter (where gv.version is not null) game_versions,
            ARRAY_AGG(DISTINCT l.loader) filter (where l.loader is not null) loaders,
            JSONB_AGG(DISTINCT jsonb_build_object('id', f.id, 'url', f.url, 'filename', f.filename, 'primary', f.is_primary, 'size', f.size, 'file_type', f.file_type))  filter (where f.id is not null) files,
            JSONB_AGG(DISTINCT jsonb_build_object('algorithm', h.algorithm, 'hash', encode(h.hash, 'escape'), 'file_id', h.file_id)) filter (where h.hash is not null) hashes,
            JSONB_AGG(DISTINCT jsonb_build_object('project_id', d.mod_dependency_id, 'version_id', d.dependency_id, 'dependency_type', d.dependency_type,'file_name', dependency_file_name)) filter (where d.dependency_type is not null) dependencies
            FROM versions v
            LEFT OUTER JOIN game_versions_versions gvv on v.id = gvv.joining_version_id
            LEFT OUTER JOIN game_versions gv on gvv.game_version_id = gv.id
            LEFT OUTER JOIN loaders_versions lv on v.id = lv.version_id
            LEFT OUTER JOIN loaders l on lv.loader_id = l.id
            LEFT OUTER JOIN files f on v.id = f.version_id
            LEFT OUTER JOIN hashes h on f.id = h.file_id
            LEFT OUTER JOIN dependencies d on v.id = d.dependent_id
            WHERE v.id = ANY($1)
            GROUP BY v.id
            ORDER BY v.date_published ASC;
            ",
            &version_ids_parsed
        )
            .fetch_many(exec)
            .try_filter_map(|e| async {
                Ok(e.right().map(|v|
                    QueryVersion {
                        inner: Version {
                            id: VersionId(v.id),
                            project_id: ProjectId(v.mod_id),
                            author_id: UserId(v.author_id),
                            name: v.version_name,
                            version_number: v.version_number,
                            changelog: v.changelog,
                            changelog_url: None,
                            date_published: v.date_published,
                            downloads: v.downloads,
                            version_type: v.version_type,
                            featured: v.featured,
                            status: VersionStatus::from_str(&v.status),
                            requested_status: v.requested_status
                                .map(|x| VersionStatus::from_str(&x)),
                        },
                        files: {
                            #[derive(Deserialize)]
                            struct Hash {
                                pub file_id: FileId,
                                pub algorithm: String,
                                pub hash: String,
                            }

                            #[derive(Deserialize)]
                            struct File {
                                pub id: FileId,
                                pub url: String,
                                pub filename: String,
                                pub primary: bool,
                                pub size: u32,
                                pub file_type: Option<FileType>,
                            }

                            let hashes: Vec<Hash> = serde_json::from_value(
                                v.hashes.unwrap_or_default(),
                            )
                                .ok()
                                .unwrap_or_default();

                            let files: Vec<File> = serde_json::from_value(
                                v.files.unwrap_or_default(),
                            )
                                .ok()
                                .unwrap_or_default();

                            let mut files = files.into_iter().map(|x| {
                                let mut file_hashes = HashMap::new();

                                for hash in &hashes {
                                    if hash.file_id == x.id {
                                        file_hashes.insert(
                                            hash.algorithm.clone(),
                                            hash.hash.clone(),
                                        );
                                    }
                                }

                                QueryFile {
                                    id: x.id,
                                    url: x.url,
                                    filename: x.filename,
                                    hashes: file_hashes,
                                    primary: x.primary,
                                    size: x.size,
                                    file_type: x.file_type,
                                }
                            }).collect::<Vec<_>>();

                            files.sort_by(|a, b| {
                                if a.primary {
                                    Ordering::Less
                                } else if b.primary {
                                    Ordering::Greater
                                } else {
                                    a.filename.cmp(&b.filename)
                                }
                            });

                            files
                        },
                        game_versions: {
                            #[derive(Deserialize)]
                            struct GameVersion {
                                pub version: String,
                                pub created: DateTime<Utc>,
                            }

                            let mut game_versions: Vec<GameVersion> = serde_json::from_value(
                                v.game_versions.unwrap_or_default(),
                            )
                                .ok()
                                .unwrap_or_default();

                            game_versions.sort_by(|a, b| a.created.cmp(&b.created));

                            game_versions.into_iter().map(|x| x.version).collect()
                        },
                        loaders: v.loaders.unwrap_or_default(),
                        dependencies: serde_json::from_value(
                            v.dependencies.unwrap_or_default(),
                        )
                            .ok()
                            .unwrap_or_default(),
                    }
                ))
            })
            .try_collect::<Vec<QueryVersion>>()
            .await
    }

    pub async fn get_full_from_id_slug<'a, 'b, E>(
        project_id_or_slug: &str,
        slug: &str,
        executor: E,
    ) -> Result<Option<QueryVersion>, sqlx::error::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let project_id_opt =
            parse_base62(project_id_or_slug).ok().map(|x| x as i64);
        let id_opt = parse_base62(slug).ok().map(|x| x as i64);
        let id = sqlx::query!(
            "
            SELECT v.id FROM versions v
            INNER JOIN mods m ON mod_id = m.id
            WHERE (m.id = $1 OR m.slug = $2) AND (v.id = $3 OR v.version_number = $4)
            ORDER BY date_published ASC
            ",
            project_id_opt,
            project_id_or_slug,
            id_opt,
            slug
        )
        .fetch_optional(executor)
        .await?;

        if let Some(version_id) = id {
            Version::get_full(VersionId(version_id.id), executor).await
        } else {
            Ok(None)
        }
    }
}

#[derive(Clone)]
pub struct QueryVersion {
    pub inner: Version,

    pub files: Vec<QueryFile>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub dependencies: Vec<QueryDependency>,
}

#[derive(Clone, Deserialize)]
pub struct QueryDependency {
    pub project_id: Option<ProjectId>,
    pub version_id: Option<VersionId>,
    pub file_name: Option<String>,
    pub dependency_type: String,
}

#[derive(Clone)]
pub struct QueryFile {
    pub id: FileId,
    pub url: String,
    pub filename: String,
    pub hashes: HashMap<String, String>,
    pub primary: bool,
    pub size: u32,
    pub file_type: Option<FileType>,
}
