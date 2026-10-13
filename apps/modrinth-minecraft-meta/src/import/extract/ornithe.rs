use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use futures::{StreamExt, TryStreamExt, stream};
use indexmap::IndexMap;
use tracing::{debug, info, info_span};
use tracing_anyhow::FutureContext;
use url::Url;

use crate::{
    AppState,
    import::{
        extract::fabriclike::{
            self, CatalogGame, CatalogLoader, CatalogSnapshot, ProfileGroup,
        },
        ornithe::{
            GameLibraries, libraries_url, mojang_game_version, profile_groups,
            version_manifest_url,
        },
    },
    model::{self, FabriclikeLoader},
    upstream::ornithe,
    util::{Sha256, from_json_slice},
};

pub async fn extract(app: &AppState) -> Result<()> {
    let mut conn = app
        .db
        .connection()
        .context(info_span!("acquiring connection"))
        .await?;
    let mut rows = model::OrnitheCatalog::all()
        .exec(&mut conn)
        .context(info_span!("fetching Ornithe catalogs"))
        .await?;
    info!(
        num_catalogs = rows.len(),
        "found downloaded Ornithe catalogs"
    );

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

    // the app only launches game versions from Mojang's catalog;
    // unpatched manifests tell which libraries Ornithe removes
    let mojang_manifests = model::MinecraftVersion::all()
        .exec(&mut conn)
        .context(info_span!("fetching extracted Minecraft versions"))
        .await?
        .into_iter()
        .map(|version| (version.name.0, version.source_manifest_sha256))
        .collect::<HashMap<_, _>>();

    let mut games = IndexMap::new();
    let mut snapshots = Vec::with_capacity(rows.len());
    for row in &rows {
        let catalog_games = from_json_slice::<Vec<ornithe::GameVersion>>(
            &app.cas
                .get(row.games_sha256)
                .context(info_span!("fetching Ornithe game versions", sha256 = %row.games_sha256))
                .await?,
        )
        .context("parsing Ornithe game versions")?;
        let catalog_loaders = from_json_slice::<Vec<ornithe::LoaderVersion>>(
            &app.cas
                .get(row.loaders_sha256)
                .context(info_span!("fetching Ornithe loader versions", sha256 = %row.loaders_sha256))
                .await?,
        )
        .context("parsing Ornithe loader versions")?;

        for game in &catalog_games {
            games
                .entry(game.version.clone())
                .or_insert_with(|| game.clone());
        }
        snapshots.push((
            row.download_run_id,
            row.games_sha256,
            catalog_games,
            catalog_loaders,
        ));
    }

    let run_priority = rows
        .iter()
        .enumerate()
        .map(|(priority, row)| (row.download_run_id, priority))
        .collect::<HashMap<_, _>>();
    let urls = games
        .keys()
        .flat_map(|game| {
            [libraries_url(game), version_manifest_url(game)]
                .map(|url| url.to_string())
        })
        .collect::<Vec<_>>();
    let mut downloads = model::BlobDownload::all()
        .filter(model::BlobDownload::fields().url().in_list(urls))
        .exec(&mut conn)
        .context(info_span!("fetching downloaded game libraries"))
        .await?;
    drop(conn);
    downloads.sort_by_key(|row| {
        (
            run_priority
                .get(&row.download_run_id)
                .copied()
                .unwrap_or(usize::MAX),
            row.download_run_id.0,
        )
    });
    let mut hashes = HashMap::<String, Sha256>::new();
    for download in downloads {
        hashes.entry(download.url).or_insert(download.sha256);
    }

    // games Ornithe has no version manifest for are never downloaded
    let available = games
        .keys()
        .filter_map(|game| {
            let libraries = *hashes.get(libraries_url(game).as_str())?;
            let manifest = *hashes.get(version_manifest_url(game).as_str())?;
            Some((game.clone(), libraries, manifest))
        })
        .collect::<Vec<_>>();
    info!(
        num_games = available.len(),
        "reading downloaded game libraries"
    );
    let mojang_manifests = &mojang_manifests;
    let game_libraries = stream::iter(available)
        .map(|(game, libraries, manifest)| async move {
            let libraries = from_json_slice(
                &app.cas
                    .get(libraries)
                    .context(info_span!("fetching game libraries", %game))
                    .await?,
            )
            .context("parsing game libraries")?;
            let manifest = from_json_slice(
                &app.cas
                    .get(manifest)
                    .context(
                        info_span!("fetching game version manifest", %game),
                    )
                    .await?,
            )
            .context("parsing game version manifest")?;
            let mojang_libraries = match mojang_manifests
                .get(&mojang_game_version(&game))
            {
                Some(sha256) => {
                    from_json_slice::<ornithe::VersionManifest>(
                        &app.cas
                            .get(*sha256)
                            .context(
                                info_span!("fetching Mojang version manifest", %game),
                            )
                            .await?,
                    )
                    .context("parsing Mojang version manifest")?
                    .libraries
                }
                None => Vec::new(),
            };
            anyhow::Ok(GameLibraries::new(
                game,
                libraries,
                manifest,
                &mojang_libraries,
            ))
        })
        .buffered(app.concurrency.download.get())
        .try_collect::<Vec<_>>()
        .await?;

    let profile_base_url = Url::parse(ornithe::PROFILE_BASE_URL)?;
    let groups = profile_groups(game_libraries)
        .into_iter()
        .filter_map(|group| {
            let fallback_templates = if group.templated {
                group
                    .games
                    .iter()
                    .filter(|game| **game != group.template)
                    .map(ToString::to_string)
                    .collect()
            } else {
                Vec::new()
            };
            let games = group
                .games
                .iter()
                .map(mojang_game_version)
                .filter(|game| mojang_manifests.contains_key(game))
                .collect::<Vec<_>>();
            if games.is_empty() {
                debug!(group = %group.id, "skipping group without Mojang game versions");
                return None;
            }
            Some(ProfileGroup {
                id: Some(group.id),
                template: group.template.to_string(),
                fallback_templates,
                games,
                profile_base_url: profile_base_url.clone(),
                templated: group.templated,
                extra_libraries: group
                    .libraries
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                removed_libraries: group.removed_libraries,
                java_version: group.java_version,
            })
        })
        .collect::<Vec<_>>();
    let grouped_games = groups
        .iter()
        .flat_map(|group| group.games.iter().cloned())
        .collect::<HashSet<_>>();

    let catalogs = snapshots
        .into_iter()
        .map(
            |(download_run_id, sha256, catalog_games, catalog_loaders)| {
                CatalogSnapshot {
                    download_run_id,
                    sha256,
                    games: catalog_games
                        .into_iter()
                        .map(|game| CatalogGame {
                            id: mojang_game_version(&game.version),
                            stable: game.stable,
                        })
                        .filter(|game| grouped_games.contains(&game.id))
                        .collect(),
                    loaders: catalog_loaders
                        .into_iter()
                        .map(|loader| CatalogLoader {
                            id: loader.version.to_string(),
                            stable: loader.stable,
                        })
                        .collect(),
                    mappings: Vec::new(),
                }
            },
        )
        .collect();

    fabriclike::extract_catalogs(
        app,
        FabriclikeLoader::Ornithe,
        &Url::parse(ornithe::MAVEN_URL)?,
        catalogs,
        groups,
    )
    .context(info_span!(
        "extracting catalogs",
        loader = ?FabriclikeLoader::Ornithe
    ))
    .await
}
