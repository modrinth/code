//! Theseus state management system
use ariadne::ids::UserId;
use ariadne::users::UserStatus;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};
#[cfg(feature = "tauri")]
use tauri::Emitter;
use tokio::sync::OnceCell;
use uuid::Uuid;

pub mod emit;

// Global event state
// Stores the Tauri app handle and other event-related state variables
static EVENT_STATE: OnceCell<Arc<EventState>> = OnceCell::const_new();
pub struct EventState {
    /// Tauri app
    #[cfg(feature = "tauri")]
    pub app: tauri::AppHandle,
    pub loading_bars: DashMap<Uuid, LoadingBar>,
}

impl EventState {
    #[cfg(feature = "tauri")]
    pub async fn init(app: tauri::AppHandle) -> crate::Result<Arc<Self>> {
        EVENT_STATE
            .get_or_try_init(|| async {
                Ok(Arc::new(Self {
                    app,
                    loading_bars: DashMap::new(),
                }))
            })
            .await
            .cloned()
    }

    #[cfg(not(feature = "tauri"))]
    pub async fn init() -> crate::Result<Arc<Self>> {
        EVENT_STATE
            .get_or_try_init(|| async {
                Ok(Arc::new(Self {
                    loading_bars: DashMap::new(),
                }))
            })
            .await
            .cloned()
    }

    pub fn get() -> crate::Result<Arc<Self>> {
        Ok(EVENT_STATE.get().ok_or(EventError::NotInitialized)?.clone())
    }

    // Values provided should not be used directly, as they are clones and are not guaranteed to be up-to-date
    pub async fn list_progress_bars() -> crate::Result<DashMap<Uuid, LoadingBar>>
    {
        let value = Self::get()?;
        Ok(value.loading_bars.clone())
    }

    #[cfg(feature = "tauri")]
    pub async fn get_main_window() -> crate::Result<Option<tauri::WebviewWindow>>
    {
        use tauri::Manager;
        let value = Self::get()?;
        Ok(value.app.get_webview_window("main"))
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct LoadingBar {
    // loading_bar_uuid not be used directly by external functions as it may not reflect the current state of the loading bar/hashmap
    pub loading_bar_uuid: Uuid,
    pub message: String,
    pub total: f64,
    pub current: f64,
    #[serde(skip)]
    pub last_sent: f64,
    pub bar_type: LoadingBarType,
    #[cfg(feature = "cli")]
    #[serde(skip)]
    pub cli_progress_bar: indicatif::ProgressBar,
}

#[derive(Serialize, Debug, Clone)]
pub struct LoadingBarId(Uuid);

// When Loading bar id is dropped, we should remove it from the hashmap
impl Drop for LoadingBarId {
    fn drop(&mut self) {
        let loader_uuid = self.0;
        tokio::spawn(async move {
            if let Ok(event_state) = EventState::get() {
                #[cfg(any(feature = "tauri", feature = "cli"))]
                if let Some((_, bar)) =
                    event_state.loading_bars.remove(&loader_uuid)
                {
                    #[cfg(feature = "tauri")]
                    {
                        let loader_uuid = bar.loading_bar_uuid;
                        let event = bar.bar_type.clone();
                        let fraction = bar.current / bar.total;

                        let _ = event_state.app.emit(
                            "loading",
                            LoadingPayload {
                                fraction: None,
                                message: "Completed".to_string(),
                                event,
                                loader_uuid,
                            },
                        );
                        tracing::trace!(
                            "Exited at {fraction} for loading bar: {:?}",
                            loader_uuid
                        );
                    }

                    // Emit event to indicatif progress bar arc
                    #[cfg(feature = "cli")]
                    {
                        let cli_progress_bar = bar.cli_progress_bar;
                        cli_progress_bar.finish();
                    }
                }

                #[cfg(not(any(feature = "tauri", feature = "cli")))]
                event_state.loading_bars.remove(&loader_uuid);
            }
        });
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum LoadingBarType {
    LegacyDataMigration,
    DirectoryMove {
        old: PathBuf,
        new: PathBuf,
    },
    JavaDownload {
        version: u32,
    },
    PackFileDownload {
        profile_path: String,
        pack_name: String,
        icon: Option<String>,
        pack_version: String,
    },
    PackDownload {
        profile_path: String,
        pack_name: String,
        icon: Option<PathBuf>,
        pack_id: Option<String>,
        pack_version: Option<String>,
    },
    MinecraftDownload {
        profile_path: String,
        profile_name: String,
    },
    ProfileUpdate {
        profile_path: String,
        profile_name: String,
    },
    ZipExtract {
        profile_path: String,
        profile_name: String,
    },
    ConfigChange {
        new_path: PathBuf,
    },
    CopyProfile {
        import_location: PathBuf,
        profile_name: String,
    },
    CheckingForUpdates,
    LauncherUpdate {
        version: String,
        current_version: String,
    },
}

#[derive(Serialize, Clone)]
pub struct LoadingPayload {
    pub event: LoadingBarType,
    pub loader_uuid: Uuid,
    pub fraction: Option<f64>, // by convention, if optional, it means the loading is done
    pub message: String,
}

#[derive(Serialize, Clone)]
pub struct OfflinePayload {
    pub offline: bool,
}

#[derive(Serialize, Clone)]
pub struct WarningPayload {
    pub message: String,
}

#[derive(Serialize, Clone)]
#[serde(tag = "event")]
pub enum CommandPayload {
    InstallMod {
        id: String,
    },
    InstallVersion {
        id: String,
    },
    InstallModpack {
        id: String,
    },
    RunMRPack {
        // run or install .mrpack
        path: PathBuf,
    },
}

#[derive(Serialize, Clone)]
pub struct ProcessPayload {
    pub profile_path_id: String,
    pub uuid: Uuid,
    pub event: ProcessPayloadType,
    pub message: String,
}
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ProcessPayloadType {
    Launched,
    Finished,
}

#[derive(Serialize, Clone)]
pub struct ProfilePayload {
    pub profile_path_id: String,
    #[serde(flatten)]
    pub event: ProfilePayloadType,
}
#[derive(Serialize, Clone)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum ProfilePayloadType {
    Created,
    Synced,
    ServersUpdated,
    WorldUpdated {
        world: String,
    },
    ServerJoined {
        host: String,
        port: u16,
        timestamp: DateTime<Utc>,
    },
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

#[derive(Serialize, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "event")]
pub enum FriendPayload {
    FriendRequest { from: UserId },
    UserOffline { id: UserId },
    StatusUpdate { user_status: UserStatus },
    StatusSync,
}
