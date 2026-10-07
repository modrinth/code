use std::sync::atomic::{self, AtomicUsize};

use anyhow::Result;
use derive_more::Display;
use futures::{StreamExt, stream};
use serde::{Deserialize, Serialize};
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;
use url::Url;

use crate::{model, task::DownloadRunContext, util::MavenCoordinate};

pub const CATALOG_URL: &str = "https://meta.fabricmc.net/v2/versions";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    pub game: Vec<GameVersion>,
    pub mappings: Vec<MappingVersion>,
    pub intermediary: Vec<IntermediaryVersion>,
    pub loader: Vec<LoaderVersion>,
    pub installer: Vec<InstallerVersion>,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct GameVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameVersion {
    pub version: GameVersionName,
    pub stable: bool,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct MappingVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MappingVersion {
    pub game_version: GameVersionName,
    pub separator: String,
    pub build: u32,
    pub maven: MavenCoordinate,
    pub version: MappingVersionName,
    pub stable: bool,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct IntermediaryVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntermediaryVersion {
    pub maven: MavenCoordinate,
    pub version: IntermediaryVersionName,
    pub stable: bool,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct LoaderVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoaderVersion {
    pub separator: String,
    pub build: u32,
    pub maven: MavenCoordinate,
    pub version: LoaderVersionName,
    pub stable: bool,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct InstallerVersionName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallerVersion {
    pub url: Url,
    pub maven: MavenCoordinate,
    pub version: InstallerVersionName,
    pub stable: bool,
}

/// Example URL: <https://meta.fabricmc.net/v2/versions/loader/1.21/0.19.5/profile/json>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameLoaderProfile {
    pub id: String,
}

pub async fn download(cx: &mut DownloadRunContext<'_>) -> Result<()> {
    /// See `README.md` for an explanation of what we're doing here.
    const GAME_VERSION: &str = "1.21";

    let (catalog, sha256) = cx
        .download_json::<Catalog>(CATALOG_URL)
        .context(info_span!("fetching catalog"))
        .await?;
    info!(
        num_game_versions = catalog.game.len(),
        num_loader_versions = catalog.loader.len(),
        "downloaded Fabric catalog"
    );

    toasty::create!(model::FabricCatalog {
        download_run_id: cx.download_run_id,
        sha256,
    })
    .exec(*cx.conn().await)
    .context(info_span!("inserting catalog"))
    .await?;

    let num_total = catalog.loader.len();
    let num_done = AtomicUsize::new(0);
    let task = async |loader_version: LoaderVersion| {
        let version_name = &loader_version.version;
        let url = format!(
            "{CATALOG_URL}/loader/{GAME_VERSION}/{version_name}/profile/json"
        );

        cx.download_json::<GameLoaderProfile>(&url)
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

    Ok(())
}
