use crate::api::Result;
use daedalus::minecraft::VersionManifest;
use daedalus::modded::Manifest;

/// Gets the game versions from daedalus
#[tauri::command]
pub async fn metadata_get_game_versions() -> Result<VersionManifest> {
    Ok(theseus::metadata::get_minecraft_versions().await?)
}

/// Gets the fabric versions from daedalus
#[tauri::command]
pub async fn metadata_get_fabric_versions() -> Result<Manifest> {
    Ok(theseus::metadata::get_fabric_versions().await?)
}

/// Gets the forge versions from daedalus
#[tauri::command]
pub async fn metadata_get_forge_versions() -> Result<Manifest> {
    Ok(theseus::metadata::get_forge_versions().await?)
}

/// Gets the forge versions from daedalus
#[tauri::command]
pub async fn metadata_get_quilt_versions() -> Result<Manifest> {
    Ok(theseus::metadata::get_quilt_versions().await?)
}
