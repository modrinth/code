//! Schemas and logic for generating manifest artifacts consumed by the Modrinth
//! App.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::upstream::mojang;

/// Minecraft version catalog exported at
/// `minecraft/v{format_version}/manifest.json`.
pub type MinecraftCatalog = mojang::Catalog;

/// Complete Minecraft version manifest exported at
/// `minecraft/v{format_version}/versions/{game_version}.json`.
pub type MinecraftVersionManifest = mojang::VersionManifest;

/// Mod-loader catalog exported at
/// `{loader}/v{format_version}/manifest.json`.
///
/// This schema is shared by Fabric, Forge, NeoForge, and Quilt.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoaderCatalog {
    /// Minecraft versions supported by the loader.
    pub game_versions: Vec<LoaderGameVersion>,
    /// Groups of Minecraft versions that share compatible loader profiles.
    pub version_groups: Vec<LoaderVersionGroup>,
}

/// A Minecraft version supported by a mod loader.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoaderGameVersion {
    /// Minecraft version identifier.
    pub id: String,
    /// Whether this Minecraft version is stable.
    pub stable: bool,
    /// Group containing the loader profiles compatible with this version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_group: Option<String>,
    /// Loader versions compatible with this Minecraft version.
    pub loaders: Vec<LoaderVersion>,
}

/// A group of Minecraft versions that share compatible loader profiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoaderVersionGroup {
    /// Version group identifier.
    pub id: String,
    /// Loader versions available to the group.
    pub loaders: Vec<LoaderVersion>,
}

/// A mod-loader version and its exported profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoaderVersion {
    /// Loader version identifier.
    pub id: String,
    /// URL of the loader version manifest.
    pub url: String,
    /// Whether this loader version is stable.
    pub stable: bool,
}

/// Partial version manifest exported for a mod-loader version.
///
/// Fabric, Forge, and NeoForge export these at
/// `{loader}/v{format_version}/versions/{loader_version}.json`. Quilt exports
/// templated profiles grouped by compatible Minecraft versions at
/// `{loader}/v{format_version}/version-group/{group}/loader-version/{loader_version}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoaderVersionManifest {
    /// Version identifier.
    pub id: String,
    /// Minecraft version inherited by this loader profile.
    pub inherits_from: String,
    /// Time when this version was released.
    pub release_time: String,
    /// Time when this version was last updated.
    pub time: String,
    /// Main class used to launch the loader.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_class: Option<String>,
    /// Legacy arguments passed to Minecraft.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_arguments: Option<String>,
    /// Arguments passed to Minecraft or the JVM.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<HashMap<mojang::ArgumentType, Vec<mojang::Argument>>>,
    /// Libraries required by the loader.
    pub libraries: Vec<mojang::Library>,
    /// Minecraft release type.
    #[serde(rename = "type")]
    pub ty: mojang::VersionType,
    /// Forge processor data, keyed by variable name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<HashMap<String, SidedDataEntry>>,
    /// Forge processors to run after downloading artifacts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processors: Option<Vec<Processor>>,
}

/// A value that differs between client and server installations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidedDataEntry {
    pub client: String,
    pub server: String,
}

/// A post-download processor from a Forge or NeoForge installer.
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
