use std::collections::HashMap;

use anyhow::Result;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;

use crate::{task::DownloadRunContext, util::ErrorVec};

pub const CATALOG_URL: &str = "https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog(pub HashMap<GameVersionName, Vec<VersionName>>);

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct GameVersionName(pub String);

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct VersionName(pub String);

pub async fn download(
    cx: &mut DownloadRunContext<'_>,
    _errors: &mut ErrorVec,
) -> Result<()> {
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
