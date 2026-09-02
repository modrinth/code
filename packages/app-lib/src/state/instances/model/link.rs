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
    /// Modpack provenance installed by a shared instance. Remote membership,
    /// manager identity, and synchronization state belong to
    /// [`SharedInstanceAttachment`].
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
    pub fn is_member(self) -> bool {
        self == Self::Member
    }

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
/// The remote shared-instance relationship for a local instance. This is
/// independent from the optional modpack provenance stored in [`InstanceLink`].
pub struct SharedInstanceAttachment {
    pub id: String,
    pub role: SharedInstanceRole,
    pub manager_id: Option<String>,
    #[serde(default)]
    pub server_manager_name: Option<String>,
    #[serde(default)]
    pub server_manager_icon_url: Option<String>,
    pub linked_user_id: Option<String>,
    pub status: ContentSetSyncStatus,
    pub applied_version: Option<i32>,
    pub latest_version: Option<i32>,
}

#[derive(Clone, Debug)]
pub(crate) struct SharedInstanceAttachmentInput {
    pub id: String,
    pub role: SharedInstanceRole,
    pub manager_id: Option<String>,
    pub server_manager_name: Option<String>,
    pub server_manager_icon_url: Option<String>,
    pub linked_user_id: Option<String>,
    pub status: ContentSetSyncStatus,
    pub applied_version: Option<i32>,
    pub latest_version: Option<i32>,
}

impl From<SharedInstanceAttachmentInput> for SharedInstanceAttachment {
    fn from(input: SharedInstanceAttachmentInput) -> Self {
        Self {
            id: input.id,
            role: input.role,
            manager_id: input.manager_id,
            server_manager_name: input.server_manager_name,
            server_manager_icon_url: input.server_manager_icon_url,
            linked_user_id: input.linked_user_id,
            status: input.status,
            applied_version: input.applied_version,
            latest_version: input.latest_version,
        }
    }
}
