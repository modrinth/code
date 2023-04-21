//! Theseus state management system
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::sync::OnceCell;
use tokio::sync::RwLock;
use uuid::Uuid;

pub mod emit;

// Global event state
// Stores the Tauri app handle and other event-related state variables
static EVENT_STATE: OnceCell<Arc<EventState>> = OnceCell::const_new();
pub struct EventState {
    /// Tauri app
    #[cfg(feature = "tauri")]
    pub app: tauri::AppHandle,
    pub loading_bars: RwLock<HashMap<Uuid, LoadingBar>>,
}

impl EventState {
    #[cfg(feature = "tauri")]
    pub async fn init(app: tauri::AppHandle) -> crate::Result<Arc<Self>> {
        EVENT_STATE
            .get_or_try_init(|| async {
                Ok(Arc::new(Self {
                    app,
                    loading_bars: RwLock::new(HashMap::new()),
                }))
            })
            .await
            .map(Arc::clone)
    }

    #[cfg(not(feature = "tauri"))]
    pub async fn init() -> crate::Result<Arc<Self>> {
        EVENT_STATE
            .get_or_try_init(|| async {
                Ok(Arc::new(Self {
                    loading_bars: RwLock::new(HashMap::new()),
                }))
            })
            .await
            .map(Arc::clone)
    }

    #[cfg(feature = "tauri")]
    pub async fn get() -> crate::Result<Arc<Self>> {
        Ok(EVENT_STATE.get().ok_or(EventError::NotInitialized)?.clone())
    }

    // Initialization requires no app handle in non-tauri mode, so we can just use the same function
    #[cfg(not(feature = "tauri"))]
    pub async fn get() -> crate::Result<Arc<Self>> {
        Self::init().await
    }
}

#[derive(Debug, Clone)]
pub struct LoadingBar {
    pub loading_bar_id: Uuid,
    pub message: String,
    pub total: f64,
    pub current: f64,
    pub bar_type: LoadingBarType,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum LoadingBarType {
    StateInit,
    PackDownload {
        pack_name: String,
        pack_id: Option<String>,
        pack_version: Option<String>,
    },
    MinecraftDownload {
        profile_uuid: Uuid,
        profile_name: String,
    },
    ProfileSync,
    LauncherSync,
}

#[derive(Serialize, Clone)]
pub struct LoadingPayload {
    pub event: LoadingBarType,
    pub loader_uuid: Uuid,
    pub fraction: Option<f64>, // by convention, if optional, it means the loading is done
    pub message: String,
}

#[derive(Serialize, Clone)]
pub struct WarningPayload {
    pub message: String,
}

#[derive(Serialize, Clone)]
pub struct ProcessPayload {
    pub uuid: Uuid, // processes in state are going to be identified by UUIDs, as they might change to different processes
    pub pid: u32,
    pub event: ProcessPayloadType,
    pub message: String,
}
#[derive(Serialize, Clone, Debug)]
pub enum ProcessPayloadType {
    Launched,
    Updated, // eg: if the MinecraftChild changes to its post-command process instead of the Minecraft process
    Finished,
}

#[derive(Serialize, Clone)]
pub struct ProfilePayload {
    pub uuid: Uuid,
    pub path: PathBuf,
    pub name: String,
    pub event: ProfilePayloadType,
}
#[derive(Serialize, Clone)]
pub enum ProfilePayloadType {
    Created,
    Added, // also triggered when Created
    Edited,
    Removed,
}

#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Event state was not properly initialized")]
    NotInitialized,

    #[error("Non-existent loading bar of key: {0}")]
    NoLoadingBar(Uuid),

    #[cfg(feature = "tauri")]
    #[error("Tauri error: {0}")]
    TauriError(#[from] tauri::Error),
}
