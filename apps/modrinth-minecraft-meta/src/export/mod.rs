//! Schemas and logic for generating manifest artifacts consumed by the Modrinth
//! App.

use anyhow::{Result, ensure};
use tracing::{info_span, warn};
use tracing_anyhow::FutureContext;

use crate::{AppState, upstream::Upstreams};

mod fabric;
mod fabriclike;
mod forgelike;
mod mojang;
mod ornithe;
mod quilt;

pub async fn run(app: AppState, upstreams: Upstreams) -> Result<()> {
    let mut failed = Vec::new();
    if upstreams.mojang {
        mojang::export(&app)
            .context(info_span!("exporting Mojang artifacts"))
            .await
            .inspect_err(|err| {
                warn!("error: {err:?}");
                failed.push("Mojang");
            })
            .ok();
    }

    if upstreams.fabric {
        fabric::export(&app)
            .context(info_span!("exporting Fabric artifacts"))
            .await
            .inspect_err(|err| {
                warn!("error: {err:?}");
                failed.push("Fabric");
            })
            .ok();
    }

    if upstreams.quilt {
        quilt::export(&app)
            .context(info_span!("exporting Quilt artifacts"))
            .await
            .inspect_err(|err| {
                warn!("error: {err:?}");
                failed.push("Quilt");
            })
            .ok();
    }

    if upstreams.ornithe {
        ornithe::export(&app)
            .context(info_span!("exporting Ornithe artifacts"))
            .await
            .inspect_err(|err| {
                warn!("error: {err:?}");
                failed.push("Ornithe");
            })
            .ok();
    }

    if upstreams.forge || upstreams.neoforge {
        forgelike::export(&app, upstreams)
            .context(info_span!("exporting Forge-like artifacts"))
            .await
            .inspect_err(|err| {
                warn!("error: {err:?}");
                failed.push("Forge/NeoForge");
            })
            .ok();
    }

    ensure!(failed.is_empty(), "export failed for {}", failed.join(", "));
    Ok(())
}
