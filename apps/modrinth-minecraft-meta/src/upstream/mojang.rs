use std::collections::HashMap;

use anyhow::Result;
use derive_more::Display;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;
use url::Url;

use crate::{model, task::DownloadRunContext, util::Sha1};

pub const CATALOG_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

/// Catalog of all game versions.
///
/// Available at [`CATALOG_URL`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Catalog {
    pub latest: Latest,
    pub versions: Vec<Version>,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct VersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Latest {
    pub release: VersionName,
    pub snapshot: VersionName,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub id: VersionName,
    #[serde(rename = "type")]
    pub ty: VersionType,
    pub url: Url,
    pub time: Timestamp,
    pub release_time: Timestamp,
    pub sha1: Sha1,
    pub compliance_level: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VersionType {
    Release,
    Snapshot,
    OldAlpha,
    OldBeta,
}

/// Manifest for a single game version.
///
/// Available at [`Version::url`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionManifest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<HashMap<ArgumentType, Vec<Argument>>>,
    pub asset_index: AssetIndex,
    pub assets: String,
    pub downloads: HashMap<DownloadType, Download>,
    pub id: VersionName,
    pub java_version: Option<JavaVersion>,
    pub libraries: Vec<Library>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<HashMap<LoggingSide, LoggingConfiguration>>,
    pub main_class: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_arguments: Option<String>,
    pub minimum_launcher_version: u32,
    pub release_time: Timestamp,
    pub time: Timestamp,
    #[serde(rename = "type")]
    pub ty: VersionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndex {
    pub id: String,
    pub sha1: Sha1,
    pub size: u32,
    pub total_size: u32,
    pub url: Url,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadType {
    Client,
    ClientMappings,
    Server,
    ServerMappings,
    WindowsServer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Download {
    pub sha1: Sha1,
    pub size: u32,
    pub url: Url,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    pub component: String,
    pub major_version: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArgumentType {
    Game,
    Jvm,
    DefaultUserJvm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Argument {
    Normal(String),
    Ruled {
        #[serde(default)]
        rules: Vec<Rule>,
        value: ArgumentValue,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArgumentValue {
    Single(String),
    Many(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub action: RuleAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<OperatingSystemRule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<FeatureRule>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleAction {
    Allow,
    Disallow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatingSystemRule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<OperatingSystem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OperatingSystem {
    Osx,
    OsxArm64,
    Windows,
    WindowsArm64,
    Linux,
    LinuxArm64,
    LinuxArm32,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureRule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_demo_user: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_custom_resolution: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_quick_plays_support: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_quick_play_singleplayer: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_quick_play_multiplayer: Option<bool>,
    pub is_quick_play_realms: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloads: Option<LibraryDownloads>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extract: Option<LibraryExtract>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub natives: Option<HashMap<OperatingSystem, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<Rule>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksums: Option<Vec<String>>,
    #[serde(default = "default_true")]
    pub include_in_classpath: bool,
    #[serde(default = "default_true")]
    pub downloadable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryDownloads {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<LibraryDownload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classifiers: Option<HashMap<String, LibraryDownload>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryDownload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub sha1: Sha1,
    pub size: u32,
    pub url: Url,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryExtract {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoggingSide {
    Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfigDownload {
    pub id: String,
    pub sha1: Sha1,
    pub size: u32,
    pub url: Url,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum LoggingConfiguration {
    Log4j2Xml {
        argument: String,
        file: LogConfigDownload,
    },
}

fn default_true() -> bool {
    true
}

pub async fn download(cx: &mut DownloadRunContext<'_>) -> Result<()> {
    let (catalog, catalog_sha256) = cx
        .download_json::<Catalog>(CATALOG_URL)
        .context(info_span!("fetching catalog"))
        .await?;
    info!(
        num_versions = catalog.versions.len(),
        "downloaded Mojang catalog"
    );

    toasty::create!(model::MojangCatalog {
        download_run_id: cx.download_run_id,
        sha256: catalog_sha256,
    })
    .exec(cx.conn)
    .context(info_span!("inserting Mojang catalog"))
    .await?;

    let mut num_done = 0usize;
    for version in catalog.versions {
        cx.download_blob()
            .skip_if_sha1(version.sha1)
            .url(version.url)
            .call()
            .await
            .inspect_err(|err| cx.errors.push(err))
            .ok();

        num_done += 1;
        if num_done.is_multiple_of(100) {
            info!("downloaded {num_done} versions");
        }
    }

    Ok(())
}
