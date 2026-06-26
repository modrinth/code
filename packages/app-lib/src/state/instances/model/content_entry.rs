use crate::state::ProjectType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{ContentSourceKind, unknown_value};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentRequirement {
    Required,
    Optional,
    Unsupported,
    Unknown,
}

impl ContentRequirement {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Required => "required",
            Self::Optional => "optional",
            Self::Unsupported => "unsupported",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_str(value: &str) -> crate::Result<Self> {
        match value {
            "required" => Ok(Self::Required),
            "optional" => Ok(Self::Optional),
            "unsupported" => Ok(Self::Unsupported),
            "unknown" => Ok(Self::Unknown),
            other => Err(unknown_value("content requirement", other)),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContentEntry {
    pub id: String,
    pub instance_id: String,
    pub content_set_id: String,
    pub file_id: Option<String>,
    pub project_type: ProjectType,
    pub project_id: Option<String>,
    pub version_id: Option<String>,
    pub source_kind: ContentSourceKind,
    pub server_requirement: ContentRequirement,
    pub client_requirement: ContentRequirement,
    pub enabled: bool,
    pub added_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}
