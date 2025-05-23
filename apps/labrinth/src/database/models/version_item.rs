use super::DatabaseError;
use super::ids::*;
use super::loader_fields::VersionField;
use crate::database::models::loader_fields::{
    QueryLoaderField, QueryLoaderFieldEnumValue, QueryVersionField,
};
use crate::database::redis::RedisPool;
use crate::models::projects::{FileType, VersionStatus};
use chrono::{DateTime, Utc};
use dashmap::{DashMap, DashSet};
use futures::TryStreamExt;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::iter;

pub const VERSIONS_NAMESPACE: &str = "versions";
const VERSION_FILES_NAMESPACE: &str = "versions_files";

#[derive(Clone)]
pub struct VersionBuilder {
    pub version_id: DBVersionId,
    pub project_id: DBProjectId,
    pub author_id: DBUserId,
    pub name: String,
    pub version_number: String,
    pub changelog: String,
    pub files: Vec<VersionFileBuilder>,
    pub dependencies: Vec<DependencyBuilder>,
    pub loaders: Vec<LoaderId>,
    pub version_fields: Vec<VersionField>,
    pub version_type: String,
    pub featured: bool,
    pub status: VersionStatus,
    pub requested_status: Option<VersionStatus>,
    pub ordering: Option<i32>,
}

#[derive(Clone)]
pub struct DependencyBuilder {
    pub project_id: Option<DBProjectId>,
    pub version_id: Option<DBVersionId>,
    pub file_name: Option<String>,
    pub dependency_type: String,
}

impl DependencyBuilder {
    pub async fn insert_many(
        builders: Vec<Self>,
        version_id: DBVersionId,
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
    ) -> Result<Option<DBProjectId>, DatabaseError> {
        Ok(if let Some(project_id) = self.project_id {
            Some(project_id)
        } else if let Some(version_id) = self.version_id {
            sqlx::query!(
                "
                SELECT mod_id FROM versions WHERE id = $1
                ",
                version_id as DBVersionId,
            )
            .fetch_optional(&mut **transaction)
            .await?
            .map(|x| DBProjectId(x.mod_id))
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
    pub async fn insert(
        self,
        version_id: DBVersionId,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<DBFileId, DatabaseError> {
        let file_id = generate_file_id(&mut *transaction).await?;

        sqlx::query!(
            "
            INSERT INTO files (id, version_id, url, filename, is_primary, size, file_type)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ",
            file_id as DBFileId,
            version_id as DBVersionId,
            self.url,
            self.filename,
            self.primary,
            self.size as i32,
            self.file_type.map(|x| x.as_str()),
        )
        .execute(&mut **transaction)
        .await?;

        for hash in self.hashes {
            sqlx::query!(
                "
                INSERT INTO hashes (file_id, algorithm, hash)
                VALUES ($1, $2, $3)
                ",
                file_id as DBFileId,
                hash.algorithm,
                hash.hash,
            )
            .execute(&mut **transaction)
            .await?;
        }

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
    ) -> Result<DBVersionId, DatabaseError> {
        let version = DBVersion {
            id: self.version_id,
            project_id: self.project_id,
            author_id: self.author_id,
            name: self.name,
            version_number: self.version_number,
            changelog: self.changelog,
            date_published: Utc::now(),
            downloads: 0,
            featured: self.featured,
            version_type: self.version_type,
            status: self.status,
            requested_status: self.requested_status,
            ordering: self.ordering,
        };

        version.insert(transaction).await?;

        sqlx::query!(
            "
            UPDATE mods
            SET updated = NOW()
            WHERE id = $1
            ",
            self.project_id as DBProjectId,
        )
        .execute(&mut **transaction)
        .await?;

        let VersionBuilder {
            dependencies,
            loaders,
            files,
            version_id,
            ..
        } = self;

        for file in files {
            file.insert(version_id, transaction).await?;
        }

        DependencyBuilder::insert_many(
            dependencies,
            self.version_id,
            transaction,
        )
        .await?;

        let loader_versions = loaders
            .iter()
            .map(|&loader_id| DBLoaderVersion {
                loader_id,
                version_id,
            })
            .collect_vec();
        DBLoaderVersion::insert_many(loader_versions, transaction).await?;

        VersionField::insert_many(self.version_fields, transaction).await?;

        Ok(self.version_id)
    }
}

#[derive(Serialize, Deserialize)]
pub struct DBLoaderVersion {
    pub loader_id: LoaderId,
    pub version_id: DBVersionId,
}

impl DBLoaderVersion {
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

#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct DBVersion {
    pub id: DBVersionId,
    pub project_id: DBProjectId,
    pub author_id: DBUserId,
    pub name: String,
    pub version_number: String,
    pub changelog: String,
    pub date_published: DateTime<Utc>,
    pub downloads: i32,
    pub version_type: String,
    pub featured: bool,
    pub status: VersionStatus,
    pub requested_status: Option<VersionStatus>,
    pub ordering: Option<i32>,
}

impl DBVersion {
    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::error::Error> {
        sqlx::query!(
            "
            INSERT INTO versions (
                id, mod_id, author_id, name, version_number,
                changelog, date_published, downloads,
                version_type, featured, status, ordering
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8,
                $9, $10, $11, $12
            )
            ",
            self.id as DBVersionId,
            self.project_id as DBProjectId,
            self.author_id as DBUserId,
            &self.name,
            &self.version_number,
            self.changelog,
            self.date_published,
            self.downloads,
            &self.version_type,
            self.featured,
            self.status.as_str(),
            self.ordering
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn remove_full(
        id: DBVersionId,
        redis: &RedisPool,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Option<()>, DatabaseError> {
        let result = Self::get(id, &mut **transaction, redis).await?;

        let result = if let Some(result) = result {
            result
        } else {
            return Ok(None);
        };

        DBVersion::clear_cache(&result, redis).await?;

        sqlx::query!(
            "
            UPDATE reports
            SET version_id = NULL
            WHERE version_id = $1
            ",
            id as DBVersionId,
        )
        .execute(&mut **transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM version_fields vf
            WHERE vf.version_id = $1
            ",
            id as DBVersionId,
        )
        .execute(&mut **transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM loaders_versions
            WHERE loaders_versions.version_id = $1
            ",
            id as DBVersionId,
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
            id as DBVersionId
        )
        .execute(&mut **transaction)
        .await?;

        sqlx::query!(
            "
            DELETE FROM files
            WHERE files.version_id = $1
            ",
            id as DBVersionId,
        )
        .execute(&mut **transaction)
        .await?;

        // Sync dependencies

        let project_id = sqlx::query!(
            "
            SELECT mod_id FROM versions WHERE id = $1
            ",
            id as DBVersionId,
        )
        .fetch_one(&mut **transaction)
        .await?;

        sqlx::query!(
            "
            UPDATE dependencies
            SET dependency_id = NULL, mod_dependency_id = $2
            WHERE dependency_id = $1
            ",
            id as DBVersionId,
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
            id as DBVersionId,
        )
        .execute(&mut **transaction)
        .await?;

        // delete version

        sqlx::query!(
            "
            DELETE FROM versions WHERE id = $1
            ",
            id as DBVersionId,
        )
        .execute(&mut **transaction)
        .await?;

        crate::database::models::DBProject::clear_cache(
            DBProjectId(project_id.mod_id),
            None,
            None,
            redis,
        )
        .await?;

        Ok(Some(()))
    }

    pub async fn get<'a, 'b, E>(
        id: DBVersionId,
        executor: E,
        redis: &RedisPool,
    ) -> Result<Option<VersionQueryResult>, DatabaseError>
    where
        E: sqlx::Acquire<'a, Database = sqlx::Postgres>,
    {
        Self::get_many(&[id], executor, redis)
            .await
            .map(|x| x.into_iter().next())
    }

    pub async fn get_many<'a, E>(
        version_ids: &[DBVersionId],
        exec: E,
        redis: &RedisPool,
    ) -> Result<Vec<VersionQueryResult>, DatabaseError>
    where
        E: sqlx::Acquire<'a, Database = sqlx::Postgres>,
    {
        let mut val = redis.get_cached_keys(
            VERSIONS_NAMESPACE,
            &version_ids.iter().map(|x| x.0).collect::<Vec<_>>(),
            |version_ids| async move {
                let mut exec = exec.acquire().await?;

                let loader_field_enum_value_ids = DashSet::new();
                let version_fields: DashMap<DBVersionId, Vec<QueryVersionField>> = sqlx::query!(
                    "
                    SELECT version_id, field_id, int_value, enum_value, string_value
                    FROM version_fields
                    WHERE version_id = ANY($1)
                    ",
                    &version_ids
                )
                    .fetch(&mut *exec)
                    .try_fold(
                        DashMap::new(),
                        |acc: DashMap<DBVersionId, Vec<QueryVersionField>>, m| {
                            let qvf = QueryVersionField {
                                version_id: DBVersionId(m.version_id),
                                field_id: LoaderFieldId(m.field_id),
                                int_value: m.int_value,
                                enum_value: if m.enum_value == -1  { None } else { Some(LoaderFieldEnumValueId(m.enum_value)) },
                                string_value: m.string_value,
                            };

                            if m.enum_value != -1 {
                                loader_field_enum_value_ids.insert(LoaderFieldEnumValueId(m.enum_value));
                            }

                            acc.entry(DBVersionId(m.version_id)).or_default().push(qvf);
                            async move { Ok(acc) }
                        },
                    )
                    .await?;

                #[derive(Default)]
                struct VersionLoaderData {
                    loaders: Vec<String>,
                    project_types: Vec<String>,
                    games: Vec<String>,
                    loader_loader_field_ids: Vec<LoaderFieldId>,
                }

                let loader_field_ids = DashSet::new();
                let loaders_ptypes_games: DashMap<DBVersionId, VersionLoaderData> = sqlx::query!(
                    "
                    SELECT DISTINCT version_id,
                        ARRAY_AGG(DISTINCT l.loader) filter (where l.loader is not null) loaders,
                        ARRAY_AGG(DISTINCT pt.name) filter (where pt.name is not null) project_types,
                        ARRAY_AGG(DISTINCT g.slug) filter (where g.slug is not null) games,
                        ARRAY_AGG(DISTINCT lfl.loader_field_id) filter (where lfl.loader_field_id is not null) loader_fields
                    FROM versions v
                    INNER JOIN loaders_versions lv ON v.id = lv.version_id
                    INNER JOIN loaders l ON lv.loader_id = l.id
                    INNER JOIN loaders_project_types lpt ON lpt.joining_loader_id = l.id
                    INNER JOIN project_types pt ON pt.id = lpt.joining_project_type_id
                    INNER JOIN loaders_project_types_games lptg ON lptg.loader_id = l.id AND lptg.project_type_id = pt.id
                    INNER JOIN games g ON lptg.game_id = g.id
                    LEFT JOIN loader_fields_loaders lfl ON lfl.loader_id = l.id
                    WHERE v.id = ANY($1)
                    GROUP BY version_id
                    ",
                    &version_ids
                ).fetch(&mut *exec)
                    .map_ok(|m| {
                        let version_id = DBVersionId(m.version_id);

                        // Add loader fields to the set we need to fetch
                        let loader_loader_field_ids = m.loader_fields.unwrap_or_default().into_iter().map(LoaderFieldId).collect::<Vec<_>>();
                        for loader_field_id in loader_loader_field_ids.iter() {
                            loader_field_ids.insert(*loader_field_id);
                        }

                        // Add loader + loader associated data to the map
                        let version_loader_data = VersionLoaderData {
                            loaders: m.loaders.unwrap_or_default(),
                            project_types: m.project_types.unwrap_or_default(),
                            games: m.games.unwrap_or_default(),
                            loader_loader_field_ids,
                        };
                        (version_id,version_loader_data)

                    }
                    ).try_collect().await?;

                // Fetch all loader fields from any version
                let loader_fields: Vec<QueryLoaderField> = sqlx::query!(
                    "
                    SELECT DISTINCT id, field, field_type, enum_type, min_val, max_val, optional
                    FROM loader_fields lf
                    WHERE id = ANY($1)
                    ",
                    &loader_field_ids.iter().map(|x| x.0).collect::<Vec<_>>()
                )
                    .fetch(&mut *exec)
                    .map_ok(|m| QueryLoaderField {
                        id: LoaderFieldId(m.id),
                        field: m.field,
                        field_type: m.field_type,
                        enum_type: m.enum_type.map(LoaderFieldEnumId),
                        min_val: m.min_val,
                        max_val: m.max_val,
                        optional: m.optional,
                    })
                    .try_collect()
                    .await?;

                let loader_field_enum_values: Vec<QueryLoaderFieldEnumValue> = sqlx::query!(
                    "
                    SELECT DISTINCT id, enum_id, value, ordering, created, metadata
                    FROM loader_field_enum_values lfev
                    WHERE id = ANY($1)
                    ORDER BY enum_id, ordering, created ASC
                    ",
                    &loader_field_enum_value_ids
                        .iter()
                        .map(|x| x.0)
                        .collect::<Vec<_>>()
                )
                    .fetch(&mut *exec)
                    .map_ok(|m| QueryLoaderFieldEnumValue {
                        id: LoaderFieldEnumValueId(m.id),
                        enum_id: LoaderFieldEnumId(m.enum_id),
                        value: m.value,
                        ordering: m.ordering,
                        created: m.created,
                        metadata: m.metadata,
                    })
                    .try_collect()
                    .await?;

                #[derive(Deserialize)]
                struct Hash {
                    pub file_id: DBFileId,
                    pub algorithm: String,
                    pub hash: String,
                }

                #[derive(Deserialize)]
                struct File {
                    pub id: DBFileId,
                    pub url: String,
                    pub filename: String,
                    pub primary: bool,
                    pub size: u32,
                    pub file_type: Option<FileType>,
                }

                let file_ids = DashSet::new();
                let reverse_file_map = DashMap::new();
                let files : DashMap<DBVersionId, Vec<File>> = sqlx::query!(
                    "
                    SELECT DISTINCT version_id, f.id, f.url, f.filename, f.is_primary, f.size, f.file_type
                    FROM files f
                    WHERE f.version_id = ANY($1)
                    ",
                    &version_ids
                ).fetch(&mut *exec)
                    .try_fold(DashMap::new(), |acc : DashMap<DBVersionId, Vec<File>>, m| {
                        let file = File {
                            id: DBFileId(m.id),
                            url: m.url,
                            filename: m.filename,
                            primary: m.is_primary,
                            size: m.size as u32,
                            file_type: m.file_type.map(|x| FileType::from_string(&x)),
                        };

                        file_ids.insert(DBFileId(m.id));
                        reverse_file_map.insert(DBFileId(m.id), DBVersionId(m.version_id));

                        acc.entry(DBVersionId(m.version_id))
                            .or_default()
                            .push(file);
                        async move { Ok(acc) }
                    }
                    ).await?;

                let hashes: DashMap<DBVersionId, Vec<Hash>> = sqlx::query!(
                    "
                    SELECT DISTINCT file_id, algorithm, encode(hash, 'escape') hash
                    FROM hashes
                    WHERE file_id = ANY($1)
                    ",
                    &file_ids.iter().map(|x| x.0).collect::<Vec<_>>()
                )
                    .fetch(&mut *exec)
                    .try_fold(DashMap::new(), |acc: DashMap<DBVersionId, Vec<Hash>>, m| {
                        if let Some(found_hash) = m.hash {
                            let hash = Hash {
                                file_id: DBFileId(m.file_id),
                                algorithm: m.algorithm,
                                hash: found_hash,
                            };

                            if let Some(version_id) = reverse_file_map.get(&DBFileId(m.file_id)) {
                                acc.entry(*version_id).or_default().push(hash);
                            }
                        }
                        async move { Ok(acc) }
                    })
                    .await?;

                let dependencies : DashMap<DBVersionId, Vec<DependencyQueryResult>> = sqlx::query!(
                    "
                    SELECT DISTINCT dependent_id as version_id, d.mod_dependency_id as dependency_project_id, d.dependency_id as dependency_version_id, d.dependency_file_name as file_name, d.dependency_type as dependency_type
                    FROM dependencies d
                    WHERE dependent_id = ANY($1)
                    ",
                    &version_ids
                ).fetch(&mut *exec)
                    .try_fold(DashMap::new(), |acc : DashMap<_,Vec<DependencyQueryResult>>, m| {
                        let dependency = DependencyQueryResult {
                            project_id: m.dependency_project_id.map(DBProjectId),
                            version_id: m.dependency_version_id.map(DBVersionId),
                            file_name: m.file_name,
                            dependency_type: m.dependency_type,
                        };

                        acc.entry(DBVersionId(m.version_id))
                            .or_default()
                            .push(dependency);
                        async move { Ok(acc) }
                    }
                    ).await?;

                let res = sqlx::query!(
                    "
                    SELECT v.id id, v.mod_id mod_id, v.author_id author_id, v.name version_name, v.version_number version_number,
                    v.changelog changelog, v.date_published date_published, v.downloads downloads,
                    v.version_type version_type, v.featured featured, v.status status, v.requested_status requested_status, v.ordering ordering
                    FROM versions v
                    WHERE v.id = ANY($1);
                    ",
                    &version_ids
                )
                    .fetch(&mut *exec)
                    .try_fold(DashMap::new(), |acc, v| {
                        let version_id = DBVersionId(v.id);
                        let VersionLoaderData {
                            loaders,
                            project_types,
                            games,
                            loader_loader_field_ids,
                        } = loaders_ptypes_games.remove(&version_id).map(|x|x.1).unwrap_or_default();
                        let files = files.remove(&version_id).map(|x|x.1).unwrap_or_default();
                        let hashes = hashes.remove(&version_id).map(|x|x.1).unwrap_or_default();
                        let version_fields = version_fields.remove(&version_id).map(|x|x.1).unwrap_or_default();
                        let dependencies = dependencies.remove(&version_id).map(|x|x.1).unwrap_or_default();

                        let loader_fields = loader_fields.iter()
                            .filter(|x| loader_loader_field_ids.contains(&x.id))
                            .collect::<Vec<_>>();

                        let query_version = VersionQueryResult {
                            inner: DBVersion {
                                id: DBVersionId(v.id),
                                project_id: DBProjectId(v.mod_id),
                                author_id: DBUserId(v.author_id),
                                name: v.version_name,
                                version_number: v.version_number,
                                changelog: v.changelog,
                                date_published: v.date_published,
                                downloads: v.downloads,
                                version_type: v.version_type,
                                featured: v.featured,
                                status: VersionStatus::from_string(&v.status),
                                requested_status: v.requested_status
                                    .map(|x| VersionStatus::from_string(&x)),
                                ordering: v.ordering,
                            },
                            files: {
                                let mut files = files.into_iter().map(|x| {
                                    let mut file_hashes = HashMap::new();

                                    for hash in hashes.iter() {
                                        if hash.file_id == x.id {
                                            file_hashes.insert(
                                                hash.algorithm.clone(),
                                                hash.hash.clone(),
                                            );
                                        }
                                    }

                                    FileQueryResult {
                                        id: x.id,
                                        url: x.url.clone(),
                                        filename: x.filename.clone(),
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
                            version_fields: VersionField::from_query_json(version_fields, &loader_fields, &loader_field_enum_values, false),
                            loaders,
                            project_types,
                            games,
                            dependencies,
                        };

                        acc.insert(v.id, query_version);
                        async move { Ok(acc) }
                    })
                    .await?;

                Ok(res)
            },
        ).await?;

        val.sort();

        Ok(val)
    }

    pub async fn get_file_from_hash<'a, 'b, E>(
        algo: String,
        hash: String,
        version_id: Option<DBVersionId>,
        executor: E,
        redis: &RedisPool,
    ) -> Result<Option<DBFile>, DatabaseError>
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
    ) -> Result<Vec<DBFile>, DatabaseError>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let val = redis.get_cached_keys(
            VERSION_FILES_NAMESPACE,
            &hashes.iter().map(|x| format!("{algorithm}_{x}")).collect::<Vec<_>>(),
            |file_ids| async move {
                let files = sqlx::query!(
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
                    &file_ids.into_iter().flat_map(|x| x.split('_').last().map(|x| x.as_bytes().to_vec())).collect::<Vec<_>>(),
                )
                    .fetch(executor)
                    .try_fold(DashMap::new(), |acc, f| {
                        #[derive(Deserialize)]
                        struct Hash {
                            pub algorithm: String,
                            pub hash: String,
                        }

                        let hashes = serde_json::from_value::<Vec<Hash>>(
                            f.hashes.unwrap_or_default(),
                        )
                            .ok()
                            .unwrap_or_default().into_iter().map(|x| (x.algorithm, x.hash))
                            .collect::<HashMap<_, _>>();

                        if let Some(hash) = hashes.get(&algorithm) {
                            let key = format!("{algorithm}_{hash}");

                            let file = DBFile {
                                id: DBFileId(f.id),
                                version_id: DBVersionId(f.version_id),
                                project_id: DBProjectId(f.mod_id),
                                url: f.url,
                                filename: f.filename,
                                hashes,
                                primary: f.is_primary,
                                size: f.size as u32,
                                file_type: f.file_type.map(|x| FileType::from_string(&x)),
                            };

                            acc.insert(key, file);
                        }

                        async move { Ok(acc) }
                    })
                    .await?;

                Ok(files)
            }
        ).await?;

        Ok(val)
    }

    pub async fn clear_cache(
        version: &VersionQueryResult,
        redis: &RedisPool,
    ) -> Result<(), DatabaseError> {
        let mut redis = redis.connect().await?;

        redis
            .delete_many(
                iter::once((
                    VERSIONS_NAMESPACE,
                    Some(version.inner.id.0.to_string()),
                ))
                .chain(version.files.iter().flat_map(
                    |file| {
                        file.hashes.iter().map(|(algo, hash)| {
                            (
                                VERSION_FILES_NAMESPACE,
                                Some(format!("{algo}_{hash}")),
                            )
                        })
                    },
                )),
            )
            .await?;
        Ok(())
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct VersionQueryResult {
    pub inner: DBVersion,

    pub files: Vec<FileQueryResult>,
    pub version_fields: Vec<VersionField>,
    pub loaders: Vec<String>,
    pub project_types: Vec<String>,
    pub games: Vec<String>,
    pub dependencies: Vec<DependencyQueryResult>,
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct DependencyQueryResult {
    pub project_id: Option<DBProjectId>,
    pub version_id: Option<DBVersionId>,
    pub file_name: Option<String>,
    pub dependency_type: String,
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct FileQueryResult {
    pub id: DBFileId,
    pub url: String,
    pub filename: String,
    pub hashes: HashMap<String, String>,
    pub primary: bool,
    pub size: u32,
    pub file_type: Option<FileType>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct DBFile {
    pub id: DBFileId,
    pub version_id: DBVersionId,
    pub project_id: DBProjectId,
    pub url: String,
    pub filename: String,
    pub hashes: HashMap<String, String>,
    pub primary: bool,
    pub size: u32,
    pub file_type: Option<FileType>,
}

impl std::cmp::Ord for VersionQueryResult {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl std::cmp::PartialOrd for VersionQueryResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for DBVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        let ordering_order = match (self.ordering, other.ordering) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(a), Some(b)) => a.cmp(&b),
        };

        match ordering_order {
            Ordering::Equal => self.date_published.cmp(&other.date_published),
            ordering => ordering,
        }
    }
}

impl std::cmp::PartialOrd for DBVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use chrono::Months;

    use super::*;

    #[test]
    fn test_version_sorting() {
        let versions = vec![
            get_version(4, None, months_ago(6)),
            get_version(3, None, months_ago(7)),
            get_version(2, Some(1), months_ago(6)),
            get_version(1, Some(0), months_ago(4)),
            get_version(0, Some(0), months_ago(5)),
        ];

        let sorted = versions.iter().cloned().sorted().collect_vec();

        let expected_sorted_ids = vec![0, 1, 2, 3, 4];
        let actual_sorted_ids = sorted.iter().map(|v| v.id.0).collect_vec();
        assert_eq!(expected_sorted_ids, actual_sorted_ids);
    }

    fn months_ago(months: u32) -> DateTime<Utc> {
        Utc::now().checked_sub_months(Months::new(months)).unwrap()
    }

    fn get_version(
        id: i64,
        ordering: Option<i32>,
        date_published: DateTime<Utc>,
    ) -> DBVersion {
        DBVersion {
            id: DBVersionId(id),
            ordering,
            date_published,
            project_id: DBProjectId(0),
            author_id: DBUserId(0),
            name: Default::default(),
            version_number: Default::default(),
            changelog: Default::default(),
            downloads: Default::default(),
            version_type: Default::default(),
            featured: Default::default(),
            status: VersionStatus::Listed,
            requested_status: Default::default(),
        }
    }
}
