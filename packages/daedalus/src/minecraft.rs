use crate::modded::{Processor, SidedDataEntry};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The latest version of the format the model structs deserialize to
pub const CURRENT_FORMAT_VERSION: usize = 0;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
/// The version type
pub enum VersionType {
    /// A major version, which is stable for all players to use
    Release,
    /// An experimental version, which is unstable and used for feature previews and beta testing
    Snapshot,
    /// The oldest versions before the game was released
    OldAlpha,
    /// Early versions of the game
    OldBeta,
}

impl VersionType {
    /// Converts the version type to a string
    pub fn as_str(&self) -> &'static str {
        match self {
            VersionType::Release => "release",
            VersionType::Snapshot => "snapshot",
            VersionType::OldAlpha => "old_alpha",
            VersionType::OldBeta => "old_beta",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
/// A game version of Minecraft
pub struct Version {
    /// A unique identifier of the version
    pub id: String,
    #[serde(rename = "type")]
    /// The release type of the version
    pub type_: VersionType,
    /// A link to additional information about the version
    pub url: String,
    /// The latest time a file in this version was updated
    pub time: DateTime<Utc>,
    /// The time this version was released
    pub release_time: DateTime<Utc>,
    /// The SHA1 hash of the additional information about the version
    pub sha1: String,
    /// Whether the version supports the latest player safety features
    pub compliance_level: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// (Modrinth Provided) The SHA1 hash of the original unmodified Minecraft versions JSON
    pub original_sha1: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// The latest snapshot and release of the game
pub struct LatestVersion {
    /// The version id of the latest release
    pub release: String,
    /// The version id of the latest snapshot
    pub snapshot: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Data of all game versions of Minecraft
pub struct VersionManifest {
    /// A struct containing the latest snapshot and release of the game
    pub latest: LatestVersion,
    /// A list of game versions of Minecraft
    pub versions: Vec<Version>,
}

/// The URL to the version manifest
pub const VERSION_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
/// Information about the assets of the game
pub struct AssetIndex {
    /// The game version ID the assets are for
    pub id: String,
    /// The SHA1 hash of the assets index
    pub sha1: String,
    /// The size of the assets index
    pub size: u32,
    /// The size of the game version's assets
    pub total_size: u32,
    /// A URL to a file which contains information about the version's assets
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
/// The type of download
pub enum DownloadType {
    /// The download is for the game client
    Client,
    /// The download is mappings for the game
    ClientMappings,
    /// The download is for the game server
    Server,
    /// The download is mappings for the game server
    ServerMappings,
    /// The download is for the windows server
    WindowsServer,
}

#[derive(Serialize, Deserialize, Debug)]
/// Download information of a file
pub struct Download {
    /// The SHA1 hash of the file
    pub sha1: String,
    /// The size of the file
    pub size: u32,
    /// The URL where the file can be downloaded
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Download information of a library
pub struct LibraryDownload {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The path that the library should be saved to
    pub path: Option<String>,
    /// The SHA1 hash of the library
    pub sha1: String,
    /// The size of the library
    pub size: u32,
    /// The URL where the library can be downloaded
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A list of files that should be downloaded for libraries
pub struct LibraryDownloads {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The primary library artifact
    pub artifact: Option<LibraryDownload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Conditional files that may be needed to be downloaded alongside the library
    /// The HashMap key specifies a classifier as additional information for downloading files
    pub classifiers: Option<HashMap<String, LibraryDownload>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
/// The action a rule can follow
pub enum RuleAction {
    /// The rule's status allows something to be done
    Allow,
    /// The rule's status disallows something to be done
    Disallow,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone)]
#[serde(rename_all = "kebab-case")]
/// An enum representing the different types of operating systems
pub enum Os {
    /// MacOS (x86)
    Osx,
    /// M1-Based Macs
    OsxArm64,
    /// Windows (x86)
    Windows,
    /// Windows ARM
    WindowsArm64,
    /// Linux (x86) and its derivatives
    Linux,
    /// Linux ARM 64
    LinuxArm64,
    /// Linux ARM 32
    LinuxArm32,
    /// The OS is unknown
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A rule which depends on what OS the user is on
pub struct OsRule {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The name of the OS
    pub name: Option<Os>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The version of the OS. This is normally a RegEx
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The architecture of the OS
    pub arch: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A rule which depends on the toggled features of the launcher
pub struct FeatureRule {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Whether the user is in demo mode
    pub is_demo_user: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Whether the user is using a custom resolution
    pub has_custom_resolution: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Whether the launcher has quick plays support
    pub has_quick_plays_support: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Whether the instance is being launched to a single-player world
    pub is_quick_play_singleplayer: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Whether the instance is being launched to a multi-player world
    pub is_quick_play_multiplayer: Option<bool>,
    ///  Whether the instance is being launched to a realms world
    pub is_quick_play_realms: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A rule deciding whether a file is downloaded, an argument is used, etc.
pub struct Rule {
    /// The action the rule takes
    pub action: RuleAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The OS rule
    pub os: Option<OsRule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The feature rule
    pub features: Option<FeatureRule>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Information delegating the extraction of the library
pub struct LibraryExtract {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Files/Folders to be excluded from the extraction of the library
    pub exclude: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
/// Information about the java version the game needs
pub struct JavaVersion {
    /// The component needed for the Java installation
    pub component: String,
    /// The major Java version number
    pub major_version: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A library which the game relies on to run
pub struct Library {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The files the library has
    pub downloads: Option<LibraryDownloads>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Rules of the extraction of the file
    pub extract: Option<LibraryExtract>,
    /// The maven name of the library. The format is `groupId:artifactId:version`
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The URL to the repository where the library can be downloaded
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Native files that the library relies on
    pub natives: Option<HashMap<Os, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Rules deciding whether the library should be downloaded or not
    pub rules: Option<Vec<Rule>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// SHA1 Checksums for validating the library's integrity. Only present for forge libraries
    pub checksums: Option<Vec<String>>,
    #[serde(default = "default_include_in_classpath")]
    /// Whether the library should be included in the classpath at the game's launch
    pub include_in_classpath: bool,
    #[serde(default = "default_downloadable")]
    /// Whether the library should be downloaded
    pub downloadable: bool,
}

#[derive(Deserialize, Debug, Clone)]
/// A partial library which should be merged with a full library
pub struct PartialLibrary {
    /// The files the library has
    pub downloads: Option<LibraryDownloads>,
    /// Rules of the extraction of the file
    pub extract: Option<LibraryExtract>,
    /// The maven name of the library. The format is `groupId:artifactId:version`
    pub name: Option<String>,
    /// The URL to the repository where the library can be downloaded
    pub url: Option<String>,
    /// Native files that the library relies on
    pub natives: Option<HashMap<Os, String>>,
    /// Rules deciding whether the library should be downloaded or not
    pub rules: Option<Vec<Rule>>,
    /// SHA1 Checksums for validating the library's integrity. Only present for forge libraries
    pub checksums: Option<Vec<String>>,
    /// Whether the library should be included in the classpath at the game's launch
    pub include_in_classpath: Option<bool>,
}

/// Merges a partial library to make a complete library
pub fn merge_partial_library(
    partial: PartialLibrary,
    mut merge: Library,
) -> Library {
    if let Some(downloads) = partial.downloads {
        if let Some(merge_downloads) = &mut merge.downloads {
            if let Some(artifact) = downloads.artifact {
                merge_downloads.artifact = Some(artifact);
            }
            if let Some(classifiers) = downloads.classifiers {
                if let Some(merge_classifiers) =
                    &mut merge_downloads.classifiers
                {
                    for classifier in classifiers {
                        merge_classifiers.insert(classifier.0, classifier.1);
                    }
                } else {
                    merge_downloads.classifiers = Some(classifiers);
                }
            }
        } else {
            merge.downloads = Some(downloads)
        }
    }
    if let Some(extract) = partial.extract {
        merge.extract = Some(extract)
    }
    if let Some(name) = partial.name {
        merge.name = name
    }
    if let Some(url) = partial.url {
        merge.url = Some(url)
    }
    if let Some(natives) = partial.natives {
        if let Some(merge_natives) = &mut merge.natives {
            for native in natives {
                merge_natives.insert(native.0, native.1);
            }
        } else {
            merge.natives = Some(natives);
        }
    }
    if let Some(rules) = partial.rules {
        if let Some(merge_rules) = &mut merge.rules {
            for rule in rules {
                merge_rules.push(rule);
            }
        } else {
            merge.rules = Some(rules)
        }
    }
    if let Some(checksums) = partial.checksums {
        merge.checksums = Some(checksums)
    }
    if let Some(include_in_classpath) = partial.include_in_classpath {
        merge.include_in_classpath = include_in_classpath
    }

    merge
}

fn default_include_in_classpath() -> bool {
    true
}
fn default_downloadable() -> bool {
    true
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
/// A container for an argument or multiple arguments
pub enum ArgumentValue {
    /// The container has one argument
    Single(String),
    /// The container has multiple arguments
    Many(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
/// A command line argument passed to a program
pub enum Argument {
    /// An argument which is applied no matter what
    Normal(String),
    /// An argument which is only applied if certain conditions are met
    Ruled {
        /// The rules deciding whether the argument(s) is used or not
        rules: Vec<Rule>,
        /// The container of the argument(s) that should be applied accordingly
        value: ArgumentValue,
    },
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone, Copy)]
#[serde(rename_all = "snake_case")]
/// The type of argument
pub enum ArgumentType {
    /// The argument is passed to the game
    Game,
    /// The argument is passed to the JVM
    Jvm,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
/// The physical side of the logging configuration
pub enum LoggingSide {
    /// Client logging configuration
    Client,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// File download information for a logging configuration
pub struct LogConfigDownload {
    /// The path that the logging configuration should be saved to
    pub id: String,
    /// The SHA1 hash of the logging configuration
    pub sha1: String,
    /// The size of the logging configuration
    pub size: u32,
    /// The URL where the logging configuration can be downloaded
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(
    tag = "type",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
/// Information about a version's logging configuration
pub enum LoggingConfiguration {
    /// Use a log4j2 XML log config file
    Log4j2Xml {
        /// The JVM argument for passing the file to the Java process
        argument: String,
        /// The config file to download
        file: LogConfigDownload,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
/// Information about a version
pub struct VersionInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Arguments passed to the game or JVM
    pub arguments: Option<HashMap<ArgumentType, Vec<Argument>>>,
    /// Assets for the game
    pub asset_index: AssetIndex,
    /// The version ID of the assets
    pub assets: String,
    /// Game downloads of the version
    pub downloads: HashMap<DownloadType, Download>,
    /// The version ID of the version
    pub id: String,

    /// The Java version this version supports
    pub java_version: Option<JavaVersion>,
    /// Libraries that the version depends on
    pub libraries: Vec<Library>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The logging configuration data for the game
    pub logging: Option<HashMap<LoggingSide, LoggingConfiguration>>,
    /// The classpath to the main class to launch the game
    pub main_class: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// (Legacy) Arguments passed to the game
    pub minecraft_arguments: Option<String>,
    /// The minimum version of the Minecraft Launcher that can run this version of the game
    pub minimum_launcher_version: u32,
    /// The time that the version was released
    pub release_time: DateTime<Utc>,
    /// The latest time a file in this version was updated
    pub time: DateTime<Utc>,
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

#[derive(Serialize, Deserialize, Debug)]
/// An asset of the game
pub struct Asset {
    /// The SHA1 hash of the asset file
    pub hash: String,
    /// The size of the asset file
    pub size: u32,
}

#[derive(Serialize, Deserialize, Debug)]
/// An index containing all assets the game needs
pub struct AssetsIndex {
    /// A hashmap containing the filename (key) and asset (value)
    pub objects: HashMap<String, Asset>,
}
