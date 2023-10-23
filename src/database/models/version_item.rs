use super::ids::*;
use super::DatabaseError;
use crate::database::redis::RedisPool;
use crate::models::projects::{FileType, VersionStatus};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::iter;

const VERSIONS_NAMESPACE: &str = "versions";
const VERSION_FILES_NAMESPACE: &str = "versions_files";

#[derive(Clone)]
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

#[derive(Clone)]
pub struct DependencyBuilder {
    pub project_id: Option<ProjectId>,
    pub version_id: Option<VersionId>,
    pub file_name: Option<String>,
    pub dependency_type: String,
}

impl DependencyBuilder {
    pub async fn insert_many(
        builders: Vec<Self>,
        version_id: VersionId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        let mut project_ids = Vec::new();
        for dependency in builders.iter() {
            project_ids.push(
                dependency
                    .try_get_project_id(transaction)
                    .await?
                    .map(|id| id.0),
            );
        }

        let (version_ids, dependency_types, dependency_ids, filenames): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = builders
            .into_iter()
            .map(|d| {
                (
                    version_id.0,
                    d.dependency_type,
                    d.version_id.map(|v| v.0),
                    d.file_name,
                )
            })
            .multiunzip();
        sqlx::query!(
            "
            INSERT INTO dependencies (dependent_id, dependency_type, dependency_id, mod_dependency_id, dependency_file_name)
            SELECT * FROM UNNEST ($1::bigint[], $2::varchar[], $3::bigint[], $4::bigint[], $5::varchar[])
            ",
            &version_ids[..],
            &dependency_types[..],
            &dependency_ids[..] as &[Option<i64>],
            &project_ids[..] as &[Option<i64>],
            &filenames[..] as &[Option<String>],
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    async fn try_get_project_id(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Option<ProjectId>, DatabaseError> {
        Ok(if let Some(project_id) = self.project_id {
            Some(project_id)
        } else if let Some(version_id) = self.version_id {
            sqlx::query!(
                "
                SELECT mod_id FROM versions WHERE id = $1
                ",
                version_id as VersionId,
            )
            .fetch_optional(&mut **transaction)
            .await?
            .map(|x| ProjectId(x.mod_id))
        } else {
            None
        })
    }
}

#[derive(Clone, Debug)]
pub struct VersionFileBuilder {
    pub url: String,
    pub filename: String,
    pub hashes: Vec<HashBuilder>,
    pub primary: bool,
    pub size: u32,
    pub file_type: Option<FileType>,
}

impl VersionFileBuilder {
    pub async fn insert_many(
        version_files: Vec<Self>,
        version_id: VersionId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<FileId, DatabaseError> {
        let file_id = generate_file_id(transaction).await?;

        let (file_ids, version_ids, urls, filenames, primary, sizes, file_types): (
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
            Vec<_>,
        ) = version_files
            .iter()
            .map(|f| {
                (
                    file_id.0,
                    version_id.0,
                    f.url.clone(),
                    f.filename.clone(),
                    f.primary,
                    f.size as i32,
                    f.file_type.map(|x| x.to_string()),
                )
            })
            .multiunzip();
        sqlx::query!(
            "
            INSERT INTO files (id, version_id, url, filename, is_primary, size, file_type)
            SELECT * FROM UNNEST($1::bigint[], $2::bigint[], $3::varchar[], $4::varchar[], $5::bool[], $6::integer[], $7::varchar[])
            ",
            &file_ids[..],
            &version_ids[..],
            &urls[..],
            &filenames[..],
            &primary[..],
            &sizes[..],
            &file_types[..] as &[Option<String>],
        )
        .execute(&mut **transaction)
        .await?;

        let (file_ids, algorithms, hashes): (Vec<_>, Vec<_>, Vec<_>) = version_files
            .into_iter()
            .flat_map(|f| {
                f.hashes
                    .into_iter()
                    .map(|h| (file_id.0, h.algorithm, h.hash))
            })
            .multiunzip();
        sqlx::query!(
            "
            INSERT INTO hashes (file_id, algorithm, hash)
            SELECT * FROM UNNEST($1::bigint[], $2::varchar[], $3::bytea[])
            ",
            &file_ids[..],
            &algorithms[..],
            &hashes[..],
        )
        .execute(&mut **transaction)
        .await?;

        Ok(file_id)
    }
}

#[derive(Clone, Debug)]
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

        version.insert(transaction).await?;

        sqlx::query!(
            "
            UPDATE mods
            SET updated = NOW()
            WHERE id = $1
            ",
            self.project_id as ProjectId,
        )
        .execute(&mut **transaction)
        .await?;

        let VersionBuilder {
            dependencies,
            loaders,
            game_versions,
            files,
            version_id,
            ..
        } = self;
        VersionFileBuilder::insert_many(files, self.version_id, transaction).await?;

        DependencyBuilder::insert_many(dependencies, self.version_id, transaction).await?;

        let loader_versions = loaders
            .iter()
            .map(|l| LoaderVersion::new(*l, version_id))
            .collect_vec();
        LoaderVersion::insert_many(loader_versions, transaction).await?;

        let game_version_versions = game_versions
            .iter()
            .map(|v| VersionVersion::new(*v, version_id))
            .collect_vec();
        VersionVersion::insert_many(game_version_versions, transaction).await?;

        Ok(self.version_id)
    }
}

#[derive(derive_new::new)]
pub struct LoaderVersion {
    pub loader_id: LoaderId,
    pub version_id: VersionId,
}

impl LoaderVersion {
    pub async fn insert_many(
        items: Vec<Self>,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        let (loader_ids, version_ids): (Vec<_>, Vec<_>) = items
            .iter()
            .map(|l| (l.loader_id.0, l.version_id.0))
            .unzip();
        sqlx::query!(
            "
            INSERT INTO loaders_versions (loader_id, version_id)
            SELECT * FROM UNNEST($1::integer[], $2::bigint[])
            ",
            &loader_ids[..],
            &version_ids[..],
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }
}

#[derive(derive_new::new)]
pub struct VersionVersion {
    pub game_version_id: GameVersionId,
    pub joining_version_id: VersionId,
}

impl VersionVersion {
    pub async fn insert_many(
        items: Vec<Self>,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), DatabaseError> {
        let (game_version_ids, version_ids): (Vec<_>, Vec<_>) = items
            .into_iter()
            .map(|i| (i.game_version_id.0, i.joining_version_id.0))
            .unzip();
        sqlx::query!(
            "
            INSERT INTO game_versions_versions (game_version_id, joining_version_id)
            SELECT * FROM UNNEST($1::integer[], $2::bigint[])
            ",
            &game_version_ids[..],
            &version_ids[..],
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }
}

#[derive(Clone, Deserialize, Serialize)]
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
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn remove_full(
        id: VersionId,
        redis: &RedisPool,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Option<()>, DatabaseError> {
        let result = Self::get(id, &mut **transaction, redis).await?;

        let result = if let Some(result) = result {
            result
        } else {
            return Ok(None);
        };

        Version::clear_cache(&result, redis).await?;

        sqlx::query!(
            "
            DELETE FROM reports
            WHERE version_id = $1
            ",
            id as VersionId,
        )
        .execute(&mut **transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM game_versions_versions gvv
            WHERE gvv.joining_version_id = $1
            ",
            id as VersionId,
        )
        .execute(&mut **transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM loaders_versions
            WHERE loaders_versions.version_id = $1
            ",
            id as VersionId,
        )
        .execute(&mut **transaction)
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
        .execute(&mut **transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM files
            WHERE files.version_id = $1
            ",
            id as VersionId,
        )
        .execute(&mut **transaction)
        .await?;

        // Sync dependencies

        let project_id = sqlx::query!(
            "
            SELECT mod_id FROM versions WHERE id = $1
            ",
            id as VersionId,
        )
        .fetch_one(&mut **transaction)
        .await?;

        sqlx::query!(
            "
            UPDATE dependencies
            SET dependency_id = NULL, mod_dependency_id = $2
            WHERE dependency_id = $1
            ",
            id as VersionId,
            project_id.mod_id,
        )
        .execute(&mut **transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM dependencies WHERE mod_dependency_id = NULL AND dependency_id = NULL AND dependency_file_name = NULL
            ",
        )
        .execute(&mut **transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM dependencies WHERE dependent_id = $1
            ",
            id as VersionId,
        )
        .execute(&mut **transaction)
        .await?;

        // delete version

        sqlx::query!(
            "
            DELETE FROM versions WHERE id = $1
            ",
            id as VersionId,
        )
        .execute(&mut **transaction)
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

    pub async fn get<'a, 'b, E>(
        id: VersionId,
        executor: E,
        redis: &RedisPool,
    ) -> Result<Option<QueryVersion>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        Self::get_many(&[id], executor, redis)
            .await
            .map(|x| x.into_iter().next())
    }

    pub async fn get_many<'a, E>(
        version_ids: &[VersionId],
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<QueryVersion>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        if version_ids.is_empty() {
            return Ok(Vec::new());
        }

        use futures::stream::TryStreamExt;

        let mut version_ids_parsed: Vec<i64> = version_ids.iter().map(|x| x.0).collect();

        let mut found_versions = Vec::new();

        let versions = redis
            .multi_get::<String, _>(VERSIONS_NAMESPACE, version_ids_parsed.clone())
            .await?;

        for version in versions {
            if let Some(version) =
                version.and_then(|x| serde_json::from_str::<QueryVersion>(&x).ok())
            {
                version_ids_parsed.retain(|x| &version.inner.id.0 != x);
                found_versions.push(version);
                continue;
            }
        }

        if !version_ids_parsed.is_empty() {
            let db_versions: Vec<QueryVersion> = sqlx::query!(
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
                                status: VersionStatus::from_string(&v.status),
                                requested_status: v.requested_status
                                    .map(|x| VersionStatus::from_string(&x)),
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
                .await?;

            for version in db_versions {
                redis
                    .set_serialized_to_json(VERSIONS_NAMESPACE, version.inner.id.0, &version, None)
                    .await?;

                found_versions.push(version);
            }
        }

        Ok(found_versions)
    }

    pub async fn get_file_from_hash<'a, 'b, E>(
        algo: String,
        hash: String,
        version_id: Option<VersionId>,
        executor: E,
        redis: &RedisPool,
    ) -> Result<Option<SingleFile>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        Self::get_files_from_hash(algo, &[hash], executor, redis)
            .await
            .map(|x| {
                x.into_iter()
                    .find_or_first(|x| Some(x.version_id) == version_id)
            })
    }

    pub async fn get_files_from_hash<'a, 'b, E>(
        algorithm: String,
        hashes: &[String],
        executor: E,
        redis: &RedisPool,
    ) -> Result<Vec<SingleFile>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        if hashes.is_empty() {
            return Ok(Vec::new());
        }

        use futures::stream::TryStreamExt;

        let mut file_ids_parsed = hashes.to_vec();

        let mut found_files = Vec::new();

        let files = redis
            .multi_get::<String, _>(
                VERSION_FILES_NAMESPACE,
                file_ids_parsed
                    .iter()
                    .map(|hash| format!("{}_{}", algorithm, hash))
                    .collect::<Vec<_>>(),
            )
            .await?;

        for file in files {
            if let Some(mut file) =
                file.and_then(|x| serde_json::from_str::<Vec<SingleFile>>(&x).ok())
            {
                file_ids_parsed.retain(|x| {
                    !file
                        .iter()
                        .any(|y| y.hashes.iter().any(|z| z.0 == &algorithm && z.1 == x))
                });
                found_files.append(&mut file);
                continue;
            }
        }

        if !file_ids_parsed.is_empty() {
            let db_files: Vec<SingleFile> = sqlx::query!(
                "
                SELECT f.id, f.version_id, v.mod_id, f.url, f.filename, f.is_primary, f.size, f.file_type,
                JSONB_AGG(DISTINCT jsonb_build_object('algorithm', h.algorithm, 'hash', encode(h.hash, 'escape'))) filter (where h.hash is not null) hashes
                FROM files f
                INNER JOIN versions v on v.id = f.version_id
                INNER JOIN hashes h on h.file_id = f.id
                WHERE h.algorithm = $1 AND h.hash = ANY($2)
                GROUP BY f.id, v.mod_id, v.date_published
                ORDER BY v.date_published
                ",
                algorithm,
                &file_ids_parsed.into_iter().map(|x| x.as_bytes().to_vec()).collect::<Vec<_>>(),
            )
                .fetch_many(executor)
                .try_filter_map(|e| async {
                    Ok(e.right().map(|f| {
                        #[derive(Deserialize)]
                        struct Hash {
                            pub algorithm: String,
                            pub hash: String,
                        }

                        SingleFile {
                            id: FileId(f.id),
                            version_id: VersionId(f.version_id),
                            project_id: ProjectId(f.mod_id),
                            url: f.url,
                            filename: f.filename,
                            hashes: serde_json::from_value::<Vec<Hash>>(
                                f.hashes.unwrap_or_default(),
                            )
                                .ok()
                                .unwrap_or_default().into_iter().map(|x| (x.algorithm, x.hash)).collect(),
                            primary: f.is_primary,
                            size: f.size as u32,
                            file_type: f.file_type.map(|x| FileType::from_string(&x)),
                        }
                    }
                    ))
                })
                .try_collect::<Vec<SingleFile>>()
                .await?;

            let mut save_files: HashMap<String, Vec<SingleFile>> = HashMap::new();

            for file in db_files {
                for (algo, hash) in &file.hashes {
                    let key = format!("{}_{}", algo, hash);

                    if let Some(files) = save_files.get_mut(&key) {
                        files.push(file.clone());
                    } else {
                        save_files.insert(key, vec![file.clone()]);
                    }
                }
            }

            for (key, mut files) in save_files {
                redis
                    .set_serialized_to_json(VERSION_FILES_NAMESPACE, key, &files, None)
                    .await?;

                found_files.append(&mut files);
            }
        }

        Ok(found_files)
    }

    pub async fn clear_cache(
        version: &QueryVersion,
        redis: &RedisPool,
    ) -> Result<(), DatabaseError> {
        redis
            .delete_many(
                iter::once((VERSIONS_NAMESPACE, Some(version.inner.id.0.to_string()))).chain(
                    version.files.iter().flat_map(|file| {
                        file.hashes.iter().map(|(algo, hash)| {
                            (VERSION_FILES_NAMESPACE, Some(format!("{}_{}", algo, hash)))
                        })
                    }),
                ),
            )
            .await?;
        Ok(())
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct QueryVersion {
    pub inner: Version,

    pub files: Vec<QueryFile>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub dependencies: Vec<QueryDependency>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct QueryDependency {
    pub project_id: Option<ProjectId>,
    pub version_id: Option<VersionId>,
    pub file_name: Option<String>,
    pub dependency_type: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct QueryFile {
    pub id: FileId,
    pub url: String,
    pub filename: String,
    pub hashes: HashMap<String, String>,
    pub primary: bool,
    pub size: u32,
    pub file_type: Option<FileType>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct SingleFile {
    pub id: FileId,
    pub version_id: VersionId,
    pub project_id: ProjectId,
    pub url: String,
    pub filename: String,
    pub hashes: HashMap<String, String>,
    pub primary: bool,
    pub size: u32,
    pub file_type: Option<FileType>,
}
