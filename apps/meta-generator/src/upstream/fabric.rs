use derive_more::Display;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::util::MavenCoordinate;

pub const CATALOG_URL: &str = "https://meta.fabricmc.net/v2/versions";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    pub game: Vec<GameVersion>,
    pub mappings: Vec<MappingVersion>,
    pub intermediary: Vec<IntermediaryVersion>,
    pub loader: Vec<LoaderVersion>,
    pub installer: Vec<InstallerVersion>,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct GameVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameVersion {
    pub version: GameVersionName,
    pub stable: bool,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct MappingVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MappingVersion {
    pub game_version: GameVersionName,
    pub separator: String,
    pub build: u32,
    pub maven: MavenCoordinate,
    pub version: MappingVersionName,
    pub stable: bool,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct IntermediaryVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntermediaryVersion {
    pub maven: MavenCoordinate,
    pub version: IntermediaryVersionName,
    pub stable: bool,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct LoaderVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoaderVersion {
    pub separator: String,
    pub build: u32,
    pub maven: MavenCoordinate,
    pub version: LoaderVersionName,
    pub stable: bool,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct InstallerVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallerVersion {
    pub url: Url,
    pub maven: MavenCoordinate,
    pub version: InstallerVersionName,
    pub stable: bool,
}

/// Example URL: <https://meta.fabricmc.net/v2/versions/loader/1.21/0.19.5/profile/json>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameLoaderProfile {
    pub id: String,
}
