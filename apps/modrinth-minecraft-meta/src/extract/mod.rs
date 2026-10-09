//! Logic for how to parse information downloaded from upstreams, and extract
//! it into some more usable, structured form (DB rows).

pub mod mojang;

use anyhow::Result;
use tracing::{info_span, warn};
use tracing_anyhow::FutureContext;

use crate::AppState;

pub async fn extract(app: &AppState) -> Result<()> {
    mojang::extract(app)
        .context(info_span!("extracting Mojang metadata"))
        .await
        .inspect_err(|err| warn!("error: {err:?}"))
        .ok();

    Ok(())
}
