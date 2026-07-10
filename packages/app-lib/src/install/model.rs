use crate::api::pack::import::ImportLauncherType;
use crate::api::pack::install_from::{CreatePackInstance, CreatePackLocation};
use crate::state::{
    InstanceInstallStage, InstanceLink, InstanceMetadata, ModLoader,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

pub type InstallModpackPreview = CreatePackInstance;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstallJobState {
    pub schema_version: u32,
    pub request: InstallRequest,
    pub target: InstallTarget,
    pub cleanup: InstallCleanup,
    pub progress: InstallProgressState,
    pub paths: InstallJobPaths,
    #[serde(default)]
    pub display: Option<InstallJobDisplay>,
    pub rollback: Option<InstallRollbackState>,
    pub error: Option<InstallErrorView>,
}

impl InstallJobState {
    pub fn new(request: InstallRequest) -> Self {
        let target = request.target();
        let cleanup = request.cleanup();
        let phase = InstallPhaseId::PreparingInstance;

        Self {
            schema_version: 1,
            request,
            target,
            cleanup,
            progress: InstallProgressState {
                phase,
                progress: None,
                details: InstallPhaseDetails::Empty,
            },
            paths: InstallJobPaths::default(),
            display: None,
            rollback: None,
            error: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InstallRequest {
    CreateInstance {
        name: String,
        game_version: String,
        loader: ModLoader,
        loader_version: Option<String>,
        icon_path: Option<String>,
        link: InstanceLink,
    },
    CreateModpackInstance {
        location: CreatePackLocation,
        #[serde(default)]
        post_install_edit: Option<InstallPostInstallEdit>,
    },
    CreateSharedInstance {
        data: SharedInstanceInstallData,
    },
    ImportInstance {
        launcher_type: ImportLauncherType,
        base_path: PathBuf,
        instance_folder: String,
    },
    DuplicateInstance {
        source_instance_id: String,
    },
    InstallExistingInstance {
        instance_id: String,
        force: bool,
    },
    InstallPackToExistingInstance {
        instance_id: String,
        location: CreatePackLocation,
        #[serde(default)]
        post_install_edit: Option<InstallPostInstallEdit>,
    },
    UpdateSharedInstance {
        instance_id: String,
        data: SharedInstanceInstallData,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct InstallPostInstallEdit {
    pub name: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub icon_path: Option<Option<String>>,
    pub link: Option<InstanceLink>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SharedInstanceInstallData {
    pub shared_instance_id: String,
    pub manager_id: Option<String>,
    #[serde(default)]
    pub server_manager_name: Option<String>,
    #[serde(default)]
    pub server_manager_icon_url: Option<String>,
    #[serde(default)]
    pub linked_user_id: Option<String>,
    pub name: String,
    pub version: i32,
    pub modrinth_ids: Vec<String>,
    #[serde(default)]
    pub external_files: Vec<SharedInstanceExternalFileData>,
    pub modpack: Option<SharedInstanceInstallModpack>,
    pub game_version: String,
    pub loader: ModLoader,
    pub loader_version: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SharedInstanceExternalFileData {
    pub file_name: String,
    pub file_type: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SharedInstanceInstallModpack {
    pub project_id: String,
    pub version_id: String,
    pub title: String,
    pub icon_url: Option<String>,
    pub dependency_count: usize,
}

impl InstallRequest {
    pub fn kind(&self) -> InstallJobKind {
        match self {
            Self::CreateInstance { .. } => InstallJobKind::CreateInstance,
            Self::CreateModpackInstance { .. } => {
                InstallJobKind::CreateModpackInstance
            }
            Self::CreateSharedInstance { .. } => {
                InstallJobKind::CreateSharedInstance
            }
            Self::ImportInstance { .. } => InstallJobKind::ImportInstance,
            Self::DuplicateInstance { .. } => InstallJobKind::DuplicateInstance,
            Self::InstallExistingInstance { .. } => {
                InstallJobKind::InstallExistingInstance
            }
            Self::InstallPackToExistingInstance { .. } => {
                InstallJobKind::InstallPackToExistingInstance
            }
            Self::UpdateSharedInstance { .. } => {
                InstallJobKind::UpdateSharedInstance
            }
        }
    }

    pub fn target(&self) -> InstallTarget {
        match self {
            Self::InstallExistingInstance { instance_id, .. }
            | Self::InstallPackToExistingInstance { instance_id, .. }
            | Self::UpdateSharedInstance { instance_id, .. } => {
                InstallTarget::ExistingInstance {
                    instance_id: instance_id.clone(),
                }
            }
            _ => InstallTarget::NewInstance { instance_id: None },
        }
    }

    pub fn cleanup(&self) -> InstallCleanup {
        match self {
            Self::InstallExistingInstance { instance_id, .. }
            | Self::InstallPackToExistingInstance { instance_id, .. }
            | Self::UpdateSharedInstance { instance_id, .. } => {
                InstallCleanup::RestoreExistingInstance {
                    instance_id: instance_id.clone(),
                }
            }
            _ => InstallCleanup::DeleteNewInstance { instance_id: None },
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InstallJobKind {
    CreateInstance,
    CreateModpackInstance,
    CreateSharedInstance,
    ImportInstance,
    DuplicateInstance,
    InstallExistingInstance,
    InstallPackToExistingInstance,
    UpdateSharedInstance,
}

impl InstallJobKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CreateInstance => "create_instance",
            Self::CreateModpackInstance => "create_modpack_instance",
            Self::CreateSharedInstance => "create_shared_instance",
            Self::ImportInstance => "import_instance",
            Self::DuplicateInstance => "duplicate_instance",
            Self::InstallExistingInstance => "install_existing_instance",
            Self::InstallPackToExistingInstance => {
                "install_pack_to_existing_instance"
            }
            Self::UpdateSharedInstance => "update_shared_instance",
        }
    }

    pub fn from_stored_str(value: &str) -> Self {
        match value {
            "create_modpack_instance" => Self::CreateModpackInstance,
            "create_shared_instance" => Self::CreateSharedInstance,
            "import_instance" => Self::ImportInstance,
            "duplicate_instance" => Self::DuplicateInstance,
            "install_existing_instance" => Self::InstallExistingInstance,
            "install_pack_to_existing_instance" => {
                Self::InstallPackToExistingInstance
            }
            "update_shared_instance" => Self::UpdateSharedInstance,
            _ => Self::CreateInstance,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InstallJobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Interrupted,
    Canceled,
}

impl InstallJobStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Interrupted => "interrupted",
            Self::Canceled => "canceled",
        }
    }

    pub fn from_stored_str(value: &str) -> Self {
        match value {
            "running" => Self::Running,
            "succeeded" => Self::Succeeded,
            "failed" => Self::Failed,
            "interrupted" => Self::Interrupted,
            "canceled" => Self::Canceled,
            _ => Self::Queued,
        }
    }

    pub fn is_finished(self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Failed | Self::Interrupted | Self::Canceled
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InstallTarget {
    NewInstance { instance_id: Option<String> },
    ExistingInstance { instance_id: String },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InstallCleanup {
    DeleteNewInstance { instance_id: Option<String> },
    RestoreExistingInstance { instance_id: String },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstallProgressState {
    pub phase: InstallPhaseId,
    pub progress: Option<InstallProgress>,
    pub details: InstallPhaseDetails,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InstallPhaseId {
    PreparingInstance,
    ResolvingPack,
    DownloadingPackFile,
    ReadingPackManifest,
    DownloadingContent,
    ExtractingOverrides,
    ResolvingMinecraft,
    ResolvingLoader,
    PreparingJava,
    DownloadingMinecraft,
    RunningLoaderProcessors,
    Finalizing,
    RollingBack,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstallProgress {
    pub current: u64,
    pub total: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secondary: Option<InstallProgressSecondary>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstallProgressSecondary {
    pub current: u64,
    pub total: u64,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InstallJavaStep {
    Resolving,
    FetchingMetadata,
    Downloading,
    Extracting,
    Validating,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InstallPhaseDetails {
    Empty,
    Instance {
        name: String,
    },
    Minecraft {
        game_version: String,
        loader: ModLoader,
    },
    Java {
        major_version: u32,
        step: InstallJavaStep,
    },
    Modpack {
        project_id: Option<String>,
        version_id: Option<String>,
        title: Option<String>,
    },
    Import {
        launcher_type: ImportLauncherType,
        instance_folder: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct InstallJobPaths {
    pub staging_dir: Option<PathBuf>,
    pub final_instance_path: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstallJobDisplay {
    pub title: String,
    pub icon: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstallRollbackState {
    pub instance: InstanceMetadata,
    pub install_stage: InstanceInstallStage,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstallErrorView {
    pub code: String,
    pub message: String,
}

impl InstallErrorView {
    pub fn from_error(code: &str, error: impl ToString) -> Self {
        Self {
            code: code.to_string(),
            message: error.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstallJobSnapshot {
    pub job_id: Uuid,
    pub instance_id: Option<String>,
    pub kind: InstallJobKind,
    pub status: InstallJobStatus,
    pub target: InstallTarget,
    pub phase: InstallPhaseId,
    pub progress: Option<InstallProgress>,
    pub details: InstallPhaseDetails,
    pub display: Option<InstallJobDisplay>,
    pub error: Option<InstallErrorView>,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub finished: Option<DateTime<Utc>>,
}
