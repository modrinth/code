use anyhow::Result;
use derive_more::Display;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;
use url::Url;

use crate::{
    task::DownloadRunContext,
    util::{ErrorVec, Sha1},
};

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

/// Manifest for a single gamae version.
///
/// Available at [`Version::url`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionManifest {}

pub async fn download(
    cx: &mut DownloadRunContext<'_>,
    errors: &mut ErrorVec,
) -> Result<()> {
    let catalog = cx
        .download_json::<Catalog>(CATALOG_URL)
        .context(info_span!("fetching catalog"))
        .await?;
    info!(
        num_versions = catalog.versions.len(),
        "downloaded Mojang catalog"
    );

    for (index, version) in catalog.versions.into_iter().enumerate() {
        cx.download_json::<VersionManifest>(
            version.url.clone(),
        )
        .context(
            info_span!("fetching version manifest", version.id = %version.id, %version.url),
        )
        .await
        .inspect_err(|err| errors.push(err))
        .ok();

        if (index + 1) % 10 == 0 {
            info!("downloaded {index} version manifests");
        }
    }

    Ok(())
}
