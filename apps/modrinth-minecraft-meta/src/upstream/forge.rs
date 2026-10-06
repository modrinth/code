use std::collections::HashMap;

use anyhow::Result;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;

use crate::{
    model::{self, ForgeInstallerName},
    task::DownloadRunContext,
};

pub const CATALOG_URL: &str = "https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json";

/// Forge loader manifest for all game versions, mapped to all loader versions
/// available for that game version.
///
/// Available at [`CATALOG_URL`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog(pub HashMap<GameVersionName, Vec<ForgeInstallerName>>);

/// Name of a Minecraft game version.
#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct GameVersionName(pub String);

pub async fn download(cx: &mut DownloadRunContext<'_>) -> Result<()> {
    let (catalog, sha256) = cx
        .download_json::<Catalog>(CATALOG_URL)
        .context(info_span!("fetching catalog"))
        .await?;
    let raw_cataloged_versions =
        catalog.0.values().map(Vec::len).sum::<usize>();
    let cataloged_versions = catalog
        .0
        .into_values()
        .flatten()
        // certain version manifests are broken;
        // we have to explicitly exclude them
        .filter(|name| !BLACKLIST.contains(&name.0.as_str()))
        .collect::<Vec<_>>();
    info!(
        raw_cataloged_versions,
        num_cataloged_versions = cataloged_versions.len(),
        "downloaded Forge catalog"
    );

    toasty::create!(model::ForgeCatalog {
        download_run_id: cx.download_run_id,
        sha256,
    })
    .exec(cx.conn)
    .context(info_span!("inserting catalog"))
    .await?;

    let existing_versions = model::ForgeInstaller::all()
        .select(model::ForgeInstaller::fields().name())
        .exec(cx.conn)
        .context(info_span!("fetching existing Forge installer records"))
        .await?;
    let missing_versions = cataloged_versions
        .iter()
        .filter(|name| !existing_versions.contains(name))
        .collect::<Vec<_>>();
    info!(
        "catalog contains {} versions, of which {} are missing; downloading",
        cataloged_versions.len(),
        missing_versions.len()
    );

    let mut num_done = 0usize;
    for installer_name in missing_versions {
        let installer_url = format!(
            "https://maven.minecraftforge.net/net/minecraftforge/forge/{installer_name}/forge-{installer_name}-installer.jar"
        );

        async {
            let sha256 = cx
                .download_blob(installer_url)
                .context(info_span!("downloading installer", %installer_name))
                .await?;

            toasty::create!(model::ForgeInstaller {
                name: installer_name,
                download_run_id: cx.download_run_id,
                sha256,
                processed_at: None,
            })
            .exec(cx.conn)
            .context(info_span!("inserting installer record"))
            .await?;

            anyhow::Ok(())
        }
        .await
        .inspect_err(|err| cx.errors.push(err))
        .ok();

        num_done += 1;
        if num_done.is_multiple_of(10) {
            info!("downloaded {num_done} version installers");
        }
    }

    Ok(())
}

const BLACKLIST: &[&str] = &[
    // Not supported due to `data` field being `[]` even though the type is a map
    "1.12.2-14.23.5.2851",
    // Malformed Archives
    "1.6.1-8.9.0.749",
    "1.6.1-8.9.0.751",
    "1.6.4-9.11.1.960",
    "1.6.4-9.11.1.961",
    "1.6.4-9.11.1.963",
    "1.6.4-9.11.1.964",
];

pub async fn extract_installers(cx: &mut DownloadRunContext<'_>) -> Result<()> {
    let unprocessed_installers = model::ForgeInstaller::all()
        .filter(model::ForgeInstaller::fields().processed_at().is_none())
        .exec(cx.conn)
        .context(info_span!("fetching unprocessed forge installers"))
        .await?;

    for installer in unprocessed_installers {
        let result = async {
            let installer_blob = cx
                .blobs
                .get(installer.sha256)
                .context(info_span!("fetching installer blob"))
                .await?;

            anyhow::Ok(())
        }
        .await;
    }

    Ok(())
}
