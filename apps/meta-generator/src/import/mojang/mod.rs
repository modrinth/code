use std::sync::atomic::{self, AtomicUsize};

use anyhow::{Result, ensure};
use futures::{StreamExt, stream};
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;

use crate::util::TaskContext;

use crate::{
    import::download::DownloadRunContext,
    model,
    upstream::mojang::{CATALOG_URL, Catalog, Version},
};

mod library;

pub use library::patch_library;

pub async fn download(cx: &mut DownloadRunContext<'_>) -> Result<()> {
    let initial_errors = cx.errors.len();
    let (catalog, sha256) = cx
        .download_json::<Catalog>(CATALOG_URL)
        .context(info_span!("fetching catalog"))
        .await?;
    info!(
        num_versions = catalog.versions.len(),
        "downloaded Mojang catalog"
    );

    let cataloged_sha1s = catalog
        .versions
        .iter()
        .map(|version| version.sha1)
        .collect::<Vec<_>>();
    let existing_sha1s = model::BlobHash::all()
        .select(model::BlobHash::fields().sha1())
        .filter(model::BlobHash::fields().sha1().in_list(cataloged_sha1s))
        .exec(*cx.conn().await)
        .context(info_span!("fetching existing sha1s"))
        .await?;
    let num_versions = catalog.versions.len();
    let missing_versions = catalog
        .versions
        .into_iter()
        .filter(|version| !existing_sha1s.contains(&version.sha1))
        .collect::<Vec<_>>();
    info!(
        "catalog contains {num_versions} versions, of which {} are missing; downloading",
        missing_versions.len()
    );

    let num_total = missing_versions.len();
    let num_done = AtomicUsize::new(0);
    let task = async |version: Version| {
        cx.download_blob(version.url.clone())
            .transaction_context(
                || info_span!("downloading version", %version.id),
            )
            .await
            .inspect_err(|err| cx.errors.push(err))
            .ok();

        let num_done = num_done.fetch_add(1, atomic::Ordering::SeqCst) + 1;
        if num_done.is_multiple_of(100) || num_done == num_total {
            info!("downloaded {num_done}/{num_total} versions");
        }
    };
    stream::iter(missing_versions)
        .for_each_concurrent(cx.download_concurrency, task)
        .await;

    ensure!(
        cx.errors.len() == initial_errors,
        "Mojang item downloads failed; catalog not registered"
    );

    toasty::create!(model::MojangCatalog {
        download_run_id: cx.download_run_id,
        sha256,
    })
    .exec(*cx.conn().await)
    .context(info_span!("inserting catalog"))
    .await?;

    Ok(())
}
