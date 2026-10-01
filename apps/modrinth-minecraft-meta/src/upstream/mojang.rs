use derive_more::Display;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::util::Sha1;

/// Manifest of all game versions.
///
/// Available at [`META_MANIFEST_URL`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaManifest {
    pub latest: Latest,
    pub versions: Vec<Version>,
}

pub const META_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

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

/// Manifest for a single gamae version.
///
/// Available at [`Version::url`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionManifest {}
