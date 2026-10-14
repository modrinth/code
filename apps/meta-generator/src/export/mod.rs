//! Schemas and logic for generating manifest artifacts consumed by the Modrinth
//! App.

use anyhow::Result;
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;

use crate::{AppState, upstream::Upstreams, util::ErrorAccumulator};

mod fabric;
mod fabriclike;
mod forgelike;
mod mojang;
mod quilt;

pub async fn run(app: AppState, upstreams: Upstreams) -> Result<()> {
    let mut errors = ErrorAccumulator::new();
    if upstreams.mojang {
        mojang::export(&app)
            .context(info_span!("exporting Mojang artifacts"))
            .await
            .inspect_err(|err| errors.push_mut(err))
            .ok();
    }

    if upstreams.fabric {
        fabric::export(&app)
            .context(info_span!("exporting Fabric artifacts"))
            .await
            .inspect_err(|err| errors.push_mut(err))
            .ok();
    }

    if upstreams.quilt {
        quilt::export(&app)
            .context(info_span!("exporting Quilt artifacts"))
            .await
            .inspect_err(|err| errors.push_mut(err))
            .ok();
    }

    if upstreams.forge || upstreams.neoforge {
        forgelike::export(&app, upstreams)
            .context(info_span!("exporting Forge-like artifacts"))
            .await
            .inspect_err(|err| errors.push_mut(err))
            .ok();
    }

    if let Some(fastly) = app.fastly.as_ref() {
        fastly
            .soft_purge_all()
            .context(info_span!("soft purging with Fastly"))
            .await
            .inspect(|()| info!("purged Fastly CDN"))
            .inspect_err(|err| errors.push_mut(err))
            .ok();
    } else {
        info!("Fastly client not configured; not purging CDN");
    }

    errors.finish_into_result()
}
