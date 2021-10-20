use crate::{download_file, Error};

use crate::minecraft::{Argument, ArgumentType, Library, VersionInfo, VersionType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The latest version of the format the fabric model structs deserialize to
pub const CURRENT_FABRIC_FORMAT_VERSION: usize = 0;
/// The latest version of the format the fabric model structs deserialize to
pub const CURRENT_FORGE_FORMAT_VERSION: usize = 0;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
/// A partial version returned by fabric meta
pub struct PartialVersionInfo {
    /// The version ID of the version
    pub id: String,
    /// The version ID this partial version inherits from
    pub inherits_from: String,
    /// The time that the version was released
    pub release_time: DateTime<Utc>,
    /// The latest time a file in this version was updated
    pub time: DateTime<Utc>,
    /// The classpath to the main class to launch the game
    pub main_class: Option<String>,
    /// Arguments passed to the game or JVM
    pub arguments: Option<HashMap<ArgumentType, Vec<Argument>>>,
    /// Libraries that the version depends on
    pub libraries: Vec<Library>,
    #[serde(rename = "type")]
    /// The type of version
    pub type_: VersionType,
}

/// Fetches the version manifest of a game version's URL
pub async fn fetch_partial_version(url: &str) -> Result<PartialVersionInfo, Error> {
    Ok(serde_json::from_slice(&download_file(url, None).await?)?)
}

/// Merges a partial version into a complete one
pub fn merge_partial_version(partial: PartialVersionInfo, merge: VersionInfo) -> VersionInfo {
    VersionInfo {
        arguments: if let Some(partial_args) = partial.arguments {
            if let Some(merge_args) = merge.arguments {
                Some(partial_args.into_iter().chain(merge_args).collect())
            } else {
                Some(partial_args)
            }
        } else {
            merge.arguments
        },
        asset_index: merge.asset_index,
        assets: merge.assets,
        downloads: merge.downloads,
        id: merge.id,
        libraries: partial
            .libraries
            .into_iter()
            .chain(merge.libraries)
            .collect::<Vec<_>>(),
        main_class: if let Some(main_class) = partial.main_class {
            main_class
        } else {
            merge.main_class
        },
        minecraft_arguments: merge.minecraft_arguments,
        minimum_launcher_version: merge.minimum_launcher_version,
        release_time: partial.release_time,
        time: partial.time,
        type_: partial.type_,
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
/// A manifest containing information about a mod loader's versions
pub struct Manifest {
    /// The game versions the mod loader supports
    pub game_versions: Vec<Version>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone)]
#[serde(rename_all = "camelCase")]
/// The version type of the loader
pub enum LoaderType {
    /// The latest type is for experimental loader versions that may not be ready for normal use
    Latest,
    /// The stable type is for the most stable but recent loader version. For the forge mod loader,
    /// this is never used
    Stable,
}

#[derive(Serialize, Deserialize, Debug)]
///  A game version of Minecraft
pub struct Version {
    /// The minecraft version ID
    pub id: String,
    /// A map that contains loader versions for the game version
    pub loaders: HashMap<LoaderType, LoaderVersion>,
}

#[derive(Serialize, Deserialize, Debug)]
/// A version of a Minecraft mod loader
pub struct LoaderVersion {
    /// The version ID of the loader
    pub id: String,
    /// The URL of the version's manifest
    pub url: String,
}

/// Fetches the manifest of a mod loader
pub async fn fetch_manifest(url: &str) -> Result<Manifest, Error> {
    Ok(serde_json::from_slice(&download_file(url, None).await?)?)
}
