use std::{
    collections::HashMap,
    sync::atomic::{self, AtomicUsize},
};

use anyhow::{Context, Result, bail, ensure};
use derive_more::Display;
use futures::{StreamExt, stream};
use serde::{Deserialize, Serialize};
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;

use crate::{
    model::{self, ForgelikeInstallerName},
    task::DownloadRunContext,
};

pub const CATALOG_URL: &str = "https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json";

/// Forge loader manifest for all game versions, mapped to all loader versions
/// available for that game version.
///
/// Available at [`CATALOG_URL`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog(pub HashMap<GameVersionName, Vec<ForgelikeInstallerName>>);

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

    let mut cataloged_versions = Vec::new();
    let mut skipped_pre_installer_versions = 0usize;
    for name in catalog.0.into_values().flatten() {
        if !has_supported_installer(&name)
            .with_context(|| format!("checking installer support for {name}"))?
        {
            skipped_pre_installer_versions += 1;
            continue;
        }
        if !BLACKLIST.contains(&name.0.as_str()) {
            cataloged_versions.push(name);
        }
    }

    cataloged_versions.sort();
    info!(
        raw_cataloged_versions,
        skipped_pre_installer_versions,
        num_cataloged_versions = cataloged_versions.len(),
        "downloaded Forge catalog"
    );

    toasty::create!(model::ForgeCatalog {
        download_run_id: cx.download_run_id,
        sha256,
    })
    .exec(*cx.conn().await)
    .context(info_span!("inserting catalog"))
    .await?;

    let existing_versions = model::ForgelikeInstaller::all()
        .filter(
            model::ForgelikeInstaller::fields()
                .loader()
                .eq(model::ForgelikeLoader::Forge),
        )
        .select(model::ForgelikeInstaller::fields().name())
        .exec(*cx.conn().await)
        .context(info_span!("fetching existing Forge installer records"))
        .await?;
    let missing_versions = cataloged_versions
        .iter()
        .filter(|name| !existing_versions.contains(name))
        .collect::<Vec<_>>();
    info!(
        "catalog contains {} versions, of which {} are missing; downloading installers",
        cataloged_versions.len(),
        missing_versions.len()
    );

    let num_total = missing_versions.len();
    let num_done = AtomicUsize::new(0);
    let task = async |installer_name: &ForgelikeInstallerName| {
        let installer_url = format!(
            "https://maven.minecraftforge.net/net/minecraftforge/forge/{installer_name}/forge-{installer_name}-installer.jar"
        );

        async {
            let sha256 = cx
                .download_blob(installer_url)
                .context(info_span!("downloading installer", %installer_name))
                .await?;

            toasty::create!(model::ForgelikeInstaller {
                loader: model::ForgelikeLoader::Forge,
                name: installer_name,
                download_run_id: cx.download_run_id,
                sha256,
            })
            .exec(*cx.conn().await)
            .context(info_span!("inserting installer record"))
            .await?;

            anyhow::Ok(())
        }
        .await
        .inspect_err(|err| cx.errors.push(err))
        .ok();

        let num_done = num_done.fetch_add(1, atomic::Ordering::SeqCst) + 1;
        if num_done.is_multiple_of(10) || num_done == num_total {
            info!("downloaded {num_done}/{num_total} version installers");
        }
    };
    stream::iter(missing_versions)
        .for_each_concurrent(cx.download_concurrency, task)
        .await;

    Ok(())
}

/// If this installer version has an installer JAR which we can process.
///
/// Very old Forge versions don't have an installer JAR, so we don't support
/// them.
fn has_supported_installer(name: &ForgelikeInstallerName) -> Result<bool> {
    let (minecraft, remaining) = name
        .0
        .split_once('-')
        .context("missing Forge version separator")?;
    ensure!(!minecraft.is_empty(), "missing Minecraft version");
    let version = remaining
        .split('-')
        .next()
        .context("missing Forge version")?;
    let components = version
        .split('.')
        .map(|part| {
            part.parse::<u32>()
                .context("invalid numeric Forge version component")
        })
        .collect::<Result<Vec<_>>>()?;

    let version_parts = match (
        <[u32; 4]>::try_from(components.as_slice()),
        <[u32; 3]>::try_from(components.as_slice()),
    ) {
        (Ok([a, b, c, d]), _) => [a, b, c, d],
        (Err(_), Ok([a, b, c])) => [a, b, c, 0],
        (Err(_), Err(_)) => bail!("expected 3 or 4 Forge version components"),
    };
    Ok(version_parts >= [7, 8, 0, 684])
}

/// Specific Forge installer versions which we can't download or process.
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

#[cfg(test)]
mod tests {
    use super::{ForgelikeInstallerName, has_supported_installer};

    #[test]
    fn installer_support_uses_numeric_release_boundary() {
        for (name, supported) in [
            ("1.1-1.3.3.26", false),
            ("1.5.2-7.8.0.683", false),
            ("1.5.2-7.8.0.684", true),
            ("1.5.2-7.8.1.1", true),
            ("1.7.10-10.13.4.1614-1.7.10", true),
            ("1.10.2-12.18.1.2016-failtests", true),
            ("1.21.8-58.1.22", true),
        ] {
            assert_eq!(
                has_supported_installer(&ForgelikeInstallerName(
                    name.to_owned()
                ))
                .unwrap(),
                supported,
                "{name}"
            );
        }
    }
}
