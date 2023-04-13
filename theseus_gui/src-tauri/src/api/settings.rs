use crate::api::Result;
use serde::{Deserialize, Serialize};
use theseus::prelude::*;

use super::TheseusSerializableError;

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
    pub version: u32,
}

// Get full settings
// invoke('settings_get')
#[tauri::command]
pub async fn settings_get() -> Result<FrontendSettings> {
    let backend_settings = settings::get().await?;
    let frontend_settings = FrontendSettings {
        theme: backend_settings.theme,
        memory: backend_settings.memory,
        game_resolution: backend_settings.game_resolution,
        custom_java_args: backend_settings.custom_java_args.join(" "),
        custom_env_args: backend_settings
            .custom_env_args
            .into_iter()
            .map(|(s1, s2)| format!("{s1}={s2}"))
            .collect::<Vec<String>>()
            .join(" "),
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
pub async fn settings_set(settings: FrontendSettings) -> Result<()> {
    let custom_env_args: Vec<(String, String)> = settings
        .custom_env_args
        .split_whitespace()
        .map(|s| s.to_string())
        .map(|f| {
            let mut split = f.split('=');
            if let (Some(name), Some(value)) = (split.next(), split.next()) {
                Ok((name.to_string(), value.to_string()))
            } else {
                Err(TheseusSerializableError::BadEnvVars(
                    "Invalid environment variable: {}".to_string(),
                )
                .into())
            }
        })
        .collect::<Result<Vec<(String, String)>>>()?;

    let backend_settings = Settings {
        theme: settings.theme,
        memory: settings.memory,
        game_resolution: settings.game_resolution,
        custom_java_args: settings
            .custom_java_args
            .split_whitespace()
            .map(|s| s.to_string())
            .collect(),
        custom_env_args,
        java_globals: settings.java_globals,
        default_user: settings.default_user,
        hooks: settings.hooks,
        max_concurrent_downloads: settings.max_concurrent_downloads,
        version: settings.version,
    };
    settings::set(backend_settings).await?;
    Ok(())
}
