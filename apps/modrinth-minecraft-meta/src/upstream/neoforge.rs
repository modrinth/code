use anyhow::Result;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;

use crate::{model, task::DownloadRunContext};

pub const FORGE_CATALOG_URL: &str = "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/forge";
pub const NEOFORGE_CATALOG_URL: &str = "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Catalog {
    pub is_snapshot: bool,
    pub versions: Vec<VersionName>,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct VersionName(pub String);

pub async fn download(cx: &mut DownloadRunContext<'_>) -> Result<()> {
    let (forge_catalog, forge_sha256) = cx
        .download_json::<Catalog>(FORGE_CATALOG_URL)
        .context(info_span!("fetching Forge catalog"))
        .await?;
    let (neoforge_catalog, neoforge_sha256) = cx
        .download_json::<Catalog>(NEOFORGE_CATALOG_URL)
        .context(info_span!("fetching NeoForge catalog"))
        .await?;
    info!(
        num_forge_versions = forge_catalog.versions.len(),
        num_neoforge_versions = neoforge_catalog.versions.len(),
        "downloaded NeoForge catalogs"
    );

    toasty::create!(model::NeoforgeCatalog {
        download_run_id: cx.download_run_id,
        forge_sha256,
        neoforge_sha256,
    })
    .exec(*cx.conn().await)
    .context(info_span!("inserting catalog"))
    .await?;

    Ok(())
}
