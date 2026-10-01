use anyhow::Result;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;
use url::Url;

use crate::{
    task::DownloadRunContext,
    util::{ErrorVec, MavenCoordinate, Sha1, Sha256},
};

pub const META_MANIFEST_URL: &str = "https://meta.quiltmc.org/v3/versions";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaManifest {
    pub game: Vec<GameVersion>,
    pub mappings: Vec<MappingVersion>,
    pub hashed: Vec<HashedVersion>,
    pub loader: Vec<LoaderVersion>,
    pub installer: Vec<InstallerVersion>,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct GameVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub maven: MavenCoordinate,
    pub version: MappingVersionName,
    pub game_version: GameVersionName,
    pub build: u32,
    pub separator: String,
    pub hashed: HashedVersionName,
    #[serde(rename = "file_size")]
    pub file_size: u64,
    pub hashes: Hashes,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct HashedVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashedVersion {
    pub maven: MavenCoordinate,
    pub version: HashedVersionName,
    pub file_size: u64,
    pub hashes: Hashes,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct LoaderVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoaderVersion {
    pub maven: MavenCoordinate,
    pub version: LoaderVersionName,
    pub build: u32,
    pub separator: String,
    pub file_size: u64,
    pub hashes: Hashes,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct InstallerVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallerVersion {
    pub maven: MavenCoordinate,
    pub version: InstallerVersionName,
    pub url: Url,
    pub file_size: u64,
    pub hashes: Hashes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hashes {
    pub sha1: Sha1,
    pub sha256: Sha256,
    pub sha512: String,
}

pub async fn download(
    cx: &mut DownloadRunContext<'_>,
    _errors: &mut ErrorVec,
) -> Result<()> {
    let meta_manifest = cx
        .download_json::<MetaManifest>(META_MANIFEST_URL)
        .context(info_span!("fetching meta manifest"))
        .await?;
    info!(
        num_game_versions = meta_manifest.game.len(),
        num_loader_versions = meta_manifest.loader.len(),
        "downloaded Quilt meta manifest"
    );

    Ok(())
}
