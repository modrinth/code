use dashmap::DashMap;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Clone)]
pub struct JavaVersion {
    pub parsed_version: u32,
    pub version: String,
    pub architecture: String,
    pub path: String,
}

impl JavaVersion {
    pub async fn get(
        major_version: u32,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<Option<JavaVersion>> {
        let version = major_version as i32;

        let res = sqlx::query!(
            "
            SELECT
                full_version, architecture, path
            FROM java_versions
            WHERE major_version = $1
            ",
            version
        )
        .fetch_optional(exec)
        .await?;

        Ok(res.map(|x| JavaVersion {
            parsed_version: major_version,
            version: x.full_version,
            architecture: x.architecture,
            path: x.path,
        }))
    }

    pub async fn get_all(
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<DashMap<u32, Self>> {
        let res = sqlx::query!(
            "
            SELECT
                major_version, full_version, architecture, path
            FROM java_versions
            "
        )
        .fetch(exec)
        .try_fold(DashMap::new(), |acc, x| {
            acc.insert(
                x.major_version as u32,
                JavaVersion {
                    parsed_version: x.major_version as u32,
                    version: x.full_version,
                    architecture: x.architecture,
                    path: x.path,
                },
            );

            async move { Ok(acc) }
        })
        .await?;

        Ok(res)
    }

    pub async fn upsert(
        &self,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<()> {
        let major_version = self.parsed_version as i32;

        sqlx::query!(
            "
            INSERT INTO java_versions (major_version, full_version, architecture, path)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (major_version) DO UPDATE SET
                full_version = $2,
                architecture = $3,
                path = $4
            ",
            major_version,
            self.version,
            self.architecture,
            self.path,
        )
            .execute(exec)
            .await?;

        Ok(())
    }
}
