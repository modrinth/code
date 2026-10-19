use anyhow::Result;
use derive_more::Display;
use indexmap::IndexMap;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::util::{Sha1, de_error};

#[derive(
    Debug,
    Display,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
)]
#[cfg_attr(feature = "toasty", derive(toasty::Embed))]
pub struct MinecraftVersionName(pub String);

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Latest {
    pub release: MinecraftVersionName,
    pub snapshot: MinecraftVersionName,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub id: MinecraftVersionName,
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
    pub arguments: Option<IndexMap<ArgumentType, Vec<Argument>>>,
    pub asset_index: AssetIndex,
    pub assets: String,
    pub downloads: IndexMap<DownloadType, Download>,
    pub id: MinecraftVersionName,
    pub java_version: Option<JavaVersion>,
    pub libraries: Vec<Library>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<IndexMap<LoggingSide, LoggingConfiguration>>,
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
    pub natives: Option<IndexMap<OperatingSystem, String>>,
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
    pub classifiers: Option<IndexMap<String, LibraryDownload>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryDownload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub sha1: Sha1,
    pub size: u32,
    // old `daedalus` has this field set as `url: String`
    // so it's required, but if it's empty then we do nothing with it.
    // we need this `serialize_with` for compat
    #[serde(
        default,
        deserialize_with = "deserialize_optional_url",
        serialize_with = "serialize_optional_url"
    )]
    pub url: Option<Url>,
}

fn serialize_optional_url<S>(
    url: &Option<Url>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(url.as_ref().map_or("", Url::as_str))
}

#[cfg(test)]
mod library_download_tests {
    use super::LibraryDownload;

    #[test]
    fn absent_urls_serialize_as_empty_strings() {
        for url in [
            None,
            Some(serde_json::Value::Null),
            Some(serde_json::json!("")),
        ] {
            let mut json = serde_json::json!({
                "sha1": "0000000000000000000000000000000000000000",
                "size": 0
            });
            if let Some(url) = url {
                json["url"] = url;
            }
            let download: LibraryDownload =
                serde_json::from_value(json).unwrap();
            assert!(download.url.is_none());
            assert_eq!(serde_json::to_value(download).unwrap()["url"], "");
        }
    }

    #[test]
    fn present_urls_are_preserved() {
        let json = serde_json::json!({
            "sha1": "0000000000000000000000000000000000000000",
            "size": 0,
            "url": "https://example.com/library.jar"
        });
        let download: LibraryDownload =
            serde_json::from_value(json.clone()).unwrap();
        assert_eq!(serde_json::to_value(download).unwrap(), json);
    }
}

fn deserialize_optional_url<'de, D>(
    deserializer: D,
) -> Result<Option<Url>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    match value.as_deref() {
        None | Some("") => Ok(None),
        Some(value) => Url::parse(value).map(Some).map_err(de_error::<D>),
    }
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

/// Version of the launcher-meta Minecraft wire format.
pub const CURRENT_FORMAT_VERSION: usize = 0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub hash: Sha1,
    pub size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetsIndex {
    pub objects: IndexMap<String, Asset>,
}

impl MinecraftVersionName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for MinecraftVersionName {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl PartialEq<str> for MinecraftVersionName {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for MinecraftVersionName {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl VersionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Release => "release",
            Self::Snapshot => "snapshot",
            Self::OldAlpha => "old_alpha",
            Self::OldBeta => "old_beta",
        }
    }
}
