use std::{
    collections::{BTreeMap, HashMap},
    sync::atomic::{AtomicUsize, Ordering},
};

use anyhow::{Context, Result, ensure};
use futures::{StreamExt, TryStreamExt, stream};
use indexmap::IndexMap;
use minecraft_meta_manifest::fabriclike::{
    FabriclikeProfileMetadata, GameVersion, LoaderVersion, Manifest,
    VersionGroup,
};
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;
use url::Url;

use crate::{
    AppState,
    model::{
        FabriclikeCatalog, FabriclikeGameVersion, FabriclikeLoader,
        FabriclikeProfile, FabriclikeProfileGroup, FabriclikeStoredArtifact,
    },
    store::ContentType,
    util::TaskContext,
};
use crate::{model::GAME_VERSION_PLACEHOLDER, store::PutStatus};

/// Publishes extracted catalog state and profiles, with store-specific URLs.
pub async fn export(
    app: &AppState,
    maven: &super::maven::MavenExports<'_>,
    loader: FabriclikeLoader,
    format_version: u32,
) -> Result<PutStatus> {
    let (catalog, rows) = fetch_catalog(app, loader).await?;
    ensure!(
        !rows.is_empty(),
        "no extracted {loader:?} profiles; run extraction before exporting"
    );
    let games = catalog
        .games
        .0
        .into_iter()
        .map(|game| (game.id.clone(), game))
        .collect();
    let groups = catalog.groups.0;
    let artifacts = deduplicate_artifacts(catalog.artifacts.0)?;
    let public_maven = app.public_blobs.url_for("maven/");
    let mut profiles = Vec::with_capacity(rows.len());
    for row in rows {
        let group = if row.version_group.is_empty() {
            None
        } else {
            Some(row.version_group)
        };
        ensure!(
            groups.iter().any(|candidate| candidate.id == group),
            "extracted profile {} has unknown group {:?}",
            row.loader_version,
            group
        );
        let path = profile_path(
            loader,
            format_version,
            group.as_deref(),
            &row.loader_version,
        );
        let profile = prepare_profile(row.metadata.0, &public_maven);
        profiles.push(ProfileExport {
            loader: LoaderVersion {
                id: row.loader_version,
                stable: row.stable,
                url: app.public_blobs.url_for(&path),
            },
            group,
            path,
            json: serde_json::to_vec(&profile)
                .context("serializing loader profile")?,
        });
    }
    let mut profile_files = BTreeMap::new();
    for profile in &profiles {
        if let Some(existing) = profile_files.get(&profile.path) {
            ensure!(
                existing == &profile.json,
                "conflicting loader profiles at {}",
                profile.path
            );
        } else {
            profile_files.insert(profile.path.clone(), profile.json.clone());
        }
    }
    let manifest = make_manifest(games, &groups, profiles)?;
    let json =
        serde_json::to_vec(&manifest).context("serializing loader manifest")?;

    // publish libraries before publishing profiles which reference them
    let num_total = artifacts.len();
    info!(num_artifacts = num_total, "publishing loader dependencies");
    let num_done = AtomicUsize::new(0);
    let mut status =
        stream::iter(artifacts.into_iter().map(|artifact| async {
            let changed = if maven
                .is_published(&artifact.artifact.coordinate, artifact.sha256)
                .await
            {
                PutStatus::Unchanged
            } else {
                publish_artifact(app, maven, artifact)
                    .transaction_context(|| {
                        info_span!("publishing loader artifact", ?loader)
                    })
                    .await?
            };
            let num_done = num_done.fetch_add(1, Ordering::Relaxed) + 1;
            if num_done.is_multiple_of(100) || num_done == num_total {
                info!("processed {num_done}/{num_total} dependencies");
            }
            anyhow::Ok(changed)
        }))
        .buffer_unordered(app.concurrency.download.get())
        .try_fold(PutStatus::Unchanged, |mut status, changed| async move {
            super::latch(&mut status, changed);
            Ok(status)
        })
        .context(info_span!("publishing loader dependencies"))
        .await?;
    // write processed loader profiles
    let num_profiles = profile_files.len();
    super::latch(
        &mut status,
        super::publish_json_files(app, profile_files).await?,
    );
    info!(num_profiles, "wrote loader profiles");
    // write manifest (overall catalog), after all referenced files are uploaded
    let path = match loader {
        FabriclikeLoader::Fabric => {
            format!("fabric/v{format_version}/manifest.json")
        }
        FabriclikeLoader::Quilt => {
            format!("quilt/v{format_version}/manifest.json")
        }
    };
    super::latch(
        &mut status,
        app.public_blobs
            .put(&path, &json, ContentType::Json)
            .context(info_span!("writing loader manifest", %path))
            .await?,
    );
    info!(%path, "wrote loader manifest");
    Ok(status)
}

async fn fetch_catalog(
    app: &AppState,
    loader: FabriclikeLoader,
) -> Result<(FabriclikeCatalog, Vec<FabriclikeProfile>)> {
    let mut conn = app
        .db
        .connection()
        .context(info_span!("acquiring connection"))
        .await?;
    // keep both queries on the same snapshot if extraction commits between them
    let mut txn = conn
        .transaction()
        .context(info_span!("starting export snapshot"))
        .await?;
    toasty::sql::statement(
        "SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY",
    )
    .exec(&mut txn)
    .context(info_span!("setting export snapshot isolation"))
    .await?;
    let catalog = FabriclikeCatalog::get_by_loader(&mut txn, loader)
        .context(info_span!("fetching extracted loader catalog", ?loader))
        .await?;
    let mut profiles = FabriclikeProfile::all()
        .filter(FabriclikeProfile::fields().loader().eq(loader))
        .exec(&mut txn)
        .context(info_span!("fetching extracted loader profiles", ?loader))
        .await?;
    txn.commit()
        .context(info_span!("finishing export snapshot"))
        .await?;
    profiles.sort_by_key(|profile| profile.position);
    Ok((catalog, profiles))
}

fn prepare_profile(
    mut profile: FabriclikeProfileMetadata,
    public_maven: &Url,
) -> FabriclikeProfileMetadata {
    for library in &mut profile.libraries {
        library.url = Some(public_maven.to_string());
    }
    profile
}

/// Publishes a stored artifact from content-addressed storage to the Maven store.
pub async fn publish_artifact(
    app: &AppState,
    maven: &super::maven::MavenExports<'_>,
    stored: FabriclikeStoredArtifact,
) -> Result<crate::store::PutStatus> {
    maven
		.publish(app, &stored.artifact.coordinate, stored.sha256)
        .context(info_span!("publishing Maven artifact", coordinate = %stored.artifact.coordinate))
        .await
}

fn deduplicate_artifacts(
    artifacts: impl IntoIterator<Item = FabriclikeStoredArtifact>,
) -> Result<Vec<FabriclikeStoredArtifact>> {
    let mut paths = BTreeMap::<String, FabriclikeStoredArtifact>::new();
    for stored in artifacts {
        let path = stored.artifact.coordinate.to_maven_path();
        if let Some(existing) = paths.get(&path) {
            ensure!(
                existing.sha256 == stored.sha256,
                "conflicting Maven artifacts at {path}: {} and {}",
                existing.sha256,
                stored.sha256
            );
        } else {
            paths.insert(path, stored);
        }
    }
    Ok(paths.into_values().collect())
}

struct ProfileExport {
    loader: LoaderVersion,
    group: Option<String>,
    path: String,
    json: Vec<u8>,
}

fn profile_path(
    loader: FabriclikeLoader,
    format_version: u32,
    group: Option<&str>,
    version: &str,
) -> String {
    let loader = match loader {
        FabriclikeLoader::Fabric => "fabric",
        FabriclikeLoader::Quilt => "quilt",
    };
    match group {
        None => format!("{loader}/v{format_version}/versions/{version}.json"),
        Some(group) => format!(
            "{loader}/v{format_version}/version-group/{group}/loader-version/{version}"
        ),
    }
}

fn make_manifest(
    games: IndexMap<String, FabriclikeGameVersion>,
    groups: &[FabriclikeProfileGroup],
    profiles: Vec<ProfileExport>,
) -> Result<Manifest> {
    let mut manifest = Manifest {
        game_versions: Vec::new(),
        version_groups: Vec::new(),
    };
    let mut grouped_loaders: IndexMap<Option<String>, Vec<LoaderVersion>> =
        IndexMap::new();
    for profile in profiles {
        grouped_loaders
            .entry(profile.group)
            .or_default()
            .push(profile.loader);
    }
    let mut game_groups = HashMap::new();
    for group in groups {
        let loaders =
            grouped_loaders.shift_remove(&group.id).unwrap_or_default();
        if let Some(id) = &group.id {
            manifest.version_groups.push(VersionGroup {
                id: id.clone(),
                loaders,
            });
        } else {
            manifest.game_versions.push(GameVersion {
                id: GAME_VERSION_PLACEHOLDER.to_owned(),
                stable: true,
                version_group: None,
                loaders,
            });
        }
        for game in &group.games {
            ensure!(
                game_groups.insert(game.clone(), group.id.clone()).is_none(),
                "game version {game} belongs to multiple profile groups"
            );
        }
    }
    for game in games.into_values() {
        let version_group =
            game_groups.remove(&game.id).with_context(|| {
                format!("game version {} has no profile group", game.id)
            })?;
        manifest.game_versions.push(GameVersion {
            id: game.id,
            stable: game.stable,
            version_group,
            loaders: Vec::new(),
        });
    }
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn stored_artifacts_preserve_flattened_provenance_and_require_content_hash()
    {
        let mut value = json!({
            "coordinate": "net.fabricmc:fabric-loader:0.19.5",
            "repository": "https://maven.fabricmc.net/",
            "source": "downloaded profile",
            "repository_source": "profile library.url",
            "sha256": "0000000000000000000000000000000000000000000000000000000000000000"
        });
        let stored: FabriclikeStoredArtifact =
            serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(stored).unwrap(), value);
        value.as_object_mut().unwrap().remove("sha256");
        assert!(
            serde_json::from_value::<FabriclikeStoredArtifact>(value).is_err()
        );
    }

    fn group(
        id: Option<&str>,
        template: &str,
        games: &[&str],
    ) -> FabriclikeProfileGroup {
        FabriclikeProfileGroup {
            id: id.map(str::to_owned),
            template: template.into(),
            fallback_templates: Vec::new(),
            games: games.iter().map(|game| (*game).into()).collect(),
            profile_base_url: "https://meta.example.com/versions/loader/"
                .parse()
                .unwrap(),
        }
    }

    fn exported(group: Option<&str>, version: &str) -> ProfileExport {
        ProfileExport {
            loader: LoaderVersion {
                id: version.into(),
                url: "https://example.com/profile.json".parse().unwrap(),
                stable: true,
            },
            group: group.map(str::to_owned),
            path: String::new(),
            json: vec![],
        }
    }

    fn catalog_games(ids: &[&str]) -> IndexMap<String, FabriclikeGameVersion> {
        ids.iter()
            .map(|id| {
                (
                    (*id).into(),
                    FabriclikeGameVersion {
                        id: (*id).into(),
                        stable: true,
                    },
                )
            })
            .collect()
    }

    #[test]
    fn universal_manifest_preserves_fabric_shape() {
        let manifest = make_manifest(
            catalog_games(&["1.21", "26.3"]),
            &[group(None, "1.21", &["1.21", "26.3"])],
            vec![exported(None, "0.19.5")],
        )
        .unwrap();
        let json = serde_json::to_value(manifest).unwrap();
        assert_eq!(json["gameVersions"][0]["id"], GAME_VERSION_PLACEHOLDER);
        assert_eq!(json["gameVersions"][0]["loaders"][0]["id"], "0.19.5");
        assert_eq!(json["gameVersions"][1]["loaders"], json!([]));
        assert!(json["gameVersions"][1].get("versionGroup").is_none());
        assert!(json.get("versionGroups").is_none());
    }

    #[test]
    fn grouped_manifest_has_quilt_group_references() {
        let manifest = make_manifest(
            catalog_games(&["26.3", "1.21"]),
            &[
                group(Some("v1"), "1.21", &["1.21"]),
                group(Some("v2"), "26.3", &["26.3"]),
            ],
            vec![
                exported(Some("v1"), "0.30.0"),
                exported(Some("v2"), "0.30.0"),
            ],
        )
        .unwrap();
        let json = serde_json::to_value(manifest).unwrap();
        assert_eq!(json["gameVersions"].as_array().unwrap().len(), 2);
        assert_eq!(json["gameVersions"][0]["versionGroup"], "v2");
        assert_eq!(json["gameVersions"][1]["versionGroup"], "v1");
        assert_eq!(json["versionGroups"][0]["id"], "v1");
        assert_eq!(json["versionGroups"][1]["loaders"][0]["id"], "0.30.0");
        assert_eq!(json["gameVersions"][0]["loaders"], json!([]));
    }

    #[test]
    fn profile_locations_match_published_formats() {
        assert_eq!(
            profile_path(FabriclikeLoader::Fabric, 0, None, "0.19.5"),
            "fabric/v0/versions/0.19.5.json"
        );
        assert_eq!(
            profile_path(FabriclikeLoader::Quilt, 1, Some("v2"), "0.30.0"),
            "quilt/v1/version-group/v2/loader-version/0.30.0"
        );
    }

    #[test]
    fn publication_applies_current_maven_url_without_changing_extracted_metadata()
     {
        let profile: FabriclikeProfileMetadata = serde_json::from_value(json!({
            "id": "fabric-loader-1-${modrinth.gameVersion}",
            "inheritsFrom": "${modrinth.gameVersion}",
            "releaseTime": "2026-01-01T00:00:00Z",
            "time": "2026-01-01T00:00:00Z",
            "type": "release",
            "libraries": [
                {"name": "net.fabricmc:intermediary:${modrinth.gameVersion}", "url": "https://maven.fabricmc.net/"},
                {"name": "net.fabricmc:fabric-loader:1"}
            ]
        })).unwrap();
        let original = serde_json::to_value(&profile).unwrap();
        for base in [
            "https://first.example.com/maven/",
            "https://second.example.com/meta/maven/",
        ] {
            let published =
                prepare_profile(profile.clone(), &base.parse().unwrap());
            let mut expected = original.clone();
            for library in expected["libraries"].as_array_mut().unwrap() {
                library["url"] = json!(base);
            }
            assert_eq!(serde_json::to_value(published).unwrap(), expected);
        }
    }
}
