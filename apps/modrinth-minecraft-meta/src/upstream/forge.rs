use std::collections::HashMap;

use anyhow::Result;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;

use crate::{task::DownloadRunContext, util::ErrorVec};

pub const META_MANIFEST_URL: &str = "https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaManifest(pub HashMap<GameVersionName, Vec<VersionName>>);

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
    let meta_manifest = cx
        .download_json::<MetaManifest>(META_MANIFEST_URL)
        .context(info_span!("fetching meta manifest"))
        .await?;
    let num_loader_versions =
        meta_manifest.0.values().map(Vec::len).sum::<usize>();
    info!(
        num_game_versions = meta_manifest.0.len(),
        num_loader_versions, "downloaded Forge meta manifest"
    );

    Ok(())
}
