use derive_more::Display;
use serde::{Deserialize, Serialize};
use url::Url;

use super::fabriclike::{GameVersion, GameVersionName, LoaderVersionName};

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
