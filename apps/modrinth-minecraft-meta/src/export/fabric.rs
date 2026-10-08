use std::collections::HashMap;

use anyhow::{Context, Result};
use indexmap::IndexMap;
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;
use url::Url;

use crate::{
    AppState,
    export::fabriclike::{
        self, Artifact, CatalogGame, CatalogLoader, CatalogSnapshot,
        ProfileGroup,
    },
    model,
    upstream::fabric,
    util::from_json_slice,
};

pub const FORMAT_VERSION: u32 = 0;

pub async fn export(app: &AppState) -> Result<()> {
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
    let repository = Url::parse(fabric::MAVEN_URL)?;
    let mut catalogs = Vec::new();
    let mut games = IndexMap::new();
    for row in rows {
        let bytes = app
            .cas
            .get(row.sha256)
            .context(
                info_span!("fetching Fabric catalog", sha256 = %row.sha256),
            )
            .await?;
        let catalog = from_json_slice::<fabric::Catalog>(&bytes)
            .context("parsing Fabric catalog")?;
        for game in &catalog.game {
            games.entry(game.version.to_string()).or_insert(());
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
			mappings: catalog
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
					repository_source: "catalog intermediary: loader Maven fallback",
				})
				.collect(),
		});
    }
    let group = ProfileGroup {
        id: None,
        template: fabric::PROFILE_GAME_VERSION.to_owned(),
        fallback_templates: Vec::new(),
        games: games.into_keys().collect(),
        profile_base_url: Url::parse(&format!(
            "{}/loader/",
            fabric::CATALOG_URL
        ))?,
    };
    fabriclike::export_catalogs(
        app,
        "fabric",
        FORMAT_VERSION,
        &repository,
        catalogs,
        vec![group],
    )
    .await
}
