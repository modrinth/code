use anyhow::{Context, Result, bail};
use futures::{StreamExt, stream::FuturesUnordered};
use tracing::{Instrument, debug, info, info_span, warn};
use tracing_anyhow::FutureContext;

use crate::{
    AppState, model, store::BlobCas, upstream::mojang, util::from_json_slice,
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

    // read each catalog from its blob in the store
    let catalogs = catalog_rows
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

    // now we start:
    // - making the uber catalog
    // - making all version manifests
    let mut uber_versions = Vec::<mojang::Version>::new();
    let mut version_manifests = Vec::<mojang::VersionManifest>::new();
    for (catalog_row, catalog_data) in catalogs {
        for version in catalog_data.versions {
            let span = info_span!(
                "making version manifest",
                catalog_sha256 = %catalog_row.sha256,
                %version.id,
            );
            match make_version_manifest(&mut conn, &app.cas, &version)
                .context(span)
                .await
            {
                Ok(manifest) => {
                    uber_versions.push(version);
                    version_manifests.push(manifest);
                }
                Err(err) => warn!("error: {err:?}"),
            }
        }
    }

    // dedup and sort by latest-first
    uber_versions.sort_by_key(|version| version.id.clone());
    uber_versions.dedup_by_key(|version| version.id.clone());
    uber_versions
        .sort_by_key(|version| std::cmp::Reverse(version.release_time));

    // find latest release/snapshot game versions
    let latest_release = uber_versions
        .iter()
        .find(|version| version.ty == mojang::VersionType::Release)
        .map(|version| version.id.clone())
        .context("no latest release")?;
    let latest_snapshot = uber_versions
        .iter()
        .find(|version| version.ty == mojang::VersionType::Snapshot)
        .map(|version| version.id.clone())
        .context("no latest snapshot")?;
    info!(
        %latest_release,
        %latest_snapshot,
        "found {} unique versions",
        uber_versions.len()
    );

    // write manifest (uber catalog)
    {
        let uber_catalog = mojang::Catalog {
            latest: mojang::Latest {
                release: latest_release,
                snapshot: latest_snapshot,
            },
            versions: uber_versions,
        };
        let manifest = serde_json::to_string(&uber_catalog)
            .expect("serialization should never fail");
        let path = format!("minecraft/v{FORMAT_VERSION}/manifest.json");
        app.public_blobs
            .put(&path, manifest.as_bytes())
            .context(info_span!("writing manifest to blob store", %path))
            .await?;
        info!(%path, "wrote uber catalog (manifest) to blob store");
    }

    // write processed version manifests
    {
        let num_total = version_manifests.len();
        let mut num_done = 0usize;
        for version_manifest in version_manifests {
            let game_version = version_manifest.id.clone();
            let version_manifest = serde_json::to_string(&version_manifest)
                .expect("serialization should never fail");
            let path = format!(
                "minecraft/v{FORMAT_VERSION}/versions/{game_version}.json"
            );
            app.public_blobs
                .put(&path, version_manifest.as_bytes())
                .context(info_span!("writing manifest to blob store", %path))
                .await?;
            debug!(%path, "wrote version manifest to blob store");

            num_done += 1;
            if num_done.is_multiple_of(100) || num_done == num_total {
                info!("wrote {num_done}/{num_total} version manifests");
            }
        }
    }

    Ok(())
}

/// Takes a [`mojang::Version`] reference in a [`mojang::Catalog`], pulls the
/// version manifest blob, post-processes it, and makes a full
/// [`mojang::VersionManifest`] which we can export.
async fn make_version_manifest(
    conn: &mut toasty::Connection,
    cas: &BlobCas,
    version: &mojang::Version,
) -> Result<mojang::VersionManifest> {
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

    Ok(version_manifest)
}
