use super::ids::*;
use super::DatabaseError;

pub struct VersionBuilder {
    pub version_id: VersionId,
    pub mod_id: ModId,
    pub name: String,
    pub version_number: String,
    pub changelog_url: Option<String>,
    pub files: Vec<VersionFileBuilder>,
    pub dependencies: Vec<VersionId>,
    pub game_versions: Vec<GameVersionId>,
    pub loaders: Vec<LoaderId>,
    pub release_channel: ChannelId,
}

pub struct VersionFileBuilder {
    pub url: String,
    pub filename: String,
    pub hashes: Vec<HashBuilder>,
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
            name: self.name,
            version_number: self.version_number,
            changelog_url: self.changelog_url,
            date_published: chrono::Utc::now(),
            downloads: 0,
            release_channel: self.release_channel,
        };

        version.insert(&mut *transaction).await?;

        for file in self.files {
            file.insert(self.version_id, transaction);
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
    pub name: String,
    pub version_number: String,
    pub changelog_url: Option<String>,
    pub date_published: chrono::DateTime<chrono::Utc>,
    pub downloads: i32,
    pub release_channel: ChannelId,
}

impl Version {
    pub async fn insert(
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), sqlx::error::Error> {
        sqlx::query!(
            "
            INSERT INTO versions (
                id, mod_id, name, version_number,
                changelog_url, date_published,
                downloads, release_channel
            )
            VALUES (
                $1, $2, $3, $4,
                $5, $6,
                $7, $8
            )
            ",
            self.id as VersionId,
            self.mod_id as ModId,
            &self.name,
            &self.version_number,
            self.changelog_url.as_ref(),
            self.date_published,
            self.downloads,
            self.release_channel as ChannelId,
        )
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }

    pub async fn get_dependencies<'a, E>(&self, exec: E) -> Result<Vec<VersionId>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        use futures::stream::TryStreamExt;

        let vec = sqlx::query!(
            "
            SELECT dependency_id id FROM dependencies
            WHERE dependent_id = $1
            ",
            self.id as VersionId,
        )
        .fetch_many(exec)
        .try_filter_map(|e| async { Ok(e.right().map(|v| VersionId(v.id))) })
        .try_collect::<Vec<VersionId>>()
        .await?;

        Ok(vec)
    }
}

pub struct ReleaseChannel {
    pub id: ChannelId,
    pub channel: String,
}
pub struct Loader {
    pub id: LoaderId,
    pub loader: String,
}
pub struct GameVersion {
    pub id: GameVersionId,
    pub version: String,
}

pub struct VersionFile {
    pub id: FileId,
    pub version_id: VersionId,
    pub url: String,
    pub filename: String,
}

pub struct FileHash {
    pub file_id: FileId,
    pub algorithm: String,
    pub hash: Vec<u8>,
}

pub struct Category {
    pub id: CategoryId,
    pub category: String,
}
