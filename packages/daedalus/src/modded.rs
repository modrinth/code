use crate::minecraft::{
    Argument, ArgumentType, JavaVersion, Library, LoggingConfiguration,
    LoggingSide, VersionInfo, VersionType,
};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

/// The latest version of the format the fabric model structs deserialize to
pub const CURRENT_FABRIC_FORMAT_VERSION: usize = 0;
/// The latest version of the format the fabric model structs deserialize to
pub const CURRENT_FORGE_FORMAT_VERSION: usize = 0;
/// The latest version of the format the quilt model structs deserialize to
pub const CURRENT_QUILT_FORMAT_VERSION: usize = 1;
/// The latest version of the format the neoforge model structs deserialize to
pub const CURRENT_NEOFORGE_FORMAT_VERSION: usize = 0;
/// The latest version of the format the ornithe model structs deserialize to
pub const CURRENT_ORNITHE_FORMAT_VERSION: usize = 0;

/// Metadata for locating and caching a loader manifest.
#[derive(Debug, Clone)]
pub struct LoaderManifestMetadata {
    /// The canonical loader name used in launcher-meta paths.
    pub loader: String,
    /// The latest manifest format version for this loader.
    pub format_version: usize,
    /// The cache key that includes the loader format version.
    pub cache_key: String,
    /// The launcher-meta path to the manifest.
    pub path: String,
}

/// Returns metadata for the latest manifest format for the provided loader.
pub fn loader_manifest_metadata(loader: &str) -> LoaderManifestMetadata {
    let format_version = current_loader_manifest_format_version(loader);
    let cache_key = format!("{loader}-v{format_version}");
    let path = format!("{loader}/v{format_version}/manifest.json");

    LoaderManifestMetadata {
        loader: loader.to_string(),
        format_version,
        cache_key,
        path,
    }
}

/// Returns loader manifest metadata from a versioned cache key.
pub fn loader_manifest_metadata_from_cache_key(
    cache_key: &str,
) -> LoaderManifestMetadata {
    if let Some((loader, format_version)) =
        cache_key.rsplit_once("-v").and_then(|(loader, version)| {
            version
                .parse::<usize>()
                .ok()
                .map(|version| (loader, version))
        })
    {
        let cache_key = format!("{loader}-v{format_version}");
        let path = format!("{loader}/v{format_version}/manifest.json");

        LoaderManifestMetadata {
            loader: loader.to_string(),
            format_version,
            cache_key,
            path,
        }
    } else {
        loader_manifest_metadata(cache_key)
    }
}

fn current_loader_manifest_format_version(loader: &str) -> usize {
    match loader {
        "fabric" => CURRENT_FABRIC_FORMAT_VERSION,
        "forge" => CURRENT_FORGE_FORMAT_VERSION,
        "quilt" => CURRENT_QUILT_FORMAT_VERSION,
        "neo" => CURRENT_NEOFORGE_FORMAT_VERSION,
        "ornithe" => CURRENT_ORNITHE_FORMAT_VERSION,
        _ => 0,
    }
}

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

fn deserialize_date<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut s = String::deserialize(deserializer)?;

    if let Some((_, offset)) = s.rsplit_once(['+', '-'])
        && matches!(
            offset.as_bytes(),
            [b'0'..=b'9', b':', b'0'..=b'9', b'0'..=b'9']
        )
    {
        s.insert(s.len() - offset.len(), '0');
    }

    DateTime::parse_from_rfc3339(&s)
        .or_else(|_| DateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f%z"))
        .map(|date| date.with_timezone(&Utc))
        .or_else(|_| {
            NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f")
                .map(|date| date.and_utc())
        })
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// The Java version the loader prefers over the one the game asks for
    pub java_version: Option<JavaVersion>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// The logging configuration the loader uses in place of the game's
    pub logging: Option<HashMap<LoggingSide, LoggingConfiguration>>,
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

    let partial_arguments = partial.arguments.map(|args| {
        args.into_iter()
            .filter(|(_, arguments)| !arguments.is_empty())
            .map(|(type_, arguments)| {
                let arguments = arguments
                    .into_iter()
                    .map(|argument| match argument {
                        Argument::Normal(value) => Argument::Normal(
                            value.replace(DUMMY_REPLACE_STRING, &merge_id),
                        ),
                        ruled => ruled,
                    })
                    .collect();

                (type_, arguments)
            })
            .collect::<HashMap<_, _>>()
    });

    VersionInfo {
        arguments: if let Some(partial_args) = partial_arguments {
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
        java_version: partial.java_version.or(merge.java_version),
        libraries: libraries
            .into_iter()
            .chain(partial.libraries)
            .map(|mut x| {
                x.name = x.name.replace(DUMMY_REPLACE_STRING, &merge_id);

                x
            })
            .collect::<Vec<_>>(),
        logging: partial.logging.or(merge.logging),
        main_class: if let Some(main_class) = partial.main_class {
            main_class
        } else {
            merge.main_class
        },
        minecraft_arguments: partial
            .minecraft_arguments
            .or(merge.minecraft_arguments),
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Groups of game versions that share compatible loader version profiles
    pub version_groups: Vec<VersionGroup>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
///  A game version of Minecraft
pub struct Version {
    /// The minecraft version ID
    pub id: String,
    /// Whether the release is stable or not
    pub stable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The loader profile group for this Minecraft version
    pub version_group: Option<String>,
    /// A map that contains loader versions for the game version
    pub loaders: Vec<LoaderVersion>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
/// A group of Minecraft versions that share loader version profiles
pub struct VersionGroup {
    /// The version group ID
    pub id: String,
    /// The loader versions for this version group
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

#[cfg(test)]
mod tests {
    use super::{PartialVersionInfo, merge_partial_version};
    use crate::minecraft::{Argument, ArgumentType, VersionInfo};
    use chrono::{DateTime, Utc};
    use serde_json::json;

    fn deserialize_version(
        date: &str,
    ) -> serde_json::Result<PartialVersionInfo> {
        serde_json::from_value(json!({
            "id": "1.21.11-forge-61.1.7",
            "inheritsFrom": "1.21.11",
            "time": date,
            "releaseTime": date,
            "libraries": [],
            "type": "release"
        }))
    }

    #[test]
    fn deserialize_loader_timestamps() {
        for (input, expected) in [
            ("2026-05-27T14:13:59+0:00", "2026-05-27T14:13:59Z"),
            ("2026-05-27T14:13:59+5:30", "2026-05-27T08:43:59Z"),
            ("2026-05-27T14:13:59-5:30", "2026-05-27T19:43:59Z"),
            ("2026-05-27T14:13:59.123+0:00", "2026-05-27T14:13:59.123Z"),
            ("2026-05-27T14:13:59+00:00", "2026-05-27T14:13:59Z"),
            ("2026-05-27T14:13:59+05:30", "2026-05-27T08:43:59Z"),
            ("2026-05-27T14:13:59+0200", "2026-05-27T12:13:59Z"),
            ("2026-05-27T14:13:59+0000", "2026-05-27T14:13:59Z"),
            ("2026-05-27T14:13:59Z", "2026-05-27T14:13:59Z"),
            ("2026-05-27T14:13:59", "2026-05-27T14:13:59Z"),
            ("2026-05-27T14:13:59.123", "2026-05-27T14:13:59.123Z"),
        ] {
            let version = deserialize_version(input).unwrap();
            let expected = expected.parse::<DateTime<Utc>>().unwrap();
            assert_eq!(version.time, expected, "{input}");
            assert_eq!(version.release_time, expected, "{input}");
        }
    }

    #[test]
    fn merge_loader_arguments_into_legacy_version() {
        let partial: PartialVersionInfo = serde_json::from_value(json!({
            "id": "fabric-loader-0.19.5-${modrinth.gameVersion}-ornithe-gen2",
            "javaVersion": { "component": "java-runtime-epsilon", "majorVersion": 25 },
            "inheritsFrom": "${modrinth.gameVersion}-vanilla",
            "logging": { "client": {
                "argument": "-Dlog4j.configurationFile=${path}",
                "file": { "id": "client-1.12.xml", "sha1": "", "size": 888, "url": "" },
                "type": "log4j2-xml"
            } },
            "time": "2026-05-27T14:13:59+0200",
            "releaseTime": "2026-05-27T14:13:59+0200",
            "arguments": {
                "game": [],
                "jvm": ["-Dfabric.gameVersion=${modrinth.gameVersion}"]
            },
            "libraries": [
                { "name": "net.ornithemc:calamus-intermediary-gen2:${modrinth.gameVersion}" },
                { "name": "com.google.code.gson:gson:2.10" },
                { "name": "org.lwjgl.lwjgl:lwjgl:2.9.4+legacyfabric.15" },
                {
                    "name": "org.lwjgl.lwjgl:lwjgl-platform:2.9.4+legacyfabric.15",
                    "natives": { "osx-arm64": "natives-osx" }
                }
            ],
            "type": "release"
        }))
        .unwrap();
        let vanilla: VersionInfo = serde_json::from_value(json!({
            "assetIndex": { "id": "1.8", "sha1": "", "size": 0, "totalSize": 0, "url": "" },
            "assets": "1.8",
            "downloads": {},
            "id": "1.8.9",
            "javaVersion": { "component": "jre-legacy", "majorVersion": 8 },
            "libraries": [
                { "name": "com.google.code.gson:gson:2.2.4" },
                { "name": "org.lwjgl.lwjgl:lwjgl:2.9.4-nightly-20150209" },
                { "name": "org.lwjgl.lwjgl:lwjgl:2.9.2-nightly-20140822" },
                { "name": "org.lwjgl.lwjgl:lwjgl-platform:2.9.2-nightly-20140822" }
            ],
            "mainClass": "net.minecraft.client.main.Main",
            "minecraftArguments": "--username ${auth_player_name}",
            "minimumLauncherVersion": 14,
            "releaseTime": "2015-12-03T09:24:39+00:00",
            "time": "2015-12-03T09:24:39+00:00",
            "type": "release"
        }))
        .unwrap();

        let merged = merge_partial_version(partial, vanilla);
        let arguments = merged.arguments.unwrap();

        assert!(!arguments.contains_key(&ArgumentType::Game));
        assert!(matches!(
            &arguments[&ArgumentType::Jvm][..],
            [Argument::Normal(x)] if x == "-Dfabric.gameVersion=1.8.9"
        ));
        assert_eq!(
            merged.minecraft_arguments.as_deref(),
            Some("--username ${auth_player_name}")
        );
        assert_eq!(
            merged
                .libraries
                .iter()
                .map(|x| x.name.as_str())
                .collect::<Vec<_>>(),
            [
                "net.ornithemc:calamus-intermediary-gen2:1.8.9",
                "com.google.code.gson:gson:2.10",
                "org.lwjgl.lwjgl:lwjgl:2.9.4+legacyfabric.15",
                "org.lwjgl.lwjgl:lwjgl-platform:2.9.4+legacyfabric.15"
            ]
        );
        assert_eq!(merged.java_version.unwrap().major_version, 25);
        assert!(merged.logging.is_some());
    }

    #[test]
    fn reject_invalid_loader_timestamps() {
        for input in [
            "not a date",
            "2026-05-27T14:13:59+0:60",
            "2026-05-27T14:13:59+0:00junk",
            "2026-05-27T14:13:59\"",
        ] {
            assert!(deserialize_version(input).is_err(), "{input}");
        }
    }
}
