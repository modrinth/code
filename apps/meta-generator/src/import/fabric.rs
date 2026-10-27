use std::sync::atomic::{self, AtomicUsize};

use anyhow::{Result, ensure};
use futures::{StreamExt, stream};
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;

use crate::util::TaskContext;

use crate::{
    import::download::DownloadRunContext,
    model,
    upstream::{
        fabric::{CATALOG_URL, Catalog, LoaderVersion},
        fabriclike::{GameLoaderProfile, LoaderVersionName},
    },
};

pub const PROFILE_GAME_VERSION: &str = "1.21";
pub const MAVEN_URL: &str = "https://maven.fabricmc.net/";

pub fn profile_url(loader_version: &LoaderVersionName) -> String {
    format!(
        "{CATALOG_URL}/loader/{PROFILE_GAME_VERSION}/{loader_version}/profile/json"
    )
}

pub async fn download(cx: &mut DownloadRunContext<'_>) -> Result<()> {
    let initial_errors = cx.errors.len();
    let (catalog, sha256) = cx
        .download_json::<Catalog>(CATALOG_URL)
        .context(info_span!("fetching catalog"))
        .await?;
    info!(
        num_game_versions = catalog.game.len(),
        num_loader_versions = catalog.loader.len(),
        "downloaded Fabric catalog"
    );

    let num_total = catalog.loader.len();
    let num_done = AtomicUsize::new(0);
    let task = async |loader_version: LoaderVersion| {
        let version_name = &loader_version.version;
        let url = profile_url(version_name);

        cx.download_json::<GameLoaderProfile>(&url)
            .transaction_context(
                || info_span!("downloading loader profile", %version_name),
            )
            .await
            .inspect_err(|err| cx.errors.push(err))
            .ok();

        let num_done = num_done.fetch_add(1, atomic::Ordering::SeqCst) + 1;
        if num_done.is_multiple_of(10) || num_done == num_total {
            info!("downloaded {num_done}/{num_total} loader profiles");
        }
    };
    stream::iter(catalog.loader)
        .for_each_concurrent(cx.download_concurrency, task)
        .await;

    ensure!(
        cx.errors.len() == initial_errors,
        "Fabric item downloads failed; catalog not registered"
    );

    toasty::create!(model::FabricCatalog {
        download_run_id: cx.download_run_id,
        sha256,
    })
    .exec(*cx.conn().await)
    .context(info_span!("inserting catalog"))
    .await?;

    Ok(())
}
