use crate::api::Result;
use serde::{Serialize, Deserialize};
use theseus::{prelude::*, window_scoped};

// Identical to theseus::settings::Settings except for the custom_java_args field
// This allows us to split the custom_java_args string into a Vec<String> here and join it back into a string in the backend
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FrontendSettings {
    pub memory: MemorySettings,
    pub game_resolution: WindowSize,
    pub custom_java_args: String,
    pub custom_env_args: Vec<(String, String)>,
    pub java_globals: JavaGlobals,
    pub default_user: Option<uuid::Uuid>,
    pub hooks: Hooks,
    pub max_concurrent_downloads: usize,
    pub version: u32,
}

// Get full settings
// invoke('settings_get')
#[tauri::command]
pub async fn settings_get(window: tauri::Window) -> Result<FrontendSettings> {
    let backend_settings = window_scoped!(window,settings::get()).await?;
    let frontend_settings = FrontendSettings {
        memory: backend_settings.memory,
        game_resolution: backend_settings.game_resolution,
        custom_java_args: backend_settings.custom_java_args.join(" "),
        custom_env_args: backend_settings.custom_env_args,
        java_globals: backend_settings.java_globals,
        default_user: backend_settings.default_user,
        hooks: backend_settings.hooks,
        max_concurrent_downloads: backend_settings.max_concurrent_downloads,
        version: backend_settings.version,
    };
    Ok(frontend_settings)
}

// Set full settings
// invoke('settings_set', settings)
#[tauri::command]
pub async fn settings_set(window: tauri::Window, settings: FrontendSettings) -> Result<()> {
    let backend_settings = Settings {
        memory: settings.memory,
        game_resolution: settings.game_resolution,
        custom_java_args: settings.custom_java_args.split_whitespace().map(|s| s.to_string()).collect(),
        custom_env_args: settings.custom_env_args,
        java_globals: settings.java_globals,
        default_user: settings.default_user,
        hooks: settings.hooks,
        max_concurrent_downloads: settings.max_concurrent_downloads,
        version: settings.version,
    };
    window_scoped!(window, settings::set(backend_settings)).await?;
    Ok(())
}
