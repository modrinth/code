//! Theseus state management system
use dashmap::DashMap;
#[cfg(feature = "tauri")]
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
#[cfg(feature = "export-ts")]
use std::path::PathBuf;
use std::sync::Arc;
#[cfg(feature = "tauri")]
use tauri::ipc::{Channel, InvokeResponseBody};
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::install::InstallJobSnapshot;

pub mod emit;

// Global event state
// Stores the Tauri app handle and other event-related state variables
static EVENT_STATE: OnceCell<Arc<EventState>> = OnceCell::const_new();
pub struct EventState {
    /// Tauri app
    #[cfg(feature = "tauri")]
    pub app: tauri::AppHandle,
    #[cfg(feature = "tauri")]
    event_channel: RwLock<Channel<InvokeResponseBody>>,
    pub loading_bars: DashMap<Uuid, LoadingBar>,
}

impl EventState {
    #[cfg(feature = "tauri")]
    pub async fn init(
        app: tauri::AppHandle,
        event_channel: Channel<InvokeResponseBody>,
    ) -> crate::Result<Arc<Self>> {
        let state = EVENT_STATE
            .get_or_try_init(|| async {
                Ok::<_, crate::Error>(Arc::new(Self {
                    app,
                    event_channel: RwLock::new(event_channel.clone()),
                    loading_bars: DashMap::new(),
                }))
            })
            .await
            .cloned()?;

        *state.event_channel.write() = event_channel;
        Ok(state)
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

    pub fn get() -> Arc<Self> {
        EVENT_STATE
            .get()
            .expect("should be initialized when used")
            .clone()
    }

    #[cfg(feature = "tauri")]
    pub fn send(&self, event: AppEvent) -> crate::Result<()> {
        let payload =
            postcard::to_allocvec(&event).map_err(EventError::from)?;
        self.event_channel
            .read()
            .send(InvokeResponseBody::Raw(payload))
            .map_err(EventError::from)?;
        Ok(())
    }

    // Values provided should not be used directly, as they are clones and are not guaranteed to be up-to-date
    pub async fn list_progress_bars() -> crate::Result<DashMap<Uuid, LoadingBar>>
    {
        let value = Self::get();
        Ok(value.loading_bars.clone())
    }

    #[cfg(feature = "tauri")]
    pub async fn get_main_window() -> crate::Result<Option<tauri::WebviewWindow>>
    {
        use tauri::Manager;
        let value = Self::get();
        Ok(value.app.get_webview_window("main"))
    }
}

#[derive(Clone)]
#[cfg_attr(
    feature = "export-ts",
    derive(ts_rs::TS, postcard_bindgen::PostcardBindings)
)]
#[serde_binhum::serde_binhum]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
#[cfg_attr(
    feature = "export-ts",
    ts(
        tag = "type",
        content = "payload",
        rename_all = "snake_case",
        export_to = "AppEvent.ts"
    )
)]
pub enum AppEvent {
    Loading(LoadingPayload),
    Process(ProcessPayload),
    Instance(InstancePayload),
    InstanceGroupsChanged(InstanceGroupsChangedPayload),
    OnboardingChecklist(crate::state::OnboardingChecklist),
    InstanceBulkUpdateProgress(InstanceBulkUpdateProgressPayload),
    InstallJob(std::sync::Arc<InstallJobSnapshot>),
    Command(CommandPayload),
    Warning(WarningPayload),
    Friend(FriendPayload),
    Notification(
        #[cfg_attr(feature = "export-ts", ts(type = "unknown"))] String,
    ),
    Log(LogPayload),
    AdsConsentRequired(bool),
}

#[cfg(feature = "export-ts")]
pub fn export_app_event_bindings(
    output: impl Into<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    use postcard_bindgen::{PackageInfo, generate_bindings, javascript};
    use ts_rs::{Config, TS};

    let output = output.into();
    let config = Config::default()
        .with_out_dir(output.clone())
        .with_large_int("number");
    AppEvent::export_all(&config)?;

    javascript::build_package(
        &output,
        PackageInfo {
            name: "postcard".into(),
            version: "0.0.0".try_into()?,
        },
        javascript::GenerationSettings::default()
            .type_script_types(true)
            .module_structure(false)
            .esm_module(true),
        generate_bindings!(
            AppEvent,
            LoadingBarType,
            LoadingPayload,
            WarningPayload,
            InstanceBulkUpdateProgressPayload,
            InstanceBulkUpdateProgressStage,
            CommandPayload,
            ProcessPayload,
            ProcessPayloadType,
            InstancePayload,
            InstancePayloadType,
            InstanceGroupsChangedPayload,
            crate::state::OnboardingChecklist,
            FriendPayload,
            FriendStatusPayload,
            LogEvent,
            LogPayload,
            crate::state::Log4jEvent,
            crate::install::InstallJobSnapshot,
            crate::install::InstallJobKind,
            crate::install::InstallJobStatus,
            crate::install::model::InstallTarget,
            crate::install::InstallPhaseId,
            crate::install::InstallProgress,
            crate::install::InstallProgressSecondary,
            crate::install::InstallPhaseDetails,
            crate::install::InstallJavaStep,
            crate::install::model::InstallJobDisplay,
            crate::install::InstallErrorView,
            crate::install::model::InstallApiErrorDetails,
            crate::install::InstallErrorContext,
            crate::api::pack::import::ImportLauncherType,
            crate::state::ModLoader,
            crate::SharedInstanceUnavailableReason
        ),
    )?;
    fix_postcard_javascript_utf8(&output.join("postcard/index.js"))?;
    Ok(())
}

#[cfg(feature = "export-ts")]
fn fix_postcard_javascript_utf8(
    output: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    const GENERATED: &str = "deserialize_string = () => { const str = this.pop_n(Number(this.try_take(U32_BYTES))); return String.fromCharCode(...str) }";
    const UTF8: &str = "deserialize_string = () => new TextDecoder().decode(new Uint8Array(this.pop_n(Number(this.try_take(U32_BYTES)))))";

    let javascript = std::fs::read_to_string(output)?;
    if !javascript.contains(GENERATED) {
        return Err(
            "postcard-bindgen's generated string decoder changed".into()
        );
    }
    std::fs::write(output, javascript.replace(GENERATED, UTF8))?;
    Ok(())
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
            let event_state = EventState::get();
            #[cfg(any(feature = "tauri", feature = "cli"))]
            if let Some((_, bar)) =
                event_state.loading_bars.remove(&loader_uuid)
            {
                #[cfg(feature = "tauri")]
                {
                    let loader_uuid = bar.loading_bar_uuid;
                    let event = bar.bar_type.clone();
                    let fraction = bar.current / bar.total;

                    let _ =
                        event_state.send(AppEvent::Loading(LoadingPayload {
                            fraction: None,
                            message: "Completed".to_string(),
                            event,
                            loader_uuid: loader_uuid.to_string(),
                        }));
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
        });
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(
    feature = "export-ts",
    derive(ts_rs::TS, postcard_bindgen::PostcardBindings)
)]
#[serde_binhum::serde_binhum]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "export-ts", ts(tag = "type", rename_all = "snake_case"))]
pub enum LoadingBarType {
    LegacyDataMigration,
    DirectoryMove {
        old: String,
        new: String,
    },
    JavaDownload {
        version: u32,
    },
    PackFileDownload {
        instance_id: String,
        pack_name: String,
        icon: Option<String>,
        pack_version: String,
    },
    PackDownload {
        instance_id: String,
        pack_name: String,
        icon: Option<String>,
        pack_id: Option<String>,
        pack_version: Option<String>,
    },
    PackImport {
        pack_name: String,
    },
    MinecraftDownload {
        instance_id: String,
        instance_name: String,
    },
    InstanceUpdate {
        instance_id: String,
        instance_name: String,
    },
    ZipExtract {
        instance_id: String,
        instance_name: String,
    },
    PackExport {
        instance_id: String,
        instance_name: String,
    },
    ConfigChange {
        new_path: String,
    },
    CopyInstance {
        import_location: String,
        instance_name: String,
    },
    LauncherUpdate {
        version: String,
        current_version: String,
    },
}

#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(
    feature = "export-ts",
    derive(ts_rs::TS, postcard_bindgen::PostcardBindings)
)]
pub struct LoadingPayload {
    pub event: LoadingBarType,
    pub loader_uuid: String,
    pub fraction: Option<f64>, // by convention, if optional, it means the loading is done
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(
    feature = "export-ts",
    derive(ts_rs::TS, postcard_bindgen::PostcardBindings)
)]
pub struct WarningPayload {
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(
    feature = "export-ts",
    derive(ts_rs::TS, postcard_bindgen::PostcardBindings)
)]
#[serde(rename_all = "camelCase")]
pub struct InstanceBulkUpdateProgressPayload {
    pub instance_id: String,
    pub stage: InstanceBulkUpdateProgressStage,
    pub current: usize,
    pub total: usize,
}

#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(
    feature = "export-ts",
    derive(ts_rs::TS, postcard_bindgen::PostcardBindings)
)]
#[serde(rename_all = "snake_case")]
pub enum InstanceBulkUpdateProgressStage {
    ResolvingVersions,
    Downloading,
    Finishing,
}

#[derive(Clone)]
#[cfg_attr(
    feature = "export-ts",
    derive(ts_rs::TS, postcard_bindgen::PostcardBindings)
)]
#[serde_binhum::serde_binhum]
#[serde(tag = "event")]
#[cfg_attr(feature = "export-ts", ts(tag = "event"))]
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
    InstallServer {
        id: String,
    },
    LaunchInstance {
        id: String,
        server: Option<String>,
        singleplayer_world: Option<String>,
    },
    InstallSharedInstanceInvite {
        invite_id: String,
    },
    RunMRPack {
        // run or install .mrpack
        path: String,
    },
}

#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(
    feature = "export-ts",
    derive(ts_rs::TS, postcard_bindgen::PostcardBindings)
)]
pub struct ProcessPayload {
    pub instance_id: String,
    pub uuid: String,
    pub event: ProcessPayloadType,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(
    feature = "export-ts",
    derive(ts_rs::TS, postcard_bindgen::PostcardBindings)
)]
#[serde(rename_all = "snake_case")]
pub enum ProcessPayloadType {
    Launched,
    Finished,
}

#[derive(Clone)]
#[cfg_attr(
    feature = "export-ts",
    derive(ts_rs::TS, postcard_bindgen::PostcardBindings)
)]
#[serde_binhum::serde_binhum]
pub struct InstancePayload {
    pub instance_id: String,
    #[serde(flatten)]
    #[cfg_attr(feature = "export-ts", ts(flatten))]
    pub event: InstancePayloadType,
}

#[derive(Clone)]
#[cfg_attr(
    feature = "export-ts",
    derive(ts_rs::TS, postcard_bindgen::PostcardBindings)
)]
#[serde_binhum::serde_binhum]
pub struct InstanceGroupsChangedPayload {
    pub instance_ids: Vec<String>,
}

#[derive(Clone)]
#[cfg_attr(
    feature = "export-ts",
    derive(ts_rs::TS, postcard_bindgen::PostcardBindings)
)]
#[serde_binhum::serde_binhum]
#[serde(tag = "event", rename_all = "snake_case")]
#[cfg_attr(feature = "export-ts", ts(tag = "event", rename_all = "snake_case"))]
pub enum InstancePayloadType {
    Created,
    Synced,
    ServersUpdated,
    WorldUpdated {
        world: String,
    },
    ServerJoined {
        host: String,
        port: u16,
        timestamp: String,
    },
    Edited,
    ContentInstallFinished {
        project_ids: Vec<String>,
    },
    ContentInstallFailed {
        project_ids: Vec<String>,
        message: String,
    },
    Removed,
}

#[derive(Clone)]
#[cfg_attr(
    feature = "export-ts",
    derive(ts_rs::TS, postcard_bindgen::PostcardBindings)
)]
#[serde_binhum::serde_binhum]
#[serde(rename_all = "snake_case")]
#[serde(tag = "event")]
#[cfg_attr(feature = "export-ts", ts(tag = "event", rename_all = "snake_case"))]
pub enum FriendPayload {
    FriendRequest { from: String },
    UserOffline { id: String },
    StatusUpdate { user_status: FriendStatusPayload },
    StatusSync,
}

#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(
    feature = "export-ts",
    derive(ts_rs::TS, postcard_bindgen::PostcardBindings)
)]
pub struct FriendStatusPayload {
    pub user_id: String,
    pub profile_name: Option<String>,
    pub last_update: String,
}

impl From<ariadne::users::UserStatus> for FriendStatusPayload {
    fn from(status: ariadne::users::UserStatus) -> Self {
        Self {
            user_id: status.user_id.to_string(),
            profile_name: status.profile_name,
            last_update: status.last_update.to_rfc3339(),
        }
    }
}

pub use self::log_types::*;

mod log_types {
    use crate::state::Log4jEvent;

    #[derive(Clone)]
    #[cfg_attr(
        feature = "export-ts",
        derive(ts_rs::TS, postcard_bindgen::PostcardBindings)
    )]
    #[serde_binhum::serde_binhum]
    #[serde(tag = "type", rename_all = "snake_case")]
    #[cfg_attr(
        feature = "export-ts",
        ts(tag = "type", rename_all = "snake_case")
    )]
    pub enum LogEvent {
        Log4j(Log4jEvent),
        Legacy { message: String },
    }

    #[derive(Clone)]
    #[cfg_attr(
        feature = "export-ts",
        derive(ts_rs::TS, postcard_bindgen::PostcardBindings)
    )]
    #[serde_binhum::serde_binhum]
    pub struct LogPayload {
        pub instance_id: String,
        #[serde(flatten)]
        #[cfg_attr(feature = "export-ts", ts(flatten))]
        pub event: LogEvent,
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Non-existent loading bar of key: {0}")]
    NoLoadingBar(Uuid),

    #[cfg(feature = "tauri")]
    #[error("Postcard encoding error: {0}")]
    PostcardEncode(#[from] postcard::Error),

    #[cfg(feature = "tauri")]
    #[error("Tauri error: {0}")]
    TauriError(#[from] tauri::Error),
}
