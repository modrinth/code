use std::collections::HashMap;

use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use toasty::Model;

use crate::{
    model::{MinecraftLoader, MinecraftVersionName},
    upstream::mojang::{Argument, ArgumentType, Library, VersionType},
};

/// Normalized loader profile to be published as a version JSON artifact.
#[derive(Debug, Clone, Model)]
#[key(loader, minecraft_version, loader_version)]
pub struct Profile {
    pub loader: MinecraftLoader,
    #[index]
    pub minecraft_version: MinecraftVersionName,
    pub loader_version: String,
    #[column(type = json)]
    pub metadata: toasty::Json<ProfileMetadata>,
}

/// Launch metadata combined with optional Forge-style installation metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileMetadata {
    pub id: String,
    pub inherits_from: String,
    pub release_time: Timestamp,
    pub time: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_arguments: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<HashMap<ArgumentType, Vec<Argument>>>,
    pub libraries: Vec<Library>,
    #[serde(rename = "type")]
    pub ty: VersionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<HashMap<String, SidedDataEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processors: Option<Vec<Processor>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidedDataEntry {
    pub client: String,
    pub server: String,
}

/// Installation step executed by the launcher after downloading libraries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Processor {
    pub jar: String,
    pub classpath: Vec<String>,
    pub args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sides: Option<Vec<String>>,
}
