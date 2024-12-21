use crate::api::Result;
use daedalus::minecraft::VersionManifest;
use daedalus::modded::Manifest;

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
pub async fn metadata_get_game_versions() -> Result<VersionManifest> {
    Ok(theseus::metadata::get_minecraft_versions().await?)
}

/// Gets the fabric versions from daedalus
#[tauri::command]
pub async fn metadata_get_loader_versions(loader: &str) -> Result<Manifest> {
    Ok(theseus::metadata::get_loader_versions(loader).await?)
}
