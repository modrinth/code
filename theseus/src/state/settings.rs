//! Theseus settings file
use crate::{jre, State};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

use super::JavaGlobals;

// TODO: convert to semver?
const CURRENT_FORMAT_VERSION: u32 = 1;

// Types
/// Global Theseus settings
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Settings {
    pub theme: Theme,
    pub memory: MemorySettings,
    pub game_resolution: WindowSize,
    pub custom_java_args: Vec<String>,
    pub custom_env_args: Vec<(String, String)>,
    pub java_globals: JavaGlobals,
    pub default_user: Option<uuid::Uuid>,
    pub hooks: Hooks,
    pub max_concurrent_downloads: usize,
    pub max_concurrent_writes: usize,
    pub version: u32,
    pub collapsed_navigation: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            memory: MemorySettings::default(),
            game_resolution: WindowSize::default(),
            custom_java_args: Vec::new(),
            custom_env_args: Vec::new(),
            java_globals: JavaGlobals::new(),
            default_user: None,
            hooks: Hooks::default(),
            max_concurrent_downloads: 10,
            max_concurrent_writes: 10,
            version: CURRENT_FORMAT_VERSION,
            collapsed_navigation: false,
        }
    }
}

impl Settings {
    #[tracing::instrument]
    pub async fn init(file: &Path) -> crate::Result<Self> {
        if file.exists() {
            fs::read(&file)
                .await
                .map_err(|err| {
                    crate::ErrorKind::FSError(format!(
                        "Error reading settings file: {err}"
                    ))
                    .as_error()
                })
                .and_then(|it| {
                    serde_json::from_slice::<Settings>(&it)
                        .map_err(crate::Error::from)
                })
        } else {
            Ok(Settings::default())
        }
    }

    #[tracing::instrument]
    #[theseus_macros::debug_pin]
    pub async fn update_java() {
        let res = async {
            let state = State::get().await?;
            let settings_read = state.settings.write().await;

            if settings_read.java_globals.count() == 0 {
                drop(settings_read);
                let java_globals = jre::autodetect_java_globals().await?;
                state.settings.write().await.java_globals = java_globals;
            }

            Ok::<(), crate::Error>(())
        }
        .await;

        match res {
            Ok(()) => {}
            Err(err) => {
                tracing::warn!("Unable to update launcher java: {err}")
            }
        };
    }

    #[tracing::instrument(skip(self))]
    pub async fn sync(&self, to: &Path) -> crate::Result<()> {
        fs::write(to, serde_json::to_vec(self)?)
            .await
            .map_err(|err| {
                crate::ErrorKind::FSError(format!(
                    "Error saving settings to file: {err}"
                ))
                .as_error()
            })
    }
}

/// Theseus theme
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Dark,
    Light,
    Oled,
}

/// Minecraft memory settings
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct MemorySettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<u32>,
    pub maximum: u32,
}

impl Default for MemorySettings {
    fn default() -> Self {
        Self {
            minimum: None,
            maximum: 2048,
        }
    }
}

/// Game window size
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct WindowSize(pub u16, pub u16);

impl Default for WindowSize {
    fn default() -> Self {
        Self(854, 480)
    }
}

/// Game initialization hooks
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct Hooks {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_launch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrapper: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_exit: Option<String>,
}
