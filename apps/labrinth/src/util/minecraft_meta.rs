use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use eyre::{Result, WrapErr};
use minecraft_meta_model::upstream::mojang::VersionType;
use minecraft_meta_model::model::{MinecraftVersion, MinecraftVersionName};

use super::refreshed_data::RefreshedData;

use crate::database::MinecraftMetaDb;
use crate::models::minecraft::{
    legacy_game_version_name, minecraft_version_date,
};

const CACHE_EXPIRY: Duration = Duration::from_secs(10 * 60);
const CACHE_EXPIRY_JITTER: Duration = Duration::from_secs(2 * 60);

#[derive(Clone, Debug)]
pub struct MinecraftVersionEntry {
    pub name: MinecraftVersionName,
    pub version_type: VersionType,
    pub released_at: DateTime<Utc>,
}

/// Shared Minecraft metadata with jittered refreshes and stale-on-error fallback.
pub struct MinecraftMeta {
    cache: RefreshedData<Arc<[MinecraftVersionEntry]>>,
}

impl MinecraftMeta {
    pub async fn new(db: MinecraftMetaDb) -> Result<Self> {
        let db = db.0;
        let cache =
            RefreshedData::new(CACHE_EXPIRY, CACHE_EXPIRY_JITTER, move || {
                let db = db.clone();
                async move { Self::load_versions(&db).await }
            })
            .await?;
        Ok(Self { cache })
    }

    pub async fn refresh(&self) {
        self.cache.refresh().await;
    }

    pub async fn minecraft_versions(&self) -> Result<Arc<[MinecraftVersionEntry]>> {
        Ok(self.cache.get().await)
    }

    pub async fn validate_minecraft_versions(
        &self,
        names: &[MinecraftVersionName],
    ) -> Result<()> {
        if names.is_empty() {
            return Ok(());
        }
        let versions = self.minecraft_versions().await?;

        let invalid_versions = names.iter()
            .filter(|name| {
                !versions.iter().any(|version| {
                    legacy_game_version_name(&name.0) == version.name.0
                })
            })
            .map(|name| format!("`{}`", name.0))
            .collect::<Vec<_>>();
        eyre::ensure!(
            invalid_versions.is_empty(),
            "invalid game versions: {}",
            invalid_versions.join(", ")
        );
        Ok(())
    }

    async fn load_versions(db: &toasty::Db) -> Result<Arc<[MinecraftVersionEntry]>> {
        let mut connection = db
            .connection()
            .await
            .wrap_err("acquiring minecraft-meta connection")?;
        let versions = MinecraftVersion::all()
            .exec(&mut connection)
            .await
            .wrap_err("loading Minecraft game versions")?;

        let mut versions = versions.into_iter().map(|version| {
            let entry = version.catalog_entry.0;
            let released_at = DateTime::from_timestamp(
                entry.release_time.as_second(),
                entry.release_time.subsec_nanosecond() as u32,
            ).ok_or_else(|| eyre::eyre!("minecraft-meta release timestamp is out of range"))?;
            Ok(MinecraftVersionEntry {
                name: MinecraftVersionName(legacy_game_version_name(&version.name.0).to_owned()),
                version_type: entry.ty,
                released_at,
            })
        }).collect::<Result<Vec<_>>>()?;
        versions.sort_by(|a, b| {
            minecraft_version_date(b)
                .cmp(&minecraft_version_date(a))
                .then_with(|| a.name.0.cmp(&b.name.0))
        });
        Ok(versions.into())
    }
}
