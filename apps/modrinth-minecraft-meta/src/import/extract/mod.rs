//! Logic for how to parse information downloaded from upstreams, and extract
//! it into some more usable, structured form (DB rows).

pub mod fabric;
pub mod fabriclike;
pub mod mojang;
pub mod ornithe;
pub mod quilt;

use anyhow::{Result, ensure};
use tracing::{info_span, warn};
use tracing_anyhow::FutureContext;

use crate::{AppState, upstream::Upstreams};

pub async fn extract(app: &AppState, upstreams: Upstreams) -> Result<()> {
    let mut failed = Vec::new();
    if upstreams.mojang {
        mojang::extract(app)
            .context(info_span!("extracting Mojang metadata"))
            .await
            .inspect_err(|err| {
                warn!("error: {err:?}");
                failed.push("Mojang");
            })
            .ok();
    }

    if upstreams.fabric {
        fabric::extract(app)
            .context(info_span!("extracting Fabric metadata"))
            .await
            .inspect_err(|err| {
                warn!("error: {err:?}");
                failed.push("Fabric");
            })
            .ok();
    }

    if upstreams.quilt {
        quilt::extract(app)
            .context(info_span!("extracting Quilt metadata"))
            .await
            .inspect_err(|err| {
                warn!("error: {err:?}");
                failed.push("Quilt");
            })
            .ok();
    }

    if upstreams.ornithe {
        ornithe::extract(app)
            .context(info_span!("extracting Ornithe metadata"))
            .await
            .inspect_err(|err| {
                warn!("error: {err:?}");
                failed.push("Ornithe");
            })
            .ok();
    }

    ensure!(
        failed.is_empty(),
        "metadata extraction failed for {}",
        failed.join(", ")
    );
    Ok(())
}
