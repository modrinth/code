//! Schemas and logic for generating manifest artifacts consumed by the Modrinth
//! App.

use anyhow::Result;
use futures::{StreamExt, TryStreamExt, stream};
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;

use crate::{
    AppState,
    store::{ContentType, PutStatus},
    upstream::Upstreams,
    util::{ErrorAccumulator, TaskContext},
};

mod fabric;
mod fabriclike;
mod forgelike;
mod maven;
mod mojang;
mod quilt;

pub async fn run(app: AppState, upstreams: Upstreams) -> Result<()> {
    let maven = maven::MavenExports::load(&app.db).await?;
    let mut errors = ErrorAccumulator::new();
    let mut status = PutStatus::Unchanged;
    if upstreams.mojang {
        mojang::export(&app)
            .transaction_context(|| info_span!("exporting Mojang artifacts"))
            .await
            .inspect(|new| latch(&mut status, *new))
            .inspect_err(|err| errors.push_mut(err))
            .ok();
    }

    if upstreams.fabric {
        fabric::export(&app, &maven)
            .transaction_context(|| info_span!("exporting Fabric artifacts"))
            .await
            .inspect(|new| latch(&mut status, *new))
            .inspect_err(|err| errors.push_mut(err))
            .ok();
    }

    if upstreams.quilt {
        quilt::export(&app, &maven)
            .transaction_context(|| info_span!("exporting Quilt artifacts"))
            .await
            .inspect(|new| latch(&mut status, *new))
            .inspect_err(|err| errors.push_mut(err))
            .ok();
    }

    if upstreams.forge || upstreams.neoforge {
        forgelike::export(&app, &maven, upstreams)
            .transaction_context(|| {
                info_span!("exporting Forge-like artifacts")
            })
            .await
            .inspect(|new| latch(&mut status, *new))
            .inspect_err(|err| errors.push_mut(err))
            .ok();
    }

    if status == PutStatus::Unchanged {
        info!("no public files changed; not purging CDN");
    } else if let Some(fastly) = app.fastly.as_ref() {
        fastly
            .soft_purge_all()
            .context(info_span!("soft purging with Fastly"))
            .await
            .inspect(|()| info!("purged Fastly CDN"))?;
    } else {
        info!("Fastly client not configured; not purging CDN");
    }

    errors.finish_into_result()
}

async fn publish_json_files(
    app: &AppState,
    files: impl IntoIterator<Item = (String, Vec<u8>)>,
) -> Result<PutStatus> {
    stream::iter(files.into_iter().map(|(path, data)| async move {
        app.public_blobs
            .put(&path, &data, ContentType::Json)
            .transaction_context(
                || info_span!("writing JSON to blob store", %path),
            )
            .await
    }))
    .buffer_unordered(app.concurrency.download.get())
    .try_fold(PutStatus::Unchanged, |mut status, changed| async move {
        latch(&mut status, changed);
        Ok(status)
    })
    .await
}

fn latch(status: &mut PutStatus, new: PutStatus) {
    *status = match (*status, new) {
        (PutStatus::Changed, _) => PutStatus::Changed,
        (PutStatus::Unchanged, new) => new,
    };
}
