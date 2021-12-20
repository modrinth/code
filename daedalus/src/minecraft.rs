use crate::modded::{Processor, SidedDataEntry};
use crate::{download_file, Error};
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
    /// (Modrinth Provided) The link to the assets index for this version
    /// This is only available when using the Modrinth mirror
    pub assets_index_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// (Modrinth Provided) The SHA1 hash of the assets index for this version
    /// This is only available when using the Modrinth mirror
    pub assets_index_sha1: Option<String>,
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
    "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json";

/// Fetches a version manifest from the specified URL. If no URL is specified, the default is used.
pub async fn fetch_version_manifest(url: Option<&str>) -> Result<VersionManifest, Error> {
    Ok(serde_json::from_slice(
        &download_file(url.unwrap_or(VERSION_MANIFEST_URL), None).await?,
    )?)
}

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

#[derive(Serialize, Deserialize, Debug)]
/// Download information of a library
pub struct LibraryDownload {
    /// The path that the library should be saved to
    pub path: String,
    /// The SHA1 hash of the library
    pub sha1: String,
    /// The size of the library
    pub size: u32,
    /// The URL where the library can be downloaded
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
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
#[serde(rename_all = "snake_case")]
/// An enum representing the different types of operating systems
pub enum Os {
    /// MacOS
    Osx,
    /// Windows
    Windows,
    /// Linux and its derivatives
    Linux,
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
    /// Whether the user is using the demo resolution
    pub has_demo_resolution: Option<bool>,
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

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
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
}

fn default_include_in_classpath() -> bool {
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

/// Fetches detailed information about a version from the manifest
pub async fn fetch_version_info(version: &Version) -> Result<VersionInfo, Error> {
    Ok(serde_json::from_slice(
        &download_file(&version.url, Some(&version.sha1)).await?,
    )?)
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

/// Fetches the assets index from the version info
pub async fn fetch_assets_index(version: &VersionInfo) -> Result<AssetsIndex, Error> {
    Ok(serde_json::from_slice(
        &download_file(&version.asset_index.url, Some(&version.asset_index.sha1)).await?,
    )?)
}
