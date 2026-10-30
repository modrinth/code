use std::sync::Arc;
use std::time::Duration;

use eyre::{Result, WrapErr};
use minecraft_meta_model::model::{MinecraftVersion, MinecraftVersionName};


use super::refreshed_data::RefreshedData;

use crate::database::MinecraftMetaDb;
use crate::models::minecraft::{
    legacy_game_version_name, minecraft_version_date,
};

const CACHE_EXPIRY: Duration = Duration::from_secs(30 * 60);
const CACHE_EXPIRY_JITTER: Duration = Duration::from_secs(5 * 60);

/// Shared Minecraft metadata with jittered refreshes and stale-on-error fallback.
pub struct MinecraftMeta {
    cache: RefreshedData<Arc<[MinecraftVersion]>>,
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

    pub async fn minecraft_versions(&self) -> Result<Arc<[MinecraftVersion]>> {
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
        eyre::ensure!(
            versions.iter().any(|version| {
                names.iter().any(|name| {
                    legacy_game_version_name(&name.0) == version.name.0
                })
            }),
            "no supplied Minecraft version exists in minecraft-meta"
        );
        Ok(())
    }

    async fn load_versions(db: &toasty::Db) -> Result<Arc<[MinecraftVersion]>> {
        let mut connection = db
            .connection()
            .await
            .wrap_err("acquiring minecraft-meta connection")?;
        let mut versions = MinecraftVersion::all()
            .exec(&mut connection)
            .await
            .wrap_err("loading Minecraft game versions")?;

        for version in &mut versions {
            version.name = MinecraftVersionName(
                legacy_game_version_name(&version.name.0).to_owned(),
            );
        }
        versions.sort_by(|a, b| {
            minecraft_version_date(b)
                .cmp(&minecraft_version_date(a))
                .then_with(|| a.name.0.cmp(&b.name.0))
        });
        Ok(versions.into())
    }
}
