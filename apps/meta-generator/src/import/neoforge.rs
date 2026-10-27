use std::{
    collections::HashSet,
    sync::atomic::{self, AtomicUsize},
};

use anyhow::{Result, ensure};
use futures::{StreamExt, stream};
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;

use crate::util::TaskContext;

use crate::{
    import::download::DownloadRunContext,
    model::{self, ForgelikeInstallerName},
    upstream::neoforge::{Catalog, FORGE_CATALOG_URL, NEOFORGE_CATALOG_URL},
};

pub async fn download(cx: &mut DownloadRunContext<'_>) -> Result<()> {
    let initial_errors = cx.errors.len();
    let (forge_catalog, forge_sha256) = cx
        .download_json::<Catalog>(FORGE_CATALOG_URL)
        .context(info_span!("fetching Forge catalog"))
        .await?;
    let (neoforge_catalog, neoforge_sha256) = cx
        .download_json::<Catalog>(NEOFORGE_CATALOG_URL)
        .context(info_span!("fetching NeoForge catalog"))
        .await?;
    info!(
        num_forge_versions = forge_catalog.versions.len(),
        num_neoforge_versions = neoforge_catalog.versions.len(),
        "downloaded NeoForge catalogs"
    );

    let mut seen = HashSet::new();
    let cataloged_versions = forge_catalog
        .versions
        .into_iter()
        .map(|version| version.0.parse().map(|name| (name, "forge")))
        .chain(
            neoforge_catalog.versions.into_iter().map(|version| {
                version.0.parse().map(|name| (name, "neoforge"))
            }),
        )
        .collect::<Result<Vec<(ForgelikeInstallerName, &str)>>>()?
        .into_iter()
        .filter(|(name, _)| !model::NEOFORGE_BLACKLIST.contains(&name.as_str()))
        .filter(|(name, _)| seen.insert(name.clone()))
        .collect::<Vec<_>>();
    let existing_versions = model::ForgelikeInstaller::all()
        .filter(
            model::ForgelikeInstaller::fields()
                .loader()
                .eq(model::ForgelikeLoader::Neoforge),
        )
        .select(model::ForgelikeInstaller::fields().name())
        .exec(*cx.conn().await)
        .context(info_span!("fetching existing NeoForge installer records"))
        .await?;
    let missing_versions = cataloged_versions
        .iter()
        .filter(|(name, _)| !existing_versions.contains(name))
        .collect::<Vec<_>>();
    info!(
        "catalogs contain {} distinct versions, of which {} are missing; downloading installers",
        cataloged_versions.len(),
        missing_versions.len()
    );

    let num_total = missing_versions.len();
    let num_done = AtomicUsize::new(0);
    let task = async |(installer_name, artifact): &(
        ForgelikeInstallerName,
        &str,
    )| {
        let url = installer_url(artifact, installer_name);
        async {
            let sha256 = cx
                .download_blob(url)
                .context(info_span!("downloading installer", %installer_name, artifact))
                .await?;
            toasty::create!(model::ForgelikeInstaller {
                loader: model::ForgelikeLoader::Neoforge,
                name: installer_name,
                download_run_id: cx.download_run_id,
                sha256,
            })
            .exec(*cx.conn().await)
            .context(info_span!("inserting installer record"))
            .await?;
            anyhow::Ok(())
		}
		.transaction_context(|| info_span!("downloading NeoForge installer", %installer_name, artifact))
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

    ensure!(
        cx.errors.len() == initial_errors,
        "NeoForge item downloads failed; catalogs not registered"
    );
    toasty::create!(model::NeoforgeCatalog {
        download_run_id: cx.download_run_id,
        forge_sha256,
        neoforge_sha256,
    })
    .exec(*cx.conn().await)
    .context(info_span!("inserting catalog"))
    .await?;

    Ok(())
}

fn installer_url(artifact: &str, name: &ForgelikeInstallerName) -> String {
    format!(
        "https://maven.neoforged.net/releases/net/neoforged/{artifact}/{name}/{artifact}-{name}-installer.jar"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installer_urls_preserve_catalog_namespace_and_version() {
        for (artifact, version, expected) in [
            (
                "forge",
                "1.20.1-47.1.74",
                "https://maven.neoforged.net/releases/net/neoforged/forge/1.20.1-47.1.74/forge-1.20.1-47.1.74-installer.jar",
            ),
            (
                "forge",
                "47.1.82",
                "https://maven.neoforged.net/releases/net/neoforged/forge/47.1.82/forge-47.1.82-installer.jar",
            ),
            (
                "neoforge",
                "20.2.29-beta",
                "https://maven.neoforged.net/releases/net/neoforged/neoforge/20.2.29-beta/neoforge-20.2.29-beta-installer.jar",
            ),
        ] {
            assert_eq!(
                installer_url(
                    artifact,
                    &version.parse::<ForgelikeInstallerName>().unwrap()
                ),
                expected
            );
        }
    }
}
