use derive_more::Display;
use serde::{Deserialize, Serialize};
use url::Url;

use super::fabriclike::{GameVersion, GameVersionName, LoaderVersionName};

use crate::util::{MavenCoordinate, Sha1, Sha256};

pub const CATALOG_URL: &str = "https://meta.quiltmc.org/v3/versions";
pub const PROFILE_BASE_URL: &str =
    "https://meta.quiltmc.org/v3/versions/loader/";
pub const MAVEN_URL: &str = "https://maven.quiltmc.org/repository/release/";

pub struct MetadataGroup {
    pub id: String,
    pub template: GameVersionName,
    pub games: Vec<GameVersionName>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    pub game: Vec<GameVersion>,
    pub mappings: Vec<MappingVersion>,
    pub hashed: Vec<HashedVersion>,
    pub loader: Vec<LoaderVersion>,
    pub installer: Vec<InstallerVersion>,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct MappingVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingVersion {
    pub maven: MavenCoordinate,
    pub version: MappingVersionName,
    #[serde(rename = "gameVersion")]
    pub game_version: GameVersionName,
    pub build: u32,
    pub separator: String,
    pub hashed: HashedVersionName,
    #[serde(rename = "file_size")]
    pub file_size: u64,
    pub hashes: Hashes,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct HashedVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct HashedVersion {
    pub maven: MavenCoordinate,
    pub version: HashedVersionName,
    pub file_size: u64,
    pub hashes: Hashes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LoaderVersion {
    #[serde(default)]
    pub stable: bool,
    pub maven: MavenCoordinate,
    pub version: LoaderVersionName,
    pub build: u32,
    pub separator: String,
    pub file_size: u64,
    pub hashes: Hashes,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct InstallerVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct InstallerVersion {
    pub maven: MavenCoordinate,
    pub version: InstallerVersionName,
    pub url: Url,
    pub file_size: u64,
    pub hashes: Hashes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hashes {
    pub sha1: Sha1,
    pub sha256: Sha256,
    pub sha512: String,
}
