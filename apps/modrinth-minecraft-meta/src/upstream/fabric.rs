use anyhow::Result;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;
use url::Url;

use crate::{
    task::DownloadRunContext,
    util::{ErrorVec, MavenCoordinate},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaManifest {
    pub game: Vec<GameVersion>,
    pub mappings: Vec<MappingVersion>,
    pub intermediary: Vec<IntermediaryVersion>,
    pub loader: Vec<LoaderVersion>,
    pub installer: Vec<InstallerVersion>,
}

pub const META_MANIFEST_URL: &str = "https://meta.fabricmc.net/v2/versions";

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct GameVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameVersion {
    pub version: GameVersionName,
    pub stable: bool,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct MappingVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MappingVersion {
    pub game_version: GameVersionName,
    pub separator: String,
    pub build: u32,
    pub maven: MavenCoordinate,
    pub version: MappingVersionName,
    pub stable: bool,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct IntermediaryVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntermediaryVersion {
    pub maven: MavenCoordinate,
    pub version: IntermediaryVersionName,
    pub stable: bool,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct LoaderVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoaderVersion {
    pub separator: String,
    pub build: u32,
    pub maven: MavenCoordinate,
    pub version: LoaderVersionName,
    pub stable: bool,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct InstallerVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallerVersion {
    pub url: Url,
    pub maven: MavenCoordinate,
    pub version: InstallerVersionName,
    pub stable: bool,
}

pub async fn download(
    cx: &mut DownloadRunContext<'_>,
    errors: &mut ErrorVec,
) -> Result<()> {
    let meta_manifest = cx
        .download_json::<MetaManifest>(META_MANIFEST_URL)
        .context(info_span!("fetching meta manifest"))
        .await?;
    info!(
        num_game_versions = meta_manifest.game.len(),
        num_loader_versions = meta_manifest.loader.len(),
        "downloaded Fabric meta manifest"
    );

    Ok(())
}
