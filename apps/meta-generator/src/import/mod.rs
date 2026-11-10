use anyhow::{Result, ensure};
use tracing::{info_span, warn};
use tracing_anyhow::FutureContext;

use crate::{AppState, upstream::Upstreams};

mod download;
mod extract;
pub mod fabric;
pub mod forge;
pub mod forgelike;
pub mod mojang;
pub mod neoforge;
pub mod quilt;

pub async fn run(app: AppState, upstreams: Upstreams) -> Result<()> {
    let completed = download::download_from_upstreams(&app, upstreams)
        .context(info_span!("downloading import sources"))
        .await?;
    let mut failed = [
        (upstreams.mojang, completed.mojang, "Mojang"),
        (upstreams.fabric, completed.fabric, "Fabric"),
        (upstreams.quilt, completed.quilt, "Quilt"),
        (upstreams.forge, completed.forge, "Forge"),
        (upstreams.neoforge, completed.neoforge, "NeoForge"),
    ]
    .into_iter()
    .filter(|(selected, completed, _)| *selected && !completed)
    .map(|(_, _, name)| format!("downloading {name}"))
    .collect::<Vec<_>>();

    // don't replace selected metadata for upstreams whose downloads failed
    extract::extract(&app, completed)
        .context(info_span!("extracting imported metadata"))
        .await
        .inspect_err(|err| {
            warn!("error: {err:?}");
            failed.push("extracting metadata".into());
        })
        .ok();
    if completed.forge || completed.neoforge {
        forgelike::extract_installers(&app, completed)
            .context(info_span!("extracting imported installers"))
            .await
            .inspect_err(|err| {
                warn!("error: {err:?}");
                failed.push("extracting installers".into());
            })
            .ok();
    }

    ensure!(failed.is_empty(), "import failed: {}", failed.join(", "));
    Ok(())
}
