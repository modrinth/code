use std::collections::HashMap;

use anyhow::Result;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;

use crate::{model, task::DownloadRunContext, util::ErrorVec};

pub const CATALOG_URL: &str = "https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json";

/// Forge loader manifest for all game versions, mapped to all loader versions
/// available for that game version.
///
/// Available at [`CATALOG_URL`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog(pub HashMap<GameVersionName, Vec<VersionName>>);

/// Name of a Minecraft game version.
#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct GameVersionName(pub String);

/// Name of a Minecraft game version, plus a `-`, plus the Forge loader
/// version name.
#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct VersionName(pub String);

pub async fn download(cx: &mut DownloadRunContext<'_>) -> Result<()> {
    let catalog = cx
        .download_json::<Catalog>(CATALOG_URL)
        .context(info_span!("fetching catalog"))
        .await?;

    let num_loader_versions = catalog.0.values().map(Vec::len).sum::<usize>();
    info!(
        num_game_versions = catalog.0.len(),
        num_loader_versions, "downloaded Forge catalog"
    );

    Ok(())
}

pub async fn extract(txn: &mut toasty::Transaction<'_>) -> Result<()> {
    let catalog_blobs = model::DownloadBlob::all()
        .filter_by_url(CATALOG_URL)
        .select(model::DownloadBlob::fields().json().json())
        .include(model::DownloadBlob::fields().json().json())
        .exec(txn)
        .context(info_span!("fetching all catalog blobs"))
        .await?;

    catalog_blobs.Ok(())
}
