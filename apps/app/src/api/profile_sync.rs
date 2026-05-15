use crate::api::Result;
use pteron::prelude::InstanceSyncOverrides;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("profile-sync")
        .invoke_handler(tauri::generate_handler![
            profile_set_sync_enabled,
            profile_set_sync_overrides
        ])
        .build()
}

#[tauri::command]
pub async fn profile_set_sync_enabled(path: String, enabled: bool) -> Result<()> {
    pteron::profile::sync::set_sync_enabled(&path, enabled).await?;
    Ok(())
}

#[tauri::command]
pub async fn profile_set_sync_overrides(
    path: String,
    overrides: Option<InstanceSyncOverrides>,
) -> Result<()> {
    pteron::profile::sync::set_sync_overrides(&path, overrides).await?;
    Ok(())
}
