pub use crate::launcher::profile::{
    LoaderManifestMetadata, ResolvedVersion, loader_manifest_metadata,
    loader_manifest_metadata_from_cache_key, merge_partial_version,
};
use crate::state::CachedEntry;
use crate::{ErrorKind, State};
pub use minecraft_meta_manifest::fabriclike::Manifest;
pub use minecraft_meta_manifest::minecraft::Catalog as VersionManifest;

#[tracing::instrument]
pub async fn get_minecraft_versions() -> crate::Result<VersionManifest> {
    let state = State::get().await?;
    let minecraft_versions = CachedEntry::get_minecraft_manifest(
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await?
    .ok_or(ErrorKind::NoValueFor("minecraft versions".to_string()))?;

    Ok(minecraft_versions)
}

// #[tracing::instrument]
pub async fn get_loader_versions(loader: &str) -> crate::Result<Manifest> {
    let state = State::get().await?;
    let cache_key = loader_manifest_metadata(loader).cache_key;
    let loaders = CachedEntry::get_loader_manifest(
        &cache_key,
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await?
    .ok_or(ErrorKind::NoValueFor(format!("{loader} loader versions")))?;

    Ok(loaders.manifest)
}
