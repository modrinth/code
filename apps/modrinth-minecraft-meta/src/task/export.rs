use anyhow::Result;
use tracing::{info_span, warn};
use tracing_anyhow::FutureContext;

use crate::{AppState, export, task::Upstreams};

pub async fn export(app: &mut AppState, upstreams: Upstreams) -> Result<()> {
    if upstreams.mojang {
        export::mojang::export(app)
            .context(info_span!("exporting Mojang artifacts"))
            .await
            .inspect_err(|err| warn!("error: {err:?}"))
            .ok();
    }

    if upstreams.fabric {
        export::fabric::export(app)
            .context(info_span!("exporting Fabric artifacts"))
            .await
            .inspect_err(|err| warn!("error: {err:?}"))
            .ok();
    }

    if upstreams.quilt {
        export::quilt::export(app)
            .context(info_span!("exporting Quilt artifacts"))
            .await
            .inspect_err(|err| warn!("error: {err:?}"))
            .ok();
    }

    Ok(())
}
