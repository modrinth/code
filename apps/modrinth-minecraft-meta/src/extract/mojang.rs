use std::collections::HashMap;

use anyhow::{Context, Result, bail, ensure};
use futures::{StreamExt, TryStreamExt, stream};
use indexmap::IndexMap;
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;

use crate::{
    AppState, model,
    store::BlobCas,
    upstream::mojang,
    util::{Sha256, from_json_slice},
};

pub async fn extract(app: &AppState) -> Result<()> {
    let mut conn = app
        .db
        .connection()
        .context(info_span!("acquiring connection"))
        .await?;
    let mut rows = model::MojangCatalog::all()
        .exec(&mut conn)
        .context(info_span!("fetching Mojang catalogs"))
        .await?;
    info!("found {} downloaded catalogs", rows.len());

    // fetch runs, so that newer catalogs take precedence over older ones
    let runs = model::DownloadRun::all()
        .exec(&mut conn)
        .context(info_span!("fetching download runs"))
        .await?
        .into_iter()
        .map(|run| (run.id, run.started_at))
        .collect::<HashMap<_, _>>();
    rows.sort_by_key(|row| {
        (
            std::cmp::Reverse(runs.get(&row.download_run_id).copied()),
            row.id.0,
        )
    });

    // read each catalog from its blob in the store;
    // preserve query order while reading concurrently
    let catalogs = stream::iter(rows)
        .map(|row| async move {
            let bytes = app
                .cas
                .get(row.sha256)
                .context(
                    info_span!("fetching Mojang catalog", sha256 = %row.sha256),
                )
                .await?;
            let catalog = from_json_slice::<mojang::Catalog>(&bytes)
                .context("parsing Mojang catalog")?;
            anyhow::Ok((row.sha256, catalog))
        })
        .buffered(app.concurrency.download.get())
        .try_collect::<Vec<_>>()
        .await?;
    let (latest, versions) = select_versions(catalogs)?;
    info!("found {} unique versions", versions.len());

    // now we make all the version manifests;
    // finish reading and patching before changing any selected rows
    let num_total = versions.len();
    let mut extracted = Vec::with_capacity(num_total);
    for (source_catalog_sha256, catalog_entry) in versions {
        let (source_manifest_sha256, manifest) = make_version_manifest(&mut conn, &app.cas, &catalog_entry)
			.context(info_span!("extracting version manifest", version = %catalog_entry.id, %source_catalog_sha256))
			.await?;
        extracted.push((
            source_catalog_sha256,
            source_manifest_sha256,
            catalog_entry,
            manifest,
        ));
        let num_done = extracted.len();
        if num_done.is_multiple_of(100) || num_done == num_total {
            info!("extracted {num_done}/{num_total} version manifests");
        }
    }
    info!("inserting into database");

    // write the selected metadata together, so a failed extraction doesn't
    // leave a mix of old and new rows or conflicting latest-version flags
    let mut txn = conn
        .transaction()
        .context(info_span!("starting extraction transaction"))
        .await?;

    let previous_latest = model::MinecraftVersion::all()
        .filter(
            model::MinecraftVersion::fields()
                .is_latest_release()
                .is_some()
                .or(model::MinecraftVersion::fields()
                    .is_latest_snapshot()
                    .is_some()),
        )
        .exec(&mut txn)
        .context(info_span!("fetching previous latest versions"))
        .await?;
    for mut row in previous_latest {
        toasty::update!(row {
            is_latest_release: None::<bool>,
            is_latest_snapshot: None::<bool>,
        })
        .exec(&mut txn)
        .context(info_span!("unsetting previous latest versions"))
        .await?;
    }

    for (
        source_catalog_sha256,
        source_manifest_sha256,
        catalog_entry,
        manifest,
    ) in extracted
    {
        let name = catalog_entry.id.clone();
        model::MinecraftVersion::upsert_by_name(name.clone())
            .is_latest_release((name == latest.release).then_some(true))
            .is_latest_snapshot((name == latest.snapshot).then_some(true))
            .catalog_entry(toasty::Json(catalog_entry))
            .manifest(toasty::Json(manifest))
            .source_catalog_sha256(source_catalog_sha256)
            .source_manifest_sha256(source_manifest_sha256)
            .exec(&mut txn)
            .context(info_span!("upserting Minecraft version", %name))
            .await?;
    }
    txn.commit()
        .context(info_span!("committing extracted Minecraft metadata"))
        .await?;

    info!(
        num_versions = num_total,
        release = %latest.release,
        snapshot = %latest.snapshot,
        "stored Minecraft metadata"
    );
    Ok(())
}

/// Makes a list of versions from a list of catalogs (ordered newest-first), by
/// taking the latest version for each unique version name.
fn select_versions(
    catalogs: Vec<(Sha256, mojang::Catalog)>,
) -> Result<(mojang::Latest, Vec<(Sha256, mojang::Version)>)> {
    let (_, latest) =
        catalogs.first().context("no downloaded Mojang catalogs")?;
    let latest = latest.latest.clone();

    let mut versions = IndexMap::new();
    for (sha256, catalog) in catalogs {
        for version in catalog.versions {
            versions
                .entry(version.id.clone())
                .or_insert((sha256, version));
        }
    }

    ensure!(
        versions.contains_key(&latest.release),
        "catalog says its latest release is {}, which doesn't exist in version list",
        latest.release
    );
    ensure!(
        versions.contains_key(&latest.snapshot),
        "catalog says its latest snapshot is {}, which doesn't exist in version list",
        latest.snapshot
    );

    Ok((latest, versions.into_values().collect()))
}

/// Pulls a catalog entry's source manifest from [`BlobCas`] and patches its
/// libraries.
///
/// Returns:
/// - patched manifest hash
/// - patched manifest
async fn make_version_manifest(
    conn: &mut toasty::Connection,
    cas: &BlobCas,
    version: &mojang::Version,
) -> Result<(Sha256, mojang::VersionManifest)> {
    let matching_hashes = model::BlobHash::all()
        .select(model::BlobHash::fields().sha256())
        .filter(model::BlobHash::fields().sha1().eq(version.sha1))
        .exec(conn)
        .context(
            info_span!("resolving version manifest hash", sha1 = %version.sha1),
        )
        .await?;
    let sha256 = match matching_hashes.as_slice() {
        [sha256] => *sha256,
        [] => bail!(
            "no stored version manifest for {} with SHA-1 {}",
            version.id,
            version.sha1
        ),
        _ => bail!(
            "ambiguous version manifest for {}: multiple blobs have SHA-1 {}",
            version.id,
            version.sha1
        ),
    };
    let bytes = cas
        .get(sha256)
        .context(info_span!("fetching version blob", %sha256))
        .await?;

    let mut manifest = from_json_slice::<mojang::VersionManifest>(&bytes)
        .context("parsing version manifest")?;

    // post-process the version's libraries;
    // they need some patching to work with launchers
    // see `mojang::library`
    manifest.libraries = manifest
        .libraries
        .into_iter()
        .flat_map(mojang::patch_library)
        .collect();

    Ok((sha256, manifest))
}
