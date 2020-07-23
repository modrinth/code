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
            let file_id = generate_file_id(&mut *transaction).await?;
            sqlx::query(
                "
                INSERT INTO files (id, version_id, url, filename)
                VALUES ($1, $2, $3, $4)
                ",
            )
            .bind(file_id)
            .bind(self.version_id)
            .bind(file.url)
            .bind(file.filename)
            .execute(&mut *transaction)
            .await?;

            for hash in file.hashes {
                sqlx::query(
                    "
                    INSERT INTO hashes (file_id, algorithm, hash)
                    VALUES ($1, $2, $3)
                    ",
                )
                .bind(file_id)
                .bind(hash.algorithm)
                .bind(hash.hash)
                .execute(&mut *transaction)
                .await?;
            }
        }

        for dependency in self.dependencies {
            sqlx::query(
                "
                INSERT INTO dependencies (dependent_id, dependency_id)
                VALUES ($1, $2)
                ",
            )
            .bind(self.version_id)
            .bind(dependency)
            .execute(&mut *transaction)
            .await?;
        }

        for loader in self.loaders {
            sqlx::query(
                "
                INSERT INTO dependencies (loader_id, version_id)
                VALUES ($1, $2)
                ",
            )
            .bind(loader)
            .bind(self.version_id)
            .execute(&mut *transaction)
            .await?;
        }

        for game_version in self.game_versions {
            sqlx::query(
                "
                INSERT INTO dependencies (game_version_id, joining_version_id)
                VALUES ($1, $2)
                ",
            )
            .bind(game_version)
            .bind(self.version_id)
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
        sqlx::query(
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
        )
        .bind(self.id)
        .bind(self.mod_id)
        .bind(&self.name)
        .bind(&self.version_number)
        .bind(self.changelog_url.as_ref())
        .bind(self.date_published)
        .bind(self.downloads)
        .bind(self.release_channel)
        .execute(&mut *transaction)
        .await?;

        Ok(())
    }

    pub async fn get_dependencies<'a, E>(&self, exec: E) -> Result<Vec<VersionId>, sqlx::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        use futures::stream::TryStreamExt;

        let vec = sqlx::query_as::<_, (VersionId,)>(
            "
            SELECT id FROM versions v
            INNER JOIN dependencies d ON d.dependency_id = v.id
            WHERE d.dependent_id = $1
            ",
        )
        .bind(self.id)
        .fetch_many(exec)
        .try_filter_map(|e| async { Ok(e.right().map(|(v,)| v)) })
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
