use std::collections::HashMap;

use anyhow::{Context, Result};
use indexmap::IndexSet;
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;
use url::Url;

use crate::{
    AppState,
    import::extract::fabriclike::{
        self, Artifact, CatalogGame, CatalogLoader, CatalogSnapshot,
        ProfileGroup,
    },
    model::{self, FabriclikeLoader},
    upstream::fabric,
    util::from_json_slice,
};

pub async fn extract(app: &AppState) -> Result<()> {
    let mut conn = app
        .db
        .connection()
        .context(info_span!("acquiring connection"))
        .await?;
    let mut rows = model::FabricCatalog::all()
        .exec(&mut conn)
        .context(info_span!("fetching Fabric catalogs"))
        .await?;
    info!("found {} downloaded catalogs", rows.len());

    // fetch download runs where we downloaded these catalogs from;
    // newer catalogs take precedence over older ones
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

    // read each catalog from its blob in the store,
    // and start assembling the artifacts we'll extract
    let repository = Url::parse(crate::import::fabric::MAVEN_URL)?;
    let mut catalogs = Vec::new();
    let mut games = IndexSet::new();
    let blobs =
        super::read_blobs(app, rows.iter().map(|row| row.sha256)).await?;
    for row in rows {
        let bytes = &blobs[&row.sha256];
        let catalog = from_json_slice::<fabric::Catalog>(bytes)
            .context("parsing Fabric catalog")?;

        for game in &catalog.game {
            games.insert(game.version.to_string());
        }

        catalogs.push(CatalogSnapshot {
            download_run_id: row.download_run_id,
            sha256: row.sha256,
            games: catalog
                .game
                .into_iter()
                .map(|game| CatalogGame {
                    id: game.version.to_string(),
                    stable: game.stable,
                })
                .collect(),
            loaders: catalog
                .loader
                .into_iter()
                .map(|loader| CatalogLoader {
                    id: loader.version.to_string(),
                    stable: loader.stable,
                })
                .collect(),
            mappings:
                catalog
                    .intermediary
                    .into_iter()
                    .map(|mapping| Artifact {
                        coordinate: mapping.maven,
                        repository: repository.clone(),
                        source: Some(format!(
                            "{} (catalog SHA256 {})",
                            fabric::CATALOG_URL,
                            row.sha256
                        )),
                        repository_source:
                            "catalog intermediary: loader Maven fallback"
                                .to_owned(),
                    })
                    .collect(),
        });
    }

    drop(blobs);

    // setup version groups
    // see README.md on what version groups are and why we need them
    // Fabric only has one, since it takes a single version (`PROFILE_GAME_VERSION`)
    // and templates that into all the json files. Quilt has more.
    let group = ProfileGroup {
        id: None,
        template: crate::import::fabric::PROFILE_GAME_VERSION.to_owned(),
        fallback_templates: Vec::new(),
        games: games.into_iter().collect(),
        profile_base_url: Url::parse(&format!(
            "{}/loader/",
            fabric::CATALOG_URL
        ))?,
    };

    fabriclike::extract_catalogs(
        app,
        FabriclikeLoader::Fabric,
        &repository,
        catalogs,
        vec![group],
    )
    .context(info_span!(
        "extracting catalogs",
        loader = ?FabriclikeLoader::Fabric
    ))
    .await
}
