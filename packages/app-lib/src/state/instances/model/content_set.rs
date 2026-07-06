use crate::state::ModLoader;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::unknown_value;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentSourceKind {
    Local,
    ModrinthModpack,
    ServerProject,
    ModrinthHosting,
    ImportedModpack,
    SharedInstance,
}

impl ContentSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::ModrinthModpack => "modrinth_modpack",
            Self::ServerProject => "server_project",
            Self::ModrinthHosting => "modrinth_hosting",
            Self::ImportedModpack => "imported_modpack",
            Self::SharedInstance => "shared_instance",
        }
    }

    pub fn from_str(value: &str) -> crate::Result<Self> {
        match value {
            "local" => Ok(Self::Local),
            "modrinth_modpack" => Ok(Self::ModrinthModpack),
            "server_project" => Ok(Self::ServerProject),
            "modrinth_hosting" => Ok(Self::ModrinthHosting),
            "imported_modpack" => Ok(Self::ImportedModpack),
            "shared_instance" => Ok(Self::SharedInstance),
            other => Err(unknown_value("content source kind", other)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentSetStatus {
    Available,
    Installing,
    Stale,
    MissingFiles,
}

impl ContentSetStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Installing => "installing",
            Self::Stale => "stale",
            Self::MissingFiles => "missing_files",
        }
    }

    pub fn from_str(value: &str) -> crate::Result<Self> {
        match value {
            "available" => Ok(Self::Available),
            "installing" => Ok(Self::Installing),
            "stale" => Ok(Self::Stale),
            "missing_files" => Ok(Self::MissingFiles),
            other => Err(unknown_value("content set status", other)),
        }
    }
}

/// Represents a playable setup slot for an instance.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContentSet {
    pub id: String,
    pub instance_id: String,
    pub name: String,
    pub source_kind: ContentSourceKind,
    pub status: ContentSetStatus,
    pub game_version: String,
    pub protocol_version: Option<u32>,
    pub loader: ModLoader,
    pub loader_version: Option<String>,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
}
