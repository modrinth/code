use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::models::ids::UserId;

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, Debug)]
#[serde(from = "Base62Id")]
#[serde(into = "Base62Id")]
pub struct SharedInstanceId(pub u64);

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, Debug)]
#[serde(from = "Base62Id")]
#[serde(into = "Base62Id")]
pub struct SharedInstanceInviteId(pub u64);

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, Debug)]
#[serde(from = "Base62Id")]
#[serde(into = "Base62Id")]
pub struct SharedInstanceFileId(pub u64);

#[derive(Serialize, Deserialize, Clone)]
pub struct SharedInstance {
    pub id: SharedInstanceId,
    pub creator_id: UserId,
    pub icon_url: String,
    pub name: String,
    pub status: SharedInstanceStatus,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,

    pub include_paths: Vec<String>,
    pub exclude_paths: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum SharedInstanceStatus {
    Active,
    Rejected,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SharedInstanceInvite {
    pub id: SharedInstanceInviteId,
    pub creator_id: UserId,
    pub shared_instance_id: SharedInstanceId,

    pub created: DateTime<Utc>,
    pub expires: Option<DateTime<Utc>>,
    pub last_used: Option<DateTime<Utc>>,
    pub max_users: Option<u64>,
    pub uses: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SharedInstanceFile {
    pub id: SharedInstanceFileId,
    pub size: u64,
    pub install_path: String,
    pub side: SharedInstanceFileSide,
    pub source: SharedInstanceFileSource,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum SharedInstanceFileSource {
    Modrinth {
        url: String,
    },
    File {
        hash: String,
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum SharedInstanceFileSide {
    ClientOnly,
    ServerOnly,
    Universal,
}