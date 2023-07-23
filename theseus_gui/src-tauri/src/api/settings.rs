use std::path::PathBuf;

use crate::api::Result;
use serde::{Deserialize, Serialize};
use theseus::prelude::*;

// Identical to theseus::settings::Settings except for the custom_java_args field
// This allows us to split the custom_java_args string into a Vec<String> here and join it back into a string in the backend
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FrontendSettings {
    pub theme: Theme,
    pub memory: MemorySettings,
    pub game_resolution: WindowSize,
    pub custom_java_args: String,
    pub custom_env_args: String,
    pub java_globals: JavaGlobals,
    pub default_user: Option<uuid::Uuid>,
    pub hooks: Hooks,
    pub max_concurrent_downloads: usize,
    pub max_concurrent_writes: usize,
    pub version: u32,
    pub collapsed_navigation: bool,
}

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("settings")
        .invoke_handler(tauri::generate_handler![
            settings_get,
            settings_set,
            settings_change_config_dir
        ])
        .build()
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

// Change config directory
// Seizes the entire State to do it
// invoke('plugin:settings|settings_change_config_dir', new_dir)
#[tauri::command]
pub async fn settings_change_config_dir(new_config_dir: PathBuf) -> Result<()> {
    settings::set_config_dir(new_config_dir).await?;
    Ok(())
}
