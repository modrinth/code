use crate::minecraft::{
    Argument, ArgumentType, Library, VersionInfo, VersionType,
};
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

/// The latest version of the format the fabric model structs deserialize to
pub const CURRENT_FABRIC_FORMAT_VERSION: usize = 0;
/// The latest version of the format the fabric model structs deserialize to
pub const CURRENT_FORGE_FORMAT_VERSION: usize = 0;
/// The latest version of the format the quilt model structs deserialize to
pub const CURRENT_QUILT_FORMAT_VERSION: usize = 0;
/// The latest version of the format the neoforge model structs deserialize to
pub const CURRENT_NEOFORGE_FORMAT_VERSION: usize = 0;

/// The dummy replace string library names, inheritsFrom, and version names should be replaced with
pub const DUMMY_REPLACE_STRING: &str = "${modrinth.gameVersion}";

/// A data variable entry that depends on the side of the installation
#[derive(Serialize, Deserialize, Debug)]
pub struct SidedDataEntry {
    /// The value on the client
    pub client: String,
    /// The value on the server
    pub server: String,
}

#[allow(deprecated)]
fn deserialize_date<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    serde_json::from_str::<DateTime<Utc>>(&format!("\"{s}\""))
        .or_else(|_| Utc.datetime_from_str(&s, "%Y-%m-%dT%H:%M:%S%.9f"))
        .map_err(serde::de::Error::custom)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
/// A partial version returned by fabric meta
pub struct PartialVersionInfo {
    /// The version ID of the version
    pub id: String,
    /// The version ID this partial version inherits from
    pub inherits_from: String,
    /// The time that the version was released
    #[serde(deserialize_with = "deserialize_date")]
    pub release_time: DateTime<Utc>,
    /// The latest time a file in this version was updated
    #[serde(deserialize_with = "deserialize_date")]
    pub time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The classpath to the main class to launch the game
    pub main_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// (Legacy) Arguments passed to the game
    pub minecraft_arguments: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Arguments passed to the game or JVM
    pub arguments: Option<HashMap<ArgumentType, Vec<Argument>>>,
    /// Libraries that the version depends on
    pub libraries: Vec<Library>,
    #[serde(rename = "type")]
    /// The type of version
    pub type_: VersionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// (Forge-only)
    pub data: Option<HashMap<String, SidedDataEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// (Forge-only) The list of processors to run after downloading the files
    pub processors: Option<Vec<Processor>>,
}

/// A processor to be ran after downloading the files
#[derive(Serialize, Deserialize, Debug)]
pub struct Processor {
    /// Maven coordinates for the JAR library of this processor.
    pub jar: String,
    /// Maven coordinates for all the libraries that must be included in classpath when running this processor.
    pub classpath: Vec<String>,
    /// Arguments for this processor.
    pub args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Represents a map of outputs. Keys and values can be data values
    pub outputs: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Which sides this processor shall be ran on.
    /// Valid values: client, server, extract
    pub sides: Option<Vec<String>>,
}

/// Merges a partial version into a complete one
pub fn merge_partial_version(
    partial: PartialVersionInfo,
    merge: VersionInfo,
) -> VersionInfo {
    let merge_id = merge.id.clone();

    let mut libraries = vec![];

    // We skip duplicate libraries that exist already in the partial version
    for mut lib in merge.libraries {
        let lib_artifact = lib.name.rsplit_once(':').map(|x| x.0);

        if let Some(lib_artifact) = lib_artifact {
            if !partial.libraries.iter().any(|x| {
                let target_artifact = x.name.rsplit_once(':').map(|x| x.0);

                target_artifact == Some(lib_artifact) && x.include_in_classpath
            }) {
                libraries.push(lib);
            } else {
                lib.include_in_classpath = false;
            }
        } else {
            libraries.push(lib);
        }
    }

    VersionInfo {
        arguments: if let Some(partial_args) = partial.arguments {
            if let Some(merge_args) = merge.arguments {
                let mut new_map = HashMap::new();

                fn add_keys(
                    new_map: &mut HashMap<ArgumentType, Vec<Argument>>,
                    args: HashMap<ArgumentType, Vec<Argument>>,
                ) {
                    for (type_, arguments) in args {
                        for arg in arguments {
                            if let Some(vec) = new_map.get_mut(&type_) {
                                vec.push(arg);
                            } else {
                                new_map.insert(type_, vec![arg]);
                            }
                        }
                    }
                }

                add_keys(&mut new_map, merge_args);
                add_keys(&mut new_map, partial_args);

                Some(new_map)
            } else {
                Some(partial_args)
            }
        } else {
            merge.arguments
        },
        asset_index: merge.asset_index,
        assets: merge.assets,
        downloads: merge.downloads,
        id: partial.id.replace(DUMMY_REPLACE_STRING, &merge_id),
        java_version: merge.java_version,
        libraries: libraries
            .into_iter()
            .chain(partial.libraries)
            .map(|mut x| {
                x.name = x.name.replace(DUMMY_REPLACE_STRING, &merge_id);

                x
            })
            .collect::<Vec<_>>(),
        logging: merge.logging,
        main_class: if let Some(main_class) = partial.main_class {
            main_class
        } else {
            merge.main_class
        },
        minecraft_arguments: partial.minecraft_arguments,
        minimum_launcher_version: merge.minimum_launcher_version,
        release_time: partial.release_time,
        time: partial.time,
        type_: partial.type_,
        data: partial.data,
        processors: partial.processors,
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
/// A manifest containing information about a mod loader's versions
pub struct Manifest {
    /// The game versions the mod loader supports
    pub game_versions: Vec<Version>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
///  A game version of Minecraft
pub struct Version {
    /// The minecraft version ID
    pub id: String,
    /// Whether the release is stable or not
    pub stable: bool,
    /// A map that contains loader versions for the game version
    pub loaders: Vec<LoaderVersion>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A version of a Minecraft mod loader
pub struct LoaderVersion {
    /// The version ID of the loader
    pub id: String,
    /// The URL of the version's manifest
    pub url: String,
    /// Whether the loader is stable or not
    pub stable: bool,
}
