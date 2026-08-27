use quartz_nbt::NbtCompound;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerSource {
    UserSynced,
    Modpack,
    LinkedServerProject,
    LocalDesynced,
}

impl ServerSource {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::UserSynced => "user_synced",
            Self::Modpack => "modpack",
            Self::LinkedServerProject => "linked_server_project",
            Self::LocalDesynced => "local_desynced",
        }
    }

    pub(super) fn from_str(value: &str) -> Option<Self> {
        match value {
            "user_synced" => Some(Self::UserSynced),
            "modpack" => Some(Self::Modpack),
            "linked_server_project" => Some(Self::LinkedServerProject),
            "local_desynced" => Some(Self::LocalDesynced),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesyncServerMode {
    KeepInOtherInstances,
    RemoveFromOtherInstances,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncedServer {
    pub id: String,
    pub name: String,
    pub address: String,
    pub accept_textures: Option<bool>,
}

#[derive(Clone, Debug)]
pub(crate) struct ServerRecord {
    pub id: String,
    pub source: ServerSource,
    pub data: NbtCompound,
}

impl ServerRecord {
    pub fn name(&self) -> String {
        self.data
            .get::<_, &str>("name")
            .unwrap_or_default()
            .to_string()
    }

    pub fn address(&self) -> String {
        self.data
            .get::<_, &str>("ip")
            .unwrap_or_default()
            .to_string()
    }

    pub fn icon(&self) -> Option<String> {
        self.data.get::<_, &str>("icon").ok().map(ToOwned::to_owned)
    }

    pub fn hidden(&self) -> bool {
        self.data.get::<_, i8>("hidden").unwrap_or(0) != 0
    }

    pub fn accept_textures(&self) -> Option<bool> {
        self.data
            .get::<_, i8>("acceptTextures")
            .ok()
            .map(|value| value != 0)
    }
}

#[derive(Clone)]
pub(super) struct CanonicalServer {
    pub id: String,
    pub data: NbtCompound,
}

#[derive(Clone)]
pub(super) struct LocalServer {
    pub id: String,
    pub source: ServerSource,
    pub excluded_synced_server_id: Option<String>,
    pub data: NbtCompound,
    pub position: i64,
}

#[derive(Clone)]
pub(super) struct ProjectionEntry {
    pub id: String,
    pub owner: ProjectionOwner,
    pub data: NbtCompound,
    pub position: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ProjectionOwner {
    Synced,
    Instance,
}

impl ProjectionOwner {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Synced => "synced",
            Self::Instance => "instance",
        }
    }

    pub(super) fn from_str(value: &str) -> Option<Self> {
        match value {
            "synced" => Some(Self::Synced),
            "instance" => Some(Self::Instance),
            _ => None,
        }
    }
}
