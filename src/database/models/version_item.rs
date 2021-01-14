use super::ids::*;
use super::DatabaseError;

pub struct VersionBuilder {
    pub version_id: VersionId,
    pub mod_id: ModId,
    pub author_id: UserId,
    pub name: String,
    pub version_number: String,
    pub changelog: String,
    pub files: Vec<VersionFileBuilder>,
    pub dependencies: Vec<VersionId>,
    pub game_versions: Vec<GameVersionId>,
    pub loaders: Vec<LoaderId>,
    pub release_channel: ChannelId,
    pub featured: bool,
}

pub struct VersionFileBuilder {
    pub url: String,
    pub filename: String,
    pub hashes: Vec<HashBuilder>,
    pub primary: bool,
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
            INSERT INTO files (id, version_id, url, filename)
            VALUES ($1, $2, $3, $4)
            ",
            file_id as FileId,
            version_id as VersionId,
            self.url,
            self.filename,
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
            mod_id: self.mod_id,
            author_id: self.author_id,
            name: self.name,
            version_number: self.version_number,
            changelog: self.changelog,
            changelog_url: None,
            date_published: chrono::Utc::now(),
            downloads: 0,
            release_channel: self.release_channel,
            accepted: false,
            featured: self.featured,
        };

        version.insert(&mut *transaction).await?;

        sqlx::query!(
            "
            UPDATE mods
            SET updated = NOW()
            WHERE id = $1
            ",
            self.mod_id as ModId,
        )
        .execute(&mut *transaction)
        .await?;

        for file in self.files {
            file.insert(self.version_id, transaction).await?;
        }

        for dependency in self.dependencies {
            sqlx::query!(
                "
                INSERT INTO dependencies (dependent_id, dependency_id)
                VALUES ($1, $2)
                ",
                self.version_id as VersionId,
                dependency as VersionId,
            )
            .execute(&mut *transaction)
            .await?;
        }

        for loader in self.loaders {
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

        for game_version in self.game_versions {
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

        Ok(self.version_id)
    }
}

pub struct Version {
    pub id: VersionId,
    pub mod_id: ModId,
    pub author_id: UserId,
    pub name: String,
    pub version_number: String,
    pub changelog: String,
    pub changelog_url: Option<String>,
    pub date_published: chrono::DateTime<chrono::Utc>,
    pub downloads: i32,
    pub release_channel: ChannelId,
    pub accepted: bool,
    pub featured: bool,
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
                changelog_url, date_published,
                downloads, release_channel, accepted, featured
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7,
                $8, $9,
                $10, $11
            )
            ",
            self.id as VersionId,
            self.mod_id as ModId,
            self.author_id as UserId,
            &self.name,
            &self.version_number,
            self.changelog_url.as_ref(),
            self.date_published,
            self.downloads,
            self.release_channel as ChannelId,
            self.accepted,
            self.featured
        )
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }

    // TODO: someone verify this
    pub async fn remove_full<'a, E>(id: VersionId, exec: E) -> Result<Option<()>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let result = sqlx::query!(
            "
            SELECT EXISTS(SELECT 1 FROM versions WHERE id = $1)
            ",
            id as VersionId,
        )
        .fetch_one(exec)
        .await?;

        if !result.exists.unwrap_or(false) {
            return Ok(None);
        }

        sqlx::query!(
            "
            DELETE FROM game_versions_versions gvv
            WHERE gvv.joining_version_id = $1
            ",
            id as VersionId,
        )
        .execute(exec)
        .await?;

        sqlx::query!(
            "
            DELETE FROM loaders_versions
            WHERE loaders_versions.version_id = $1
            ",
            id as VersionId,
        )
        .execute(exec)
        .await?;

        sqlx::query!(
            "
            DELETE FROM downloads
            WHERE downloads.version_id = $1
            ",
            id as VersionId,
        )
        .execute(exec)
        .await?;

        use futures::TryStreamExt;

        let files = sqlx::query!(
            "
            SELECT files.id, files.url, files.filename, files.is_primary FROM files
            WHERE files.version_id = $1
            ",
            id as VersionId,
        )
        .fetch_many(exec)
        .try_filter_map(|e| async {
            Ok(e.right().map(|c| VersionFile {
                id: FileId(c.id),
                version_id: id,
                url: c.url,
                filename: c.filename,
                primary: c.is_primary,
            }))
        })
        .try_collect::<Vec<VersionFile>>()
        .await?;

        for file in files {
            // TODO: store backblaze id in database so that we can delete the files here
            // For now, we can't delete the files since we don't have the backblaze id
            log::warn!(
                "Can't delete version file id: {} (url: {}, name: {})",
                file.id.0,
                file.url,
                file.filename
            )
        }

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
        .execute(exec)
        .await?;

        sqlx::query!(
            "
            DELETE FROM files
            WHERE files.version_id = $1
            ",
            id as VersionId,
        )
        .execute(exec)
        .await?;

        sqlx::query!(
            "
            DELETE FROM versions WHERE id = $1
            ",
            id as VersionId,
        )
        .execute(exec)
        .await?;

        sqlx::query!(
            "
            DELETE FROM dependencies WHERE dependent_id = $1
            ",
            id as VersionId,
        )
        .execute(exec)
        .await?;

        Ok(Some(()))
    }

    pub async fn get_dependencies<'a, E>(
        id: VersionId,
        exec: E,
    ) -> Result<Vec<VersionId>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        use futures::stream::TryStreamExt;

        let vec = sqlx::query!(
            "
            SELECT dependency_id id FROM dependencies
            WHERE dependent_id = $1
            ",
            id as VersionId,
        )
        .fetch_many(exec)
        .try_filter_map(|e| async { Ok(e.right().map(|v| VersionId(v.id))) })
        .try_collect::<Vec<VersionId>>()
        .await?;

        Ok(vec)
    }

    pub async fn get_mod_versions<'a, E>(
        mod_id: ModId,
        exec: E,
    ) -> Result<Vec<VersionId>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        use futures::stream::TryStreamExt;

        let vec = sqlx::query!(
            "
            SELECT id FROM versions
            WHERE mod_id = $1
            ORDER BY date_published ASC
            ",
            mod_id as ModId,
        )
        .fetch_many(exec)
        .try_filter_map(|e| async { Ok(e.right().map(|v| VersionId(v.id))) })
        .try_collect::<Vec<VersionId>>()
        .await?;

        Ok(vec)
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
                v.changelog, v.changelog_url, v.date_published, v.downloads,
                v.release_channel, v.accepted, v.featured
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
                mod_id: ModId(row.mod_id),
                author_id: UserId(row.author_id),
                name: row.name,
                version_number: row.version_number,
                changelog: row.changelog,
                changelog_url: row.changelog_url,
                date_published: row.date_published,
                downloads: row.downloads,
                release_channel: ChannelId(row.release_channel),
                accepted: row.accepted,
                featured: row.featured,
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

        let version_ids_parsed: Vec<i64> = version_ids.into_iter().map(|x| x.0).collect();
        let versions = sqlx::query!(
            "
            SELECT v.id, v.mod_id, v.author_id, v.name, v.version_number,
                v.changelog, v.changelog_url, v.date_published, v.downloads,
                v.release_channel, v.accepted, v.featured
            FROM versions v
            WHERE v.id IN (SELECT * FROM UNNEST($1::bigint[]))
            ",
            &version_ids_parsed
        )
        .fetch_many(exec)
        .try_filter_map(|e| async {
            Ok(e.right().map(|v| Version {
                id: VersionId(v.id),
                mod_id: ModId(v.mod_id),
                author_id: UserId(v.author_id),
                name: v.name,
                version_number: v.version_number,
                changelog: v.changelog,
                changelog_url: v.changelog_url,
                date_published: v.date_published,
                downloads: v.downloads,
                release_channel: ChannelId(v.release_channel),
                accepted: v.accepted,
                featured: v.featured,
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
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        let result = sqlx::query!(
            "
            SELECT v.mod_id, v.author_id, v.name, v.version_number,
                v.changelog, v.changelog_url, v.date_published, v.downloads,
                release_channels.channel, v.accepted, v.featured
            FROM versions v
            INNER JOIN release_channels ON v.release_channel = release_channels.id
            WHERE v.id = $1
            ",
            id as VersionId,
        )
        .fetch_optional(executor)
        .await?;

        if let Some(row) = result {
            use futures::TryStreamExt;

            let game_versions: Vec<String> = sqlx::query!(
                "
                SELECT gv.version FROM game_versions_versions gvv
                INNER JOIN game_versions gv ON gvv.game_version_id=gv.id
                WHERE gvv.joining_version_id = $1
                ",
                id as VersionId,
            )
            .fetch_many(executor)
            .try_filter_map(|e| async { Ok(e.right().map(|c| c.version)) })
            .try_collect::<Vec<String>>()
            .await?;

            let loaders: Vec<String> = sqlx::query!(
                "
                SELECT loaders.loader FROM loaders
                INNER JOIN loaders_versions ON loaders.id = loaders_versions.loader_id
                WHERE loaders_versions.version_id = $1
                ",
                id as VersionId,
            )
            .fetch_many(executor)
            .try_filter_map(|e| async { Ok(e.right().map(|c| c.loader)) })
            .try_collect::<Vec<String>>()
            .await?;

            let mut files = sqlx::query!(
                "
                SELECT files.id, files.url, files.filename, files.is_primary FROM files
                WHERE files.version_id = $1
                ",
                id as VersionId,
            )
            .fetch_many(executor)
            .try_filter_map(|e| async {
                Ok(e.right().map(|c| QueryFile {
                    id: FileId(c.id),
                    url: c.url,
                    filename: c.filename,
                    hashes: std::collections::HashMap::new(),
                    primary: c.is_primary,
                }))
            })
            .try_collect::<Vec<QueryFile>>()
            .await?;

            for file in files.iter_mut() {
                let files = sqlx::query!(
                    "
                    SELECT hashes.algorithm, hashes.hash FROM hashes
                    WHERE hashes.file_id = $1
                    ",
                    file.id as FileId
                )
                .fetch_many(executor)
                .try_filter_map(|e| async { Ok(e.right().map(|c| (c.algorithm, c.hash))) })
                .try_collect::<Vec<(String, Vec<u8>)>>()
                .await?;

                file.hashes.extend(files);
            }

            Ok(Some(QueryVersion {
                id,
                mod_id: ModId(row.mod_id),
                author_id: UserId(row.author_id),
                name: row.name,
                version_number: row.version_number,
                changelog: row.changelog,
                changelog_url: row.changelog_url,
                date_published: row.date_published,
                downloads: row.downloads,

                release_channel: row.channel,
                files,
                loaders,
                game_versions,
                accepted: row.accepted,
                featured: row.featured,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_many_full<'a, E>(
        version_ids: Vec<VersionId>,
        exec: E,
    ) -> Result<Vec<Option<QueryVersion>>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres> + Copy,
    {
        // TODO: this could be optimized
        futures::future::try_join_all(version_ids.into_iter().map(|id| Self::get_full(id, exec)))
            .await
    }
}

pub struct ReleaseChannel {
    pub id: ChannelId,
    pub channel: String,
}

pub struct VersionFile {
    pub id: FileId,
    pub version_id: VersionId,
    pub url: String,
    pub filename: String,
    pub primary: bool,
}

pub struct FileHash {
    pub file_id: FileId,
    pub algorithm: String,
    pub hash: Vec<u8>,
}

pub struct QueryVersion {
    pub id: VersionId,
    pub mod_id: ModId,
    pub author_id: UserId,
    pub name: String,
    pub version_number: String,
    pub changelog: String,
    pub changelog_url: Option<String>,
    pub date_published: chrono::DateTime<chrono::Utc>,
    pub downloads: i32,

    pub release_channel: String,
    pub files: Vec<QueryFile>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub accepted: bool,
    pub featured: bool,
}

pub struct QueryFile {
    pub id: FileId,
    pub url: String,
    pub filename: String,
    pub hashes: std::collections::HashMap<String, Vec<u8>>,
    pub primary: bool,
}
