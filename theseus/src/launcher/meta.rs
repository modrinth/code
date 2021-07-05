use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum VersionType {
    Release,
    Snapshot,
    OldAlpha,
    OldBeta,
}

impl VersionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            VersionType::Release => "release",
            VersionType::Snapshot => "snapshot",
            VersionType::OldAlpha => "old_alpha",
            VersionType::OldBeta => "old_beta",
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: VersionType,
    pub url: String,
    pub time: DateTime<Utc>,
    pub release_time: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
pub struct LatestVersion {
    pub release: String,
    pub snapshot: String,
}

#[derive(Deserialize, Debug)]
pub struct VersionManifest {
    pub latest: LatestVersion,
    pub versions: Vec<Version>,
}

pub async fn fetch_version_manifest() -> Result<VersionManifest, reqwest::Error> {
    reqwest::get("https://launchermeta.mojang.com/mc/game/version_manifest.json")
        .await?
        .json()
        .await
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u32,
    pub total_size: u32,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DownloadType {
    Client,
    ClientMappings,
    Server,
    ServerMappings,
    WindowsServer,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Download {
    pub sha1: String,
    pub size: u32,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LibraryDownload {
    pub path: String,
    pub sha1: String,
    pub size: u32,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LibraryDownloads {
    pub artifact: Option<LibraryDownload>,
    pub classifiers: Option<HashMap<String, LibraryDownload>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum RuleAction {
    Allow,
    Disallow,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Os {
    Osx,
    Windows,
    Linux,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OsRule {
    pub name: Option<Os>,
    pub version: Option<String>,
    pub arch: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FeatureRule {
    pub is_demo_user: Option<bool>,
    pub has_custom_resolution: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Rule {
    pub action: RuleAction,
    pub os: Option<OsRule>,
    pub feature: Option<FeatureRule>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LibraryExtract {
    pub exclude: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Library {
    pub downloads: LibraryDownloads,
    pub extract: Option<LibraryExtract>,
    pub name: String,
    pub natives: Option<HashMap<Os, String>>,
    pub rules: Option<Vec<Rule>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ArgumentValue {
    Single(String),
    Many(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Argument {
    Normal(String),
    Ruled {
        rules: Vec<Rule>,
        value: ArgumentValue,
    },
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ArgumentType {
    Game,
    Jvm,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
    pub arguments: Option<HashMap<ArgumentType, Vec<Argument>>>,
    pub asset_index: AssetIndex,
    pub assets: String,
    pub downloads: HashMap<DownloadType, Download>,
    pub id: String,
    pub libraries: Vec<Library>,
    pub main_class: String,
    pub minecraft_arguments: Option<String>,
    pub minimum_launcher_version: u32,
    pub release_time: DateTime<Utc>,
    pub time: DateTime<Utc>,
    #[serde(rename = "type")]
    pub type_: VersionType,
}

pub async fn fetch_version_info(version: &Version) -> Result<VersionInfo, reqwest::Error> {
    reqwest::get(&version.url).await?.json().await
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Asset {
    pub hash: String,
    pub size: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AssetsIndex {
    pub objects: HashMap<String, Asset>,
}

pub async fn fetch_assets_index(version: &VersionInfo) -> Result<AssetsIndex, reqwest::Error> {
    reqwest::get(&version.asset_index.url).await?.json().await
}
