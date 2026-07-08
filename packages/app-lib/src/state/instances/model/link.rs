use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::ContentSetSyncStatus;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceLink {
    Unmanaged,
    ModrinthModpack {
        project_id: String,
        version_id: String,
    },
    ServerProject {
        project_id: String,
    },
    /// A server project that points at a separate content project/version.
    ServerProjectModpack {
        server_project_id: String,
        content_project_id: String,
        content_version_id: String,
    },
    /// Hosting sync still flows through the shared-instance service.
    ModrinthHosting {
        server_id: Uuid,
        instance_ids: Vec<Uuid>,
        active_instance_id: Option<Uuid>,
    },
    /// A custom modpack source without a Modrinth project/version link.
    ImportedModpack {
        project_id: Option<String>,
        version_id: Option<String>,
        name: Option<String>,
        version_number: Option<String>,
        filename: Option<String>,
    },
    SharedInstance {
        modpack_project_id: Option<String>,
        modpack_version_id: Option<String>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SharedInstanceRole {
    Owner,
    Member,
}

impl SharedInstanceRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Member => "member",
        }
    }

    pub fn from_stored_str(value: &str) -> crate::Result<Self> {
        match value {
            "owner" => Ok(Self::Owner),
            "member" => Ok(Self::Member),
            other => Err(super::unknown_value("shared instance role", other)),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SharedInstanceAttachment {
    pub id: String,
    pub role: SharedInstanceRole,
    pub manager_id: Option<String>,
    pub linked_user_id: Option<String>,
    pub access_token: Option<String>,
    pub status: ContentSetSyncStatus,
    pub applied_version: Option<i32>,
    pub latest_version: Option<i32>,
}
