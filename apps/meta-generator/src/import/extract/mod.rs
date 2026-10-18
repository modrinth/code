//! Logic for how to parse information downloaded from upstreams, and extract
//! it into some more usable, structured form (DB rows).

pub mod fabric;
pub mod fabriclike;
pub mod mojang;
pub mod quilt;

use anyhow::{Result, ensure};
use tracing::{info_span, warn};

use crate::{AppState, upstream::Upstreams, util::TaskContext};

pub async fn extract(app: &AppState, upstreams: Upstreams) -> Result<()> {
    let mut failed = Vec::new();
    if upstreams.mojang {
        mojang::extract(app)
            .transaction_context(|| info_span!("extracting Mojang metadata"))
            .await
            .inspect_err(|err| {
                warn!("error: {err:?}");
                failed.push("Mojang");
            })
            .ok();
    }

    if upstreams.fabric {
        fabric::extract(app)
            .transaction_context(|| info_span!("extracting Fabric metadata"))
            .await
            .inspect_err(|err| {
                warn!("error: {err:?}");
                failed.push("Fabric");
            })
            .ok();
    }

    if upstreams.quilt {
        quilt::extract(app)
            .transaction_context(|| info_span!("extracting Quilt metadata"))
            .await
            .inspect_err(|err| {
                warn!("error: {err:?}");
                failed.push("Quilt");
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

/// Reads each immutable content hash once, retaining first-seen order for errors.
async fn read_blobs(
    app: &AppState,
    hashes: impl IntoIterator<Item = crate::util::Sha256>,
) -> Result<std::collections::HashMap<crate::util::Sha256, Vec<u8>>> {
    use futures::{StreamExt, TryStreamExt, stream};
    let hashes = hashes.into_iter().collect::<indexmap::IndexSet<_>>();
    stream::iter(hashes)
        .map(|sha256| {
            async move {
                let bytes = app.cas.get(sha256).await?;
                anyhow::Ok((sha256, bytes.to_vec()))
            }
            .transaction_context(
                move || info_span!("reading import blob", %sha256),
            )
        })
        .buffered(app.concurrency.download.get())
        .try_collect()
        .await
}
