use std::path::Path;

use crate::{data::DataError, LAUNCHER_WORK_DIR};
use once_cell::sync;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, RwLockReadGuard};

const SETTINGS_FILE: &str = "settings.json";

static SETTINGS: sync::OnceCell<RwLock<Settings>> = sync::OnceCell::new();

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Settings {
    pub memory: i32,
    pub game_resolution: (i32, i32),
    pub custom_java_args: String,
    pub java_8_path: Option<String>,
    pub java_17_path: Option<String>,
    pub wrapper_command: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            memory: 2048,
            game_resolution: (854, 480),
            custom_java_args: "".to_string(),
            java_8_path: None,
            java_17_path: None,
            wrapper_command: None,
        }
    }
}

impl Settings {
    pub async fn init() -> Result<(), DataError> {
        let settings_path = Path::new(LAUNCHER_WORK_DIR).join(SETTINGS_FILE);

        if settings_path.exists() {
            let settings_data = std::fs::read_to_string(settings_path)
                .map(|x| serde_json::from_str::<Settings>(&*x).ok())
                .ok()
                .flatten();

            if let Some(settings) = settings_data {
                SETTINGS.get_or_init(|| RwLock::new(settings));
            }
        }

        if SETTINGS.get().is_none() {
            let new = Self::default();

            std::fs::write(
                Path::new(LAUNCHER_WORK_DIR).join(SETTINGS_FILE),
                &*serde_json::to_string(&new)?,
            )?;

            SETTINGS.get_or_init(|| RwLock::new(new));
        }

        Ok(())
    }

    pub async fn load() -> Result<(), DataError> {
        let new = serde_json::from_str::<Settings>(&std::fs::read_to_string(
            Path::new(LAUNCHER_WORK_DIR).join(SETTINGS_FILE),
        )?)?;

        let write = &mut *SETTINGS
            .get()
            .ok_or_else(|| DataError::InitializedError("settings".to_string()))?
            .write()
            .await;

        *write = new;

        Ok(())
    }

    pub async fn save() -> Result<(), DataError> {
        let settings = Self::get().await?;

        std::fs::write(
            Path::new(LAUNCHER_WORK_DIR).join(SETTINGS_FILE),
            &serde_json::to_string(&*settings)?,
        )?;

        Ok(())
    }

    pub async fn get<'a>() -> Result<RwLockReadGuard<'a, Self>, DataError> {
        Ok(SETTINGS
            .get()
            .ok_or_else(|| DataError::InitializedError("settings".to_string()))?
            .read()
            .await)
    }
}
