use anyhow::Result;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;

use crate::{task::DownloadRunContext, util::ErrorVec};

pub const FORGE_META_MANIFEST_URL: &str = "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/forge";
pub const NEOFORGE_META_MANIFEST_URL: &str = "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaManifest {
    pub is_snapshot: bool,
    pub versions: Vec<VersionName>,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct VersionName(pub String);

pub async fn download(
    cx: &mut DownloadRunContext<'_>,
    _errors: &mut ErrorVec,
) -> Result<()> {
    let forge_meta_manifest = cx
        .download_json::<MetaManifest>(FORGE_META_MANIFEST_URL)
        .context(info_span!("fetching Forge meta manifest"))
        .await?;
    let neoforge_meta_manifest = cx
        .download_json::<MetaManifest>(NEOFORGE_META_MANIFEST_URL)
        .context(info_span!("fetching NeoForge meta manifest"))
        .await?;
    info!(
        num_forge_versions = forge_meta_manifest.versions.len(),
        num_neoforge_versions = neoforge_meta_manifest.versions.len(),
        "downloaded NeoForge meta manifests"
    );

    Ok(())
}
