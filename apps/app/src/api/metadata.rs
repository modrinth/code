use crate::api::Result;
use daedalus::minecraft::VersionManifest;
use daedalus::modded::Manifest;
use theseus::prelude::CacheBehaviour;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("metadata")
        .invoke_handler(tauri::generate_handler![
            metadata_get_game_versions,
            metadata_get_loader_versions,
        ])
        .build()
}

/// Gets the game versions from daedalus
#[tauri::command]
pub async fn metadata_get_game_versions(
    cache_behaviour: Option<CacheBehaviour>,
) -> Result<VersionManifest> {
    Ok(theseus::metadata::get_minecraft_versions(cache_behaviour).await?)
}

/// Gets the fabric versions from daedalus
#[tauri::command]
pub async fn metadata_get_loader_versions(
    loader: &str,
    cache_behaviour: Option<CacheBehaviour>,
) -> Result<Manifest> {
    Ok(theseus::metadata::get_loader_versions(loader, cache_behaviour).await?)
}
