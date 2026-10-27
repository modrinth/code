use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result, bail, ensure};
use futures::{StreamExt, TryStreamExt, stream};
use indexmap::IndexMap;
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;

use crate::{
    AppState, model,
    store::BlobCas,
    upstream::mojang,
    util::{Sha1, Sha256, TaskContext, from_json_slice},
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
    let blobs =
        super::read_blobs(app, rows.iter().map(|row| row.sha256)).await?;
    let catalogs = rows
        .into_iter()
        .map(|row| {
            let catalog =
                from_json_slice::<mojang::Catalog>(&blobs[&row.sha256])
                    .context("parsing Mojang catalog")?;
            anyhow::Ok((row.sha256, catalog))
        })
        .collect::<Result<Vec<_>>>()?;
    drop(blobs);
    let candidate_sha1s = catalogs
        .iter()
        .flat_map(|(_, catalog)| {
            catalog.versions.iter().map(|version| version.sha1)
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let matching_hashes = model::BlobHash::all()
        .filter(model::BlobHash::fields().sha1().in_list(candidate_sha1s))
        .exec(&mut conn)
        .context(info_span!("fetching available version manifest hashes"))
        .await?;
    let mut hashes_by_sha1: HashMap<Sha1, Vec<Sha256>> = HashMap::new();
    for hash in matching_hashes {
        hashes_by_sha1
            .entry(hash.sha1)
            .or_default()
            .push(hash.sha256);
    }
    let available_sha1s = hashes_by_sha1.keys().copied().collect();
    let (latest, versions) = select_versions(catalogs, &available_sha1s)?;
    info!("found {} unique versions", versions.len());

    // now we make all the version manifests;
    // finish reading and patching before changing any selected rows
    let num_total = versions.len();
    let mut num_done = 0usize;
    let extracted = stream::iter(versions)
        .map(|(source_catalog_sha256, catalog_entry)| {
            let hashes_by_sha1 = &hashes_by_sha1;
            async move {
                let (source_manifest_sha256, manifest) = make_version_manifest(
                    hashes_by_sha1, &app.cas, &catalog_entry,
                )
			.transaction_context(|| info_span!("extracting version manifest", version = %catalog_entry.id, %source_catalog_sha256))
			.await?;
                anyhow::Ok((source_catalog_sha256, source_manifest_sha256, catalog_entry, manifest))
            }
        })
        .buffered(app.concurrency.download.get())
        .inspect_ok(|_| {
            num_done += 1;
        if num_done.is_multiple_of(100) || num_done == num_total {
            info!("extracted {num_done}/{num_total} version manifests");
        }
        })
        .try_collect::<Vec<_>>()
        .await?;
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
            .transaction_context(
                || info_span!("upserting Minecraft version", %name),
            )
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
/// taking the newest available revision for each unique version name.
/// Every version in the newest catalog must have a downloaded source blob.
fn select_versions(
    catalogs: Vec<(Sha256, mojang::Catalog)>,
    available_sha1s: &HashSet<Sha1>,
) -> Result<(mojang::Latest, Vec<(Sha256, mojang::Version)>)> {
    let (_, latest) =
        catalogs.first().context("no downloaded Mojang catalogs")?;
    let latest = latest.latest.clone();

    let mut versions = IndexMap::new();
    for (priority, (sha256, catalog)) in catalogs.into_iter().enumerate() {
        for version in catalog.versions {
            if !available_sha1s.contains(&version.sha1) {
                ensure!(
                    priority != 0,
                    "no stored version manifest for {} with SHA-1 {} in newest Mojang catalog",
                    version.id,
                    version.sha1
                );
                continue;
            }
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
    hashes_by_sha1: &HashMap<Sha1, Vec<Sha256>>,
    cas: &BlobCas,
    version: &mojang::Version,
) -> Result<(Sha256, mojang::VersionManifest)> {
    let matching_hashes = hashes_by_sha1
        .get(&version.sha1)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let sha256 = match matching_hashes {
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
        .flat_map(crate::import::mojang::patch_library)
        .collect();

    Ok((sha256, manifest))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(id: &str, bytes: &[u8]) -> mojang::Version {
        serde_json::from_value(serde_json::json!({
            "id": id,
            "type": "release",
            "url": "https://example.com/version.json",
            "time": "2026-01-01T00:00:00Z",
            "releaseTime": "2026-01-01T00:00:00Z",
            "sha1": Sha1::from_digest(bytes).to_string(),
            "complianceLevel": 1
        }))
        .unwrap()
    }

    fn catalog(versions: Vec<mojang::Version>) -> (Sha256, mojang::Catalog) {
        (
            Sha256::from_digest(b"catalog"),
            mojang::Catalog {
                latest: serde_json::from_value(serde_json::json!({
                    "release": "current", "snapshot": "current"
                }))
                .unwrap(),
                versions,
            },
        )
    }

    #[test]
    fn historical_selection_uses_newest_available_revision() {
        let current = version("current", b"current");
        let missing = version("historical", b"missing revision");
        let available = version("historical", b"available revision");
        let older = version("historical", b"older revision");
        let never_downloaded = version("absent", b"never downloaded");
        let hashes = HashSet::from([current.sha1, available.sha1, older.sha1]);
        let (latest, selected) = select_versions(
            vec![
                catalog(vec![current]),
                catalog(vec![missing, never_downloaded]),
                catalog(vec![available.clone()]),
                catalog(vec![older]),
            ],
            &hashes,
        )
        .unwrap();
        assert_eq!(latest.release, selected[0].1.id);
        assert_eq!(latest.snapshot, selected[0].1.id);
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[1].1.sha1, available.sha1);
    }

    #[test]
    fn newest_catalog_cannot_fall_back_to_downloaded_historical_revision() {
        let current = version("current", b"current");
        let missing = version("required", b"missing new revision");
        let old = version("required", b"downloaded old revision");
        let hashes = HashSet::from([current.sha1, old.sha1]);
        assert!(
            select_versions(
                vec![catalog(vec![current, missing]), catalog(vec![old]),],
                &hashes
            )
            .is_err()
        );
    }

    #[test]
    fn latest_versions_must_exist_in_available_selection() {
        assert!(
            select_versions(vec![catalog(Vec::new())], &HashSet::new())
                .is_err()
        );
        assert!(select_versions(Vec::new(), &HashSet::new()).is_err());
    }
}
