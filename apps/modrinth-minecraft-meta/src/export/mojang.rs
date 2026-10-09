use anyhow::{Result, bail, ensure};
use tracing::{debug, info, info_span};
use tracing_anyhow::FutureContext;

use crate::{
    AppState, model,
    store::ContentType,
    upstream::mojang,
    util::{Sha1, to_json_string},
};

pub const FORMAT_VERSION: u32 = 0;

struct VersionManifestExport {
    version: mojang::Version,
    json: String,
    path: String,
}

pub async fn export(app: &AppState) -> Result<()> {
    let mut conn = app
        .db
        .connection()
        .context(info_span!("acquiring connection"))
        .await?;

    let rows = model::MinecraftVersion::all()
        .exec(&mut conn)
        .context(info_span!("fetching extracted Minecraft versions"))
        .await?;
    ensure!(
        !rows.is_empty(),
        "no extracted Minecraft versions; run Mojang extraction before exporting"
    );
    info!("found {} extracted Minecraft versions", rows.len());

    let latest_release = {
        let mut all_latest = rows
            .iter()
            .filter(|row| row.is_latest_release == Some(true))
            .collect::<Vec<_>>()
            .into_iter();
        let (Some(latest), None) = (all_latest.next(), all_latest.next())
        else {
            bail!(
                "expected exactly one extracted latest release; \
                run Mojang extraction before exporting",
            );
        };
        latest
    };

    let latest_snapshot = {
        let mut all_latest = rows
            .iter()
            .filter(|row| row.is_latest_snapshot == Some(true))
            .collect::<Vec<_>>()
            .into_iter();
        let (Some(latest), None) = (all_latest.next(), all_latest.next())
        else {
            bail!(
                "expected exactly one extracted latest snapshot; \
                run Mojang extraction before exporting",
            );
        };
        latest
    };

    let latest = mojang::Latest {
        release: latest_release.name.clone(),
        snapshot: latest_snapshot.name.clone(),
    };
    info!(release = %latest.release, snapshot = %latest.snapshot, "found latest versions");

    // prepare all version manifests before writing
    let mut exports = rows
        .into_iter()
        .map(|mut row| {
            let json = to_json_string(&row.manifest);
            row.catalog_entry.0.sha1 = Sha1::from_digest(json.as_bytes());
            let path = format!(
                "minecraft/v{FORMAT_VERSION}/versions/{}.json",
                row.name
            );
            row.catalog_entry.0.url = app.public_blobs.url_for(&path);

            VersionManifestExport {
                version: row.catalog_entry.0,
                json,
                path,
            }
        })
        .collect::<Vec<_>>();

    // sort by latest-first, breaking ties by version ID
    exports.sort_by_key(|export| {
        (
            std::cmp::Reverse(export.version.release_time),
            export.version.id.clone(),
        )
    });

    let uber_catalog = mojang::Catalog {
        latest,
        versions: exports
            .iter()
            .map(|export| export.version.clone())
            .collect(),
    };
    let data = to_json_string(&uber_catalog);

    // write processed version manifests
    let num_total = exports.len();
    for (index, export) in exports.iter().enumerate() {
        let game_version = &export.version.id;
        let path = &export.path;
        app.public_blobs
			.put(path, export.json.as_bytes(), ContentType::Json)
			.context(
				info_span!("writing manifest to blob store", %game_version, %path),
			)
			.await?;
        debug!(%path, "wrote version manifest to blob store");

        let num_done = index + 1;
        if num_done.is_multiple_of(100) || num_done == num_total {
            info!("wrote {num_done}/{num_total} version manifests");
        }
    }

    // write manifest (uber catalog)
    let path = format!("minecraft/v{FORMAT_VERSION}/manifest.json");
    app.public_blobs
        .put(&path, data.as_bytes(), ContentType::Json)
        .context(info_span!("writing manifest to blob store", %path))
        .await?;
    info!(%path, "wrote uber catalog (manifest) to blob store");

    Ok(())
}
