use serde::{Deserialize, Serialize};

use super::{ContentEntry, InstanceFile};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstanceContentManifest {
    pub instance_id: String,
    pub content_set_id: String,
    pub entries: Vec<ContentEntry>,
    pub files: Vec<InstanceFile>,
}
