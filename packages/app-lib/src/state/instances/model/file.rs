use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstanceFile {
    pub id: String,
    pub instance_id: String,
    pub relative_path: String,
    pub file_name: String,
    pub enabled: bool,
    pub sha1: String,
    pub size: u64,
    pub missing: bool,
    pub added_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}
