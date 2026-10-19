use indexmap::IndexMap;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    minecraft::{
        Argument, ArgumentType, LibraryDownloads, LibraryExtract,
        OperatingSystem, Rule, VersionType,
    },
    profile::{Processor, SidedDataEntry},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub game_versions: Vec<GameVersion>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub version_groups: Vec<VersionGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameVersion {
    pub id: String,
    pub stable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_group: Option<String>,
    pub loaders: Vec<LoaderVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoaderVersion {
    pub id: String,
    pub url: Url,
    pub stable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionGroup {
    pub id: String,
    pub loaders: Vec<LoaderVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabriclikeProfileMetadata {
    pub id: String,
    pub inherits_from: String,
    pub release_time: Timestamp,
    pub time: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_arguments: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<IndexMap<ArgumentType, Vec<Argument>>>,
    pub libraries: Vec<FabriclikeLibrary>,
    #[serde(rename = "type")]
    pub ty: VersionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<IndexMap<String, SidedDataEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processors: Option<Vec<Processor>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabriclikeLibrary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloads: Option<LibraryDownloads>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extract: Option<LibraryExtract>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub natives: Option<IndexMap<OperatingSystem, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<Rule>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksums: Option<Vec<String>>,
    #[serde(default = "default_true")]
    pub include_in_classpath: bool,
    #[serde(default = "default_true")]
    pub downloadable: bool,
}

fn default_true() -> bool {
    true
}
