use crate::api::Result;
use theseus::prelude::*;

// Get full settings
// invoke('settings_get')
#[tauri::command]
pub async fn settings_get() -> Result<Settings> {
    Ok(settings::get().await?)
}

// Set full settings
// invoke('settings_set', settings)
#[tauri::command]
pub async fn settings_set(settings: Settings) -> Result<()> {
    settings::set(settings).await?;
    Ok(())
}
