use std::{
	collections::HashMap,
	sync::atomic::{AtomicUsize, Ordering},
};

use anyhow::{Context, Result};
use futures::{TryStreamExt, stream};
use indexmap::IndexMap;
use serde::Serialize;
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;
use url::Url;

use crate::{
	AppState,
	export::fabriclike::{self, Artifact, GAME_VERSION_PLACEHOLDER},
	model,
	store::ContentType,
	upstream::fabric,
	util::from_json_slice,
};

pub const FORMAT_VERSION: u32 = 0;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
	game_versions: Vec<GameVersion>,
}

#[derive(Serialize)]
struct GameVersion {
	id: String,
	stable: bool,
	loaders: Vec<LoaderVersion>,
}

#[derive(Serialize)]
struct LoaderVersion {
	id: String,
	url: Url,
	stable: bool,
}

struct ProfileExport {
	loader: LoaderVersion,
	path: String,
	json: Vec<u8>,
}

pub async fn export(app: &AppState) -> Result<()> {
	let mut conn = app
		.db
		.connection()
		.context(info_span!("acquiring connection"))
		.await?;

	let mut catalog_rows = model::FabricCatalog::all()
		.exec(&mut conn)
		.context(info_span!("fetching Fabric catalogs"))
		.await?;
	info!("found {} downloaded catalogs", catalog_rows.len());

	// sort catalogs by latest-downloaded-first
	let runs = model::DownloadRun::all()
		.exec(&mut conn)
		.context(info_span!("fetching download runs"))
		.await?
		.into_iter()
		.map(|run| (run.id, run.started_at))
		.collect::<HashMap<_, _>>();
	catalog_rows.sort_by_key(|row| {
		(
			std::cmp::Reverse(runs.get(&row.download_run_id).copied()),
			row.id.0,
		)
	});

	let mut games = IndexMap::new();
	let mut loaders = IndexMap::new();
	let mut artifacts = IndexMap::new();
	let repository = Url::parse(fabric::MAVEN_URL)?;
	for row in catalog_rows {
		let bytes = app
			.cas
			.get(row.sha256)
			.context(
				info_span!("fetching Fabric catalog", sha256 = %row.sha256),
			)
			.await?;

		let catalog = from_json_slice::<fabric::Catalog>(&bytes)
			.context("parsing Fabric catalog")?;
		for game in catalog.game {
			games.entry(game.version.clone()).or_insert(game);
		}
		for loader in catalog.loader {
			loaders.entry(loader.version.clone()).or_insert(loader);
		}
		for intermediary in catalog.intermediary {
			artifacts
				.entry(intermediary.maven.to_maven_path())
				.or_insert_with(|| Artifact {
					coordinate: intermediary.maven,
					repository: repository.clone(),
					source: Some(format!(
						"{} (catalog SHA256 {})",
						fabric::CATALOG_URL,
						row.sha256
					)),
					repository_source: "catalog intermediary: loader Maven fallback",
				});
		}
	}
	anyhow::ensure!(
		!games.is_empty() && !loaders.is_empty(),
		"no Fabric game or loader versions to export"
	);
	let game_ids = games
		.values()
		.map(|game| game.version.to_string())
		.collect::<Vec<_>>();

	// get all the individual profile loader blobs that we downloaded
	let profile_urls = loaders
		.values()
		.map(|loader| fabric::profile_url(&loader.version))
		.collect::<Vec<_>>();
	let mut downloads = model::BlobDownload::all()
		.filter(model::BlobDownload::fields().url().in_list(profile_urls))
		.exec(&mut conn)
		.context(info_span!("fetching downloaded profiles"))
		.await?;
	downloads.sort_by_key(|row| {
		(
			std::cmp::Reverse(runs.get(&row.download_run_id).copied()),
			row.download_run_id.0,
		)
	});

	let mut profile_hashes = HashMap::new();
	for download in downloads {
		profile_hashes
			.entry(download.url)
			.or_insert(download.sha256);
	}
	let public_maven = app.public_blobs.url_for("maven/");
	let mut profiles = Vec::new();
	for loader in loaders.into_values() {
		let source = fabric::profile_url(&loader.version);
		let sha256 = profile_hashes.get(&source).with_context(|| {
			format!(
				"no downloaded Fabric profile for {} at {source}",
				loader.version
			)
		})?;
		let bytes = app
			.cas
			.get(*sha256)
			.context(
				info_span!("fetching Fabric profile", loader_version = %loader.version),
			)
			.await?;
		let profile = from_json_slice::<fabriclike::ProfileMetadata>(&bytes)
			.context("parsing Fabric profile")?;
		let (profile, dependencies) = fabriclike::normalize_profile(
			profile,
			fabric::PROFILE_GAME_VERSION,
			&game_ids,
			&repository,
			&public_maven,
		)
		.with_context(|| {
			format!("normalizing Fabric profile {}", loader.version)
		})?;
		for mut artifact in dependencies {
			artifact.source =
				Some(format!("{source} (profile SHA256 {sha256})"));
			artifacts
				.entry(artifact.coordinate.to_maven_path())
				.or_insert(artifact);
		}
		let path = format!(
			"fabric/v{FORMAT_VERSION}/versions/{}.json",
			loader.version
		);
		profiles.push(ProfileExport {
			loader: LoaderVersion {
				id: loader.version.to_string(),
				url: app.public_blobs.url_for(&path),
				stable: loader.stable,
			},
			path,
			json: serde_json::to_vec(&profile)
				.context("serializing Fabric profile")?,
		});
	}

	info!(
		num_artifacts = artifacts.len(),
		"mirroring Fabric dependencies"
	);

	let num_total = artifacts.len();
	let num_done = AtomicUsize::new(0);
	stream::iter(artifacts.into_values().map(Ok::<_, anyhow::Error>))
		.try_for_each_concurrent(
			app.concurrency.download.get(),
			|artifact| async {
				fabriclike::mirror_artifact(app, artifact).await?;
				let num_done = num_done.fetch_add(1, Ordering::Relaxed) + 1;
				if num_done.is_multiple_of(100) || num_done == num_total {
					info!(
						"mirrored {num_done}/{num_total} Fabric dependencies"
					);
				}
				anyhow::Ok(())
			},
		)
		.context(info_span!("mirroring Fabric dependencies"))
		.await?;

	for profile in &profiles {
		app.public_blobs
			.put(&profile.path, &profile.json, ContentType::Json)
			.context(info_span!("writing Fabric profile", path = %profile.path))
			.await?;
	}
	info!(num_profiles = profiles.len(), "wrote Fabric profiles");
	let mut game_versions = vec![GameVersion {
		id: GAME_VERSION_PLACEHOLDER.to_owned(),
		stable: true,
		loaders: profiles.into_iter().map(|profile| profile.loader).collect(),
	}];
	game_versions.extend(games.into_values().map(|game| GameVersion {
		id: game.version.to_string(),
		stable: game.stable,
		loaders: Vec::new(),
	}));
	let json = serde_json::to_vec(&Manifest { game_versions })
		.context("serializing Fabric manifest")?;
	let path = format!("fabric/v{FORMAT_VERSION}/manifest.json");
	app.public_blobs
		.put(&path, &json, ContentType::Json)
		.context(info_span!("writing Fabric manifest", %path))
		.await?;

	info!(%path, "wrote Fabric manifest");

	Ok(())
}
