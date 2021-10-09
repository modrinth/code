use crate::minecraft::{Argument, ArgumentType, Library, VersionInfo, VersionType};
use crate::{download_file, Error};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The latest version of the format the model structs deserialize to
pub const CURRENT_FORMAT_VERSION: usize = 0;

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
    pub main_class: String,
    /// Arguments passed to the game or JVM
    pub arguments: Option<HashMap<ArgumentType, Vec<Argument>>>,
    /// Libraries that the version depends on
    pub libraries: Vec<Library>,
    #[serde(rename = "type")]
    /// The type of version
    pub type_: VersionType,
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
        main_class: partial.main_class,
        minecraft_arguments: merge.minecraft_arguments,
        minimum_launcher_version: merge.minimum_launcher_version,
        release_time: partial.release_time,
        time: partial.time,
        type_: partial.type_,
    }
}

/// The default servers for fabric meta
pub const FABRIC_META_URL: &str = "https://meta.fabricmc.net/v2";

/// Fetches the manifest of a fabric loader version and game version
pub async fn fetch_fabric_version(
    version_number: &str,
    loader_version: &str,
) -> Result<PartialVersionInfo, Error> {
    Ok(serde_json::from_slice(
        &download_file(
            &*format!(
                "{}/versions/loader/{}/{}/profile/json",
                FABRIC_META_URL, version_number, loader_version
            ),
            None,
        )
        .await?,
    )?)
}

/// Fetches the manifest of a game version's URL
pub async fn fetch_fabric_game_version(url: &str) -> Result<PartialVersionInfo, Error> {
    Ok(serde_json::from_slice(&download_file(url, None).await?)?)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Versions of fabric components
pub struct FabricVersions {
    /// Versions of Minecraft that fabric supports
    pub game: Vec<FabricGameVersion>,
    /// Available versions of the fabric loader
    pub loader: Vec<FabricLoaderVersion>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A version of Minecraft that fabric supports
pub struct FabricGameVersion {
    /// The version number of the game
    pub version: String,
    /// Whether the Minecraft version is stable or not
    pub stable: bool,
    /// (Modrinth Provided) The URLs to download this version's profile with a loader
    ///  The key of the map is the loader version, and the value is the URL
    pub urls: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A version of the fabric loader
pub struct FabricLoaderVersion {
    /// The separator to get the build number
    pub separator: String,
    /// The build number
    pub build: u32,
    /// The maven artifact
    pub maven: String,
    /// The version number of the fabric loader
    pub version: String,
    /// Whether the loader is stable or not
    pub stable: bool,
}
/// Fetches the list of fabric versions
pub async fn fetch_fabric_versions(url: Option<&str>) -> Result<FabricVersions, Error> {
    Ok(serde_json::from_slice(
        &download_file(
            url.unwrap_or(&*format!("{}/versions", FABRIC_META_URL)),
            None,
        )
        .await?,
    )?)
}
