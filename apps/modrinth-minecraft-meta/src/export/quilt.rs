use std::collections::HashMap;

use anyhow::{Context, Result};
use indexmap::IndexMap;
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;
use url::Url;

use crate::{
    AppState,
    export::fabriclike::{
        self, CatalogGame, CatalogLoader, CatalogSnapshot, ProfileGroup,
    },
    model::{self, FabriclikeLoader},
    upstream::quilt,
    util::from_json_slice,
};

pub const FORMAT_VERSION: u32 = 1;

pub async fn export(app: &AppState) -> Result<()> {
    let mut conn = app
        .db
        .connection()
        .context(info_span!("acquiring connection"))
        .await?;
    let mut rows = model::QuiltCatalog::all()
        .exec(&mut conn)
        .context(info_span!("fetching Quilt catalogs"))
        .await?;
    info!(num_catalogs = rows.len(), "found downloaded Quilt catalogs");
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
	// keep the first copy of each game version, in latest-catalog order
    let mut games = IndexMap::new();
    let mut catalogs = Vec::with_capacity(rows.len());
    for row in rows {
        let bytes = app
            .cas
            .get(row.sha256)
            .context(info_span!("fetching Quilt catalog", sha256 = %row.sha256))
            .await?;
        let catalog = from_json_slice::<quilt::Catalog>(&bytes)
            .context("parsing Quilt catalog")?;
        for game in &catalog.game {
            games
                .entry(game.version.clone())
                .or_insert_with(|| game.clone());
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
                .filter(|loader| loader.version.0 != "0.17.5-beta.4")
                .map(|loader| CatalogLoader {
                    id: loader.version.to_string(),
                    stable: loader.stable,
                })
                .collect(),
            mappings: Vec::new(),
        });
    }
	// older loaders may only have been downloaded with an older template;
	// keep those templates as fallbacks, within the same version group
    let mut historical_templates: IndexMap<String, IndexMap<String, ()>> =
        IndexMap::new();
    for catalog in &catalogs {
        let snapshot_games = catalog
            .games
            .iter()
            .map(|game| quilt::GameVersion {
                version: quilt::GameVersionName(game.id.clone()),
                stable: game.stable,
            })
            .collect::<Vec<_>>();
        for group in quilt::profile_groups(&snapshot_games) {
            historical_templates
                .entry(group.id)
                .or_default()
                .entry(group.template.to_string())
                .or_insert(());
        }
    }
    let games = games.into_values().collect::<Vec<_>>();
    let profile_base_url = Url::parse(quilt::PROFILE_BASE_URL)?;
	// use the downloader's grouping rules;
	// legacy and modern games need separate profiles for the same loader
    let groups = quilt::profile_groups(&games)
        .into_iter()
        .map(|group| ProfileGroup {
            fallback_templates: historical_templates
                .shift_remove(&group.id)
                .unwrap_or_default()
                .into_keys()
                .filter(|template| template != &group.template.0)
                .collect(),
            id: Some(group.id),
            template: group.template.to_string(),
            games: group
                .games
                .into_iter()
                .map(|game| game.to_string())
                .collect(),
            profile_base_url: profile_base_url.clone(),
        })
        .collect();
    fabriclike::export_catalogs(
        app,
        FabriclikeLoader::Quilt,
        FORMAT_VERSION,
        &Url::parse(quilt::MAVEN_URL)?,
        catalogs,
        groups,
    )
    .context(info_span!(
        "exporting catalogs",
        loader = ?FabriclikeLoader::Quilt
    ))
    .await
}
