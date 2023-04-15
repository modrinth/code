//! Theseus state management system
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt, path::PathBuf, sync::Arc};
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
    pub loading_bars: RwLock<HashMap<LoadingBarId, LoadingBar>>,
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
    pub loading_bar_uuid: LoadingBarId,
    pub message: String,
    pub total: f64,
    pub current: f64,
}

// Loading Bar Id lets us uniquely identify loading bars stored in the state
// the uuid lets us identify loading bars across threads
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct LoadingBarId {
    pub key: LoadingBarType,
    pub uuid: Uuid,
}

impl LoadingBarId {
    pub fn new(key: LoadingBarType) -> Self {
        Self {
            key,
            uuid: Uuid::new_v4(),
        }
    }
}

impl fmt::Display for LoadingBarId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}-{}", self.key, self.uuid)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub enum LoadingBarType {
    StateInit,
    PackDownload,
    MinecraftDownload,
    ProfileSync,
}

#[derive(Serialize, Clone)]
pub struct LoadingPayload {
    pub event_type: LoadingBarType,
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
#[derive(Serialize, Clone)]
pub enum ProcessPayloadType {
    Launched,
    // Finishing, // TODO: process restructing incoming, currently this is never emitted
    // Finished, // TODO: process restructing incoming, currently this is never emitted
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
    NoLoadingBar(LoadingBarId),

    #[cfg(feature = "tauri")]
    #[error("Tauri error: {0}")]
    TauriError(#[from] tauri::Error),
}
