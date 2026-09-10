use crate::api::Result;
use tauri::Runtime;
use theseus::prelude::*;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("settings")
        .invoke_handler(tauri::generate_handler![
            settings_get,
            settings_set,
            store_usage,
            store_cleanup,
            store_set_cache_limit,
            store_verify,
            cancel_directory_change
        ])
        .build()
}

#[tauri::command]
pub async fn store_usage() -> Result<settings::StoreUsage> {
    Ok(settings::store_usage().await?)
}

#[tauri::command]
pub async fn store_cleanup() -> Result<u64> {
    Ok(settings::store_cleanup().await?)
}

#[tauri::command]
pub async fn store_set_cache_limit(bytes: u64) -> Result<()> {
    Ok(settings::store_set_cache_limit(bytes).await?)
}

#[tauri::command]
pub async fn store_verify(repair: bool) -> Result<settings::StoreVerification> {
    Ok(settings::store_verify(repair).await?)
}

// Get full settings
// invoke('plugin:settings|settings_get')
#[tauri::command]
pub async fn settings_get() -> Result<Settings> {
    let res = settings::get().await?;
    Ok(res)
}

// Set full settings
// invoke('plugin:settings|settings_set', settings)
#[tauri::command]
pub async fn settings_set(settings: Settings) -> Result<()> {
    settings::set(settings).await?;
    Ok(())
}

#[tauri::command]
pub async fn cancel_directory_change<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<()> {
    let identifier = &app.config().identifier;
    settings::cancel_directory_change(identifier).await?;
    Ok(())
}
