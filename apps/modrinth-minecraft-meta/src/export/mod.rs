//! Schemas and logic for generating manifest artifacts consumed by the Modrinth
//! App.

use anyhow::Result;
use tracing::{info_span, warn};
use tracing_anyhow::FutureContext;

use crate::{AppState, upstream::Upstreams};

mod fabric;
mod fabriclike;
mod mojang;
mod quilt;

pub async fn export(app: &mut AppState, upstreams: Upstreams) -> Result<()> {
    if upstreams.mojang {
        mojang::export(app)
            .context(info_span!("exporting Mojang artifacts"))
            .await
            .inspect_err(|err| warn!("error: {err:?}"))
            .ok();
    }

    if upstreams.fabric {
        fabric::export(app)
            .context(info_span!("exporting Fabric artifacts"))
            .await
            .inspect_err(|err| warn!("error: {err:?}"))
            .ok();
    }

    if upstreams.quilt {
        quilt::export(app)
            .context(info_span!("exporting Quilt artifacts"))
            .await
            .inspect_err(|err| warn!("error: {err:?}"))
            .ok();
    }

    Ok(())
}
