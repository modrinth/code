use super::profiles::*;
use std::{collections::HashSet, path::PathBuf};

use crate::{data::DataError, LAUNCHER_WORK_DIR};
use once_cell::sync;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

const SETTINGS_FILE: &str = "settings.json";
const ICONS_PATH: &str = "icons";
const METADATA_DIR: &str = "meta";
const SETTINGS_PATH_ENV: &str = "THESEUS_CONFIG_DIR";

static SETTINGS: sync::OnceCell<RwLock<Settings>> = sync::OnceCell::new();
pub const FORMAT_VERSION: u32 = 1;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Settings {
    pub memory: MemorySettings,
    pub game_resolution: WindowSize,
    pub custom_java_args: Vec<String>,
    pub java_8_path: Option<PathBuf>,
    pub java_17_path: Option<PathBuf>,
    pub hooks: ProfileHooks,
    pub icon_path: PathBuf,
    pub metadata_dir: PathBuf,
    pub profiles: HashSet<PathBuf>,
    pub max_concurrent_downloads: usize,
    pub version: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            memory: MemorySettings::default(),
            game_resolution: WindowSize::default(),
            custom_java_args: Vec::new(),
            java_8_path: None,
            java_17_path: None,
            hooks: ProfileHooks::default(),
            icon_path: LAUNCHER_WORK_DIR.join(ICONS_PATH),
            metadata_dir: LAUNCHER_WORK_DIR.join(METADATA_DIR),
            profiles: HashSet::new(),
            max_concurrent_downloads: 32,
            version: FORMAT_VERSION,
        }
    }
}

impl Settings {
    pub async fn init() -> Result<(), DataError> {
        let settings_path = std::env::var_os(SETTINGS_PATH_ENV)
            .map_or(LAUNCHER_WORK_DIR.join(SETTINGS_FILE), PathBuf::from);

        if settings_path.exists() {
            let settings_data = tokio::fs::read_to_string(&settings_path)
                .await
                .map(|x| serde_json::from_str::<Settings>(&x).ok())
                .ok()
                .flatten();

            if let Some(settings) = settings_data {
                SETTINGS.get_or_init(|| RwLock::new(settings));
            }
        }

        if SETTINGS.get().is_none() {
            let new = Self::default();

            if settings_path.exists() {
                tokio::fs::rename(SETTINGS_FILE, format!("{SETTINGS_FILE}.bak"))
                    .await?;
            }

            tokio::fs::write(
                LAUNCHER_WORK_DIR.join(SETTINGS_FILE),
                &serde_json::to_string(&new)?,
            )
            .await?;

            SETTINGS.get_or_init(|| RwLock::new(new));
        }

        Ok(())
    }

    pub async fn load() -> Result<(), DataError> {
        let new = serde_json::from_str::<Settings>(&std::fs::read_to_string(
            LAUNCHER_WORK_DIR.join(SETTINGS_FILE),
        )?)?;

        let mut write = SETTINGS
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
            LAUNCHER_WORK_DIR.join(SETTINGS_FILE),
            &serde_json::to_string_pretty(&*settings)?,
        )?;

        Ok(())
    }

    pub async fn get<'a>() -> Result<RwLockReadGuard<'a, Self>, DataError> {
        Ok(Self::get_or_uninit::<'a>()?.read().await)
    }

    pub async fn get_mut<'a>() -> Result<RwLockWriteGuard<'a, Self>, DataError>
    {
        Ok(Self::get_or_uninit::<'a>()?.write().await)
    }

    fn get_or_uninit<'a>() -> Result<&'a RwLock<Self>, DataError> {
        SETTINGS
            .get()
            .ok_or_else(|| DataError::InitializedError("settings".to_string()))
    }
}
