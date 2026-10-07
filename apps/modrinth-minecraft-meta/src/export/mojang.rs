use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result, bail, ensure};
use futures::{StreamExt, stream::FuturesUnordered};
use tracing::{Instrument, debug, info, info_span, warn};
use tracing_anyhow::FutureContext;

use crate::{
    AppState, model,
    store::BlobCas,
    upstream::mojang,
    util::{Sha1, from_json_slice},
};

pub const FORMAT_VERSION: u32 = 0;

pub async fn export(app: &AppState) -> Result<()> {
    let mut conn = app
        .db
        .connection()
        .context(info_span!("acquiring connection"))
        .await?;

    let catalog_rows = model::MojangCatalog::all()
        .exec(&mut conn)
        .context(info_span!("fetching Mojang catalogs"))
        .await?;
    info!("found {} downloaded catalogs", catalog_rows.len());

    // fetch runs, so we can prioritze the latest catalog later
    let runs = model::DownloadRun::all()
        .exec(&mut conn)
        .context(info_span!("fetching download runs"))
        .await?
        .into_iter()
        .map(|run| (run.id, run.started_at))
        .collect::<HashMap<_, _>>();

    // read each catalog from its blob in the store
    let mut catalogs = catalog_rows
        .into_iter()
        .map(|catalog_row| {
            let sha256 = catalog_row.sha256;
            let span = info_span!("fetching catalog", %sha256);
            async move {
                let blob = app
                    .cas
                    .get(sha256)
                    .context(info_span!("fetching blob"))
                    .await?;
                let catalog_data = from_json_slice::<mojang::Catalog>(&blob)
                    .context("parsing as catalog")?;
                anyhow::Ok((catalog_row, catalog_data))
            }
            .instrument(span)
        })
        .map(|fut| async {
            fut.await.inspect_err(|err| warn!("error: {err:?}")).ok()
        })
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<Option<_>>>()
        .await
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    info!(
        "{} of those catalogs are valid Mojang version catalogs",
        catalogs.len()
    );

    catalogs.sort_by_key(|(row, _)| {
        (
            std::cmp::Reverse(runs.get(&row.download_run_id).copied()),
            row.id.0,
        )
    });
    let latest = catalogs
        .first()
        .context("no valid Mojang catalogs")?
        .1
        .latest
        .clone();
    let mut seen = HashSet::new();

    // now we make all the version manifests
    let mut processed_versions = Vec::new();
    for (catalog_row, catalog_data) in catalogs {
        for version in catalog_data.versions {
            if !seen.insert(version.id.clone()) {
                continue;
            }
            let span = info_span!(
                "making version manifest",
                catalog_sha256 = %catalog_row.sha256,
                %version.id,
            );
            match make_version_manifest(&mut conn, &app.cas, version)
                .context(span)
                .await
            {
                Ok((mut version, version_manifest, version_manifest_str)) => {
                    let mut url = app.config.public_base_url.clone();
                    url.set_query(None);
                    url.set_fragment(None);
                    url.path_segments_mut()
                        .map_err(|()| {
                            anyhow::anyhow!(
                                "public_base_url must support path segments"
                            )
                        })?
                        .pop_if_empty()
                        .extend([
                            "minecraft",
                            &format!("v{FORMAT_VERSION}"),
                            "versions",
                            &format!("{}.json", version.id),
                        ]);
                    version.url = url;
                    processed_versions.push((
                        version,
                        (version_manifest, version_manifest_str),
                    ));
                }
                Err(err) => warn!("error: {err:?}"),
            }
        }
    }

    // dedup and sort by latest-first
    processed_versions.sort_by_key(|(version, _)| version.id.clone());
    processed_versions.dedup_by_key(|(version, _)| version.id.clone());
    processed_versions
        .sort_by_key(|(version, _)| std::cmp::Reverse(version.release_time));
    info!("found {} unique versions", processed_versions.len());

    // make the uber catalog
    let (uber_versions, version_manifests): (Vec<_>, Vec<_>) =
        processed_versions.into_iter().unzip();

    for id in [&latest.release, &latest.snapshot] {
        ensure!(
            uber_versions.iter().any(|version| &version.id == id),
            "latest version {id} was not exported"
        );
    }
    info!(release = %latest.release, snapshot = %latest.snapshot, "found latest versions");

    // write processed version manifests
    {
        let num_total = version_manifests.len();
        let mut num_done = 0usize;
        for (version_manifest, data) in version_manifests {
            let game_version = version_manifest.id.clone();
            let path = format!(
                "minecraft/v{FORMAT_VERSION}/versions/{game_version}.json"
            );
            app.public_blobs
                .put(&path, data.as_bytes())
                .context(info_span!("writing manifest to blob store", %path))
                .await?;
            debug!(%path, "wrote version manifest to blob store");

            num_done += 1;
            if num_done.is_multiple_of(100) || num_done == num_total {
                info!("wrote {num_done}/{num_total} version manifests");
            }
        }
    }

    // write manifest (uber catalog)
    {
        let uber_catalog = mojang::Catalog {
            latest,
            versions: uber_versions,
        };
        let data = serde_json::to_string(&uber_catalog)
            .expect("serialization should never fail");
        let path = format!("minecraft/v{FORMAT_VERSION}/manifest.json");
        app.public_blobs
            .put(&path, data.as_bytes())
            .context(info_span!("writing manifest to blob store", %path))
            .await?;
        info!(%path, "wrote uber catalog (manifest) to blob store");
    }

    Ok(())
}

/// Takes a [`mojang::Version`] reference in a [`mojang::Catalog`], pulls the
/// version manifest blob, post-processes it, and makes a full
/// [`mojang::VersionManifest`] which we can export.
///
/// Returns:
/// - a post-processed version of the `version` you pass in
/// - the `VersionManifest` we fetch from the blob, with post-processing
/// - the version manifest, encoded as JSON
async fn make_version_manifest(
    conn: &mut toasty::Connection,
    cas: &BlobCas,
    mut version: mojang::Version,
) -> Result<(mojang::Version, mojang::VersionManifest, String)> {
    let matching_hashes = model::BlobHash::all()
        .select(model::BlobHash::fields().sha256())
        .filter(model::BlobHash::fields().sha1().eq(version.sha1))
        .exec(conn)
        .context(
            info_span!("resolving version manifest hash", sha1 = %version.sha1),
        )
        .await?;
    let version_blob_sha256 = match matching_hashes.as_slice() {
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

    let version_blob = cas
        .get(version_blob_sha256)
        .context(info_span!("fetching version blob", %version_blob_sha256))
        .await?;
    let mut version_manifest =
        from_json_slice::<mojang::VersionManifest>(&version_blob)
            .context("parsing as version")?;

    // post-process the version's libraries;
    // they need some patching to work with launchers
    // see `mojang::library`
    version_manifest.libraries = version_manifest
        .libraries
        .into_iter()
        .flat_map(mojang::patch_library)
        .collect();

    // adjust the version ref that goes into the uber catalog
    let version_manifest_str = serde_json::to_string(&version_manifest)
        .expect("serialization should not fail");
    let version_manifest_sha1 =
        Sha1::from_digest(version_manifest_str.as_bytes());
    version.sha1 = version_manifest_sha1;

    // legacy Daedalus also changed:
    // - `complianceLevel`: always set to `1`
    // - `time`: set to `version_manifest.time`
    //
    // I haven't kept these in since app-lib doesn't use them,
    // and I have no clue why these got changed.

    Ok((version, version_manifest, version_manifest_str))
}
