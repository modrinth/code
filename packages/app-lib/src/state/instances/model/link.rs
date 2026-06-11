use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    },
    SharedInstance {
        shared_instance_id: Uuid,
    },
}
