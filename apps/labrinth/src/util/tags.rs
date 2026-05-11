use crate::database::PgPool;
use crate::database::models::legacy_loader_fields::MinecraftGameVersion;
use crate::database::models::loader_fields::Loader;
use crate::database::redis::RedisPool;
use crate::routes::ApiError;
use crate::util::error::Context;
use arc_swap::ArcSwapOption;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Cached set of valid loaders and game version tags.
///
/// Fetched using [`valid_download_tags`].
#[derive(Debug)]
pub struct DownloadTagsCache {
    expires: Instant,
    pub loaders: HashSet<String>,
    pub game_versions: HashSet<String>,
}

/// Fetches download tags from the database or returns a cached version.
///
/// We cache tags since we get a large volume of download ingests, and querying
/// the database or even Redis for each request is too expensive.
pub async fn valid_download_tags(
    pool: &PgPool,
    redis: &RedisPool,
) -> Result<Arc<DownloadTagsCache>, ApiError> {
    const DOWNLOAD_TAGS_CACHE_TTL: Duration = Duration::from_secs(60 * 5);

    static DOWNLOAD_TAGS_CACHE: ArcSwapOption<DownloadTagsCache> =
        ArcSwapOption::const_empty();
    static DOWNLOAD_TAGS_CACHE_REFRESH_LOCK: Mutex<()> = Mutex::const_new(());

    let now = Instant::now();
    let cached = DOWNLOAD_TAGS_CACHE.load();
    if let Some(cached) = &*cached
        && cached.expires > now
    {
        return Ok(cached.clone());
    }

    let _refresh_lock = DOWNLOAD_TAGS_CACHE_REFRESH_LOCK.lock().await;

    let now = Instant::now();
    let cached = DOWNLOAD_TAGS_CACHE.load();
    if let Some(cached) = &*cached
        && cached.expires > now
    {
        return Ok(cached.clone());
    }

    let loaders = Loader::list(pool, redis)
        .await
        .wrap_internal_err("failed to fetch loaders")?
        .into_iter()
        .map(|loader| loader.loader)
        .collect();
    let game_versions = MinecraftGameVersion::list(None, None, pool, redis)
        .await
        .wrap_internal_err("failed to fetch game versions")?
        .into_iter()
        .map(|game_version| game_version.version)
        .collect();

    let cache = Arc::new(DownloadTagsCache {
        expires: now + DOWNLOAD_TAGS_CACHE_TTL,
        loaders,
        game_versions,
    });
    DOWNLOAD_TAGS_CACHE.store(Some(cache.clone()));

    Ok(cache)
}
