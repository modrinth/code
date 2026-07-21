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
pub async fn metadata_get_game_versions(
    invocation_context: theseus::InvocationContext,
) -> Result<VersionManifest> {
    let context = crate::api::operation_context(invocation_context);
    Ok(theseus::metadata::get_minecraft_versions(&context).await?)
}

/// Gets the fabric versions from daedalus
#[tauri::command]
pub async fn metadata_get_loader_versions(
    loader: &str,
    invocation_context: theseus::InvocationContext,
) -> Result<Manifest> {
    let context = crate::api::operation_context(invocation_context);
    Ok(theseus::metadata::get_loader_versions(&context, loader).await?)
}
