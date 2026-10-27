use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use eyre::{Result, WrapErr};
use minecraft_meta_model::model::{MinecraftVersion, MinecraftVersionName};
use tokio::sync::{Mutex, RwLock};

use crate::database::MinecraftMetaDb;
use crate::models::minecraft::{
    legacy_game_version_name, minecraft_version_date,
};

const CACHE_EXPIRY: Duration = Duration::from_secs(30 * 60);

/// Shared Minecraft metadata refreshed on demand after thirty minutes.
pub struct MinecraftMeta {
    db: Mutex<toasty::Db>,
    cache: RwLock<Arc<[MinecraftVersion]>>,
    clock_origin: Instant,
    /// Expiration in elapsed seconds from the monotonic clock origin.
    expires_at: AtomicU64,
}

impl MinecraftMeta {
    pub async fn new(db: MinecraftMetaDb) -> Result<Self> {
        let versions = Self::load_versions(&db.0).await?;
        Ok(Self {
            db: Mutex::new(db.0),
            cache: RwLock::new(versions),
            clock_origin: Instant::now(),
            expires_at: AtomicU64::new(CACHE_EXPIRY.as_secs()),
        })
    }

    pub async fn minecraft_versions(&self) -> Result<Arc<[MinecraftVersion]>> {
        if !self.is_expired() {
            return Ok(Arc::clone(&*self.cache.read().await));
        }

        let db = self.db.lock().await;
        if !self.is_expired() {
            return Ok(Arc::clone(&*self.cache.read().await));
        }

        let versions = Self::load_versions(&db).await?;
        let mut cache = self.cache.write().await;
        *cache = Arc::clone(&versions);
        self.expires_at.store(
            self.clock_origin
                .elapsed()
                .as_secs()
                .saturating_add(CACHE_EXPIRY.as_secs()),
            Ordering::Release,
        );
        Ok(versions)
    }

    fn is_expired(&self) -> bool {
        self.clock_origin.elapsed().as_secs()
            >= self.expires_at.load(Ordering::Acquire)
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
