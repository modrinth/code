use anyhow::{Context, Result, anyhow, ensure};
use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

use futures::{TryStreamExt, stream};
use indexmap::IndexMap;

use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use tracing::{info, info_span, warn};
use tracing_anyhow::FutureContext;
use url::Url;

use crate::{
    AppState,
    model::{self, Processor, SidedDataEntry},
    store::ContentType,
    upstream::mojang::{
        Argument, ArgumentType, LibraryDownloads, LibraryExtract,
        OperatingSystem, Rule, VersionType,
    },
    util::{MavenCoordinate, ResponseExt, Sha256, from_json_slice},
};

pub const GAME_VERSION_PLACEHOLDER: &str = "${modrinth.gameVersion}";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileMetadata {
    pub id: String,
    pub inherits_from: String,
    pub release_time: Timestamp,
    pub time: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_arguments: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<HashMap<ArgumentType, Vec<Argument>>>,
    pub libraries: Vec<Library>,
    #[serde(rename = "type")]
    pub ty: VersionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<HashMap<String, SidedDataEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processors: Option<Vec<Processor>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloads: Option<LibraryDownloads>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extract: Option<LibraryExtract>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub natives: Option<HashMap<OperatingSystem, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<Rule>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksums: Option<Vec<String>>,
    #[serde(default = "default_true")]
    pub include_in_classpath: bool,
    #[serde(default = "default_true")]
    pub downloadable: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone)]
pub struct Artifact {
    pub coordinate: MavenCoordinate,
    pub repository: Url,
    pub source: Option<String>,
    pub repository_source: &'static str,
}

pub struct CatalogGame {
    pub id: String,
    pub stable: bool,
}

pub struct CatalogLoader {
    pub id: String,
    pub stable: bool,
}

pub struct CatalogSnapshot {
    pub download_run_id: model::DownloadRunId,
    pub sha256: Sha256,
    pub games: Vec<CatalogGame>,
    pub loaders: Vec<CatalogLoader>,
    pub mappings: Vec<Artifact>,
}

pub struct ProfileGroup {
    pub id: Option<String>,
    pub template: String,
    pub fallback_templates: Vec<String>,
    pub games: Vec<String>,
    pub profile_base_url: Url,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    game_versions: Vec<GameVersion>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    version_groups: Vec<VersionGroup>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GameVersion {
    id: String,
    stable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    version_group: Option<String>,
    loaders: Vec<LoaderVersion>,
}

#[derive(Debug, Serialize)]
struct LoaderVersion {
    id: String,
    url: Url,
    stable: bool,
}

#[derive(Debug, Serialize)]
struct VersionGroup {
    id: String,
    loaders: Vec<LoaderVersion>,
}

struct ProfileExport {
    loader: LoaderVersion,
    group: Option<String>,
    path: String,
    json: Vec<u8>,
}

fn profile_source(
    group: &ProfileGroup,
    template: &str,
    loader_version: &str,
) -> Result<Url> {
    let mut url = group.profile_base_url.clone();
    url.path_segments_mut()
        .map_err(|()| anyhow!("profile base URL cannot contain path segments"))?
        .pop_if_empty()
        .extend([template, loader_version, "profile", "json"]);
    Ok(url)
}

fn profile_path(
    loader: &str,
    format_version: u32,
    group: Option<&str>,
    version: &str,
) -> String {
    match group {
        None => format!("{loader}/v{format_version}/versions/{version}.json"),
        Some(group) => format!(
            "{loader}/v{format_version}/version-group/{group}/loader-version/{version}"
        ),
    }
}

fn make_manifest(
    games: IndexMap<String, CatalogGame>,
    groups: &[ProfileGroup],
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

/// Publishes profiles and a catalog from snapshots ordered newest-first.
pub async fn export_catalogs(
    app: &AppState,
    loader: &str,
    format_version: u32,
    fallback_maven: &Url,
    catalogs: Vec<CatalogSnapshot>,
    groups: Vec<ProfileGroup>,
) -> Result<()> {
    let mut games = IndexMap::new();
    let mut loaders = IndexMap::new();
    let mut artifacts = IndexMap::new();
    let mut run_priority = HashMap::new();
    for (priority, snapshot) in catalogs.into_iter().enumerate() {
        run_priority
            .entry(snapshot.download_run_id)
            .or_insert(priority);
        for game in snapshot.games {
            games.entry(game.id.clone()).or_insert(game);
        }
        for version in snapshot.loaders {
            loaders.entry(version.id.clone()).or_insert(version);
        }
        for mut artifact in snapshot.mappings {
            if artifact.source.is_none() {
                artifact.source =
                    Some(format!("catalog SHA256 {}", snapshot.sha256));
            }
            artifacts
                .entry(artifact.coordinate.to_maven_path())
                .or_insert(artifact);
        }
    }
    ensure!(
        !games.is_empty() && !loaders.is_empty() && !groups.is_empty(),
        "no {loader} game versions, loaders, or profile groups to export"
    );
    let mut profile_urls = Vec::new();
    for group in &groups {
        for version in loaders.keys() {
            for template in std::iter::once(&group.template)
                .chain(&group.fallback_templates)
            {
                profile_urls.push(
                    profile_source(group, template, version)?.to_string(),
                );
            }
        }
    }
    let mut conn = app
        .db
        .connection()
        .context(info_span!("acquiring connection"))
        .await?;
    let mut downloads = model::BlobDownload::all()
        .filter(model::BlobDownload::fields().url().in_list(profile_urls))
        .exec(&mut conn)
        .context(info_span!("fetching downloaded profiles", loader))
        .await?;
    downloads.sort_by_key(|row| {
        (
            run_priority
                .get(&row.download_run_id)
                .copied()
                .unwrap_or(usize::MAX),
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
    for group in &groups {
        for version in loaders.values() {
            let mut selected = None;
            for template in std::iter::once(&group.template)
                .chain(&group.fallback_templates)
            {
                let source = profile_source(group, template, &version.id)?;
                if let Some(sha256) = profile_hashes.get(source.as_str()) {
                    selected = Some((template, source, *sha256));
                    break;
                }
            }
            let (template, source, sha256) = selected.with_context(|| {
                format!(
                    "no downloaded {loader} profile for {} in group {:?}",
                    version.id, group.id
                )
            })?;
            let bytes = app.cas.get(sha256)
				.context(info_span!("fetching loader profile", loader, loader_version = %version.id, %source))
				.await?;
            let profile = from_json_slice::<ProfileMetadata>(&bytes)
                .context("parsing loader profile")?;
            let (profile, dependencies) = normalize_profile(
                profile,
                template,
                &group.games,
                fallback_maven,
                &public_maven,
            )
            .with_context(|| {
                format!(
                    "normalizing {loader} profile {} from {source}",
                    version.id
                )
            })?;
            for mut artifact in dependencies {
                artifact.source =
                    Some(format!("{source} (profile SHA256 {sha256})"));
                artifacts
                    .entry(artifact.coordinate.to_maven_path())
                    .or_insert(artifact);
            }
            let path = profile_path(
                loader,
                format_version,
                group.id.as_deref(),
                &version.id,
            );
            profiles.push(ProfileExport {
                loader: LoaderVersion {
                    id: version.id.clone(),
                    url: app.public_blobs.url_for(&path),
                    stable: version.stable,
                },
                group: group.id.clone(),
                path,
                json: serde_json::to_vec(&profile)
                    .context("serializing loader profile")?,
            });
        }
    }
    let num_total = artifacts.len();
    info!(
        loader,
        num_artifacts = num_total,
        "mirroring loader dependencies"
    );
    let num_done = AtomicUsize::new(0);
    stream::iter(artifacts.into_values().map(Ok::<_, anyhow::Error>))
        .try_for_each_concurrent(
            app.concurrency.download.get(),
            |artifact| async {
                mirror_artifact(app, artifact).await?;
                let num_done = num_done.fetch_add(1, Ordering::Relaxed) + 1;
                if num_done.is_multiple_of(100) || num_done == num_total {
                    info!(
                        loader,
                        "mirrored {num_done}/{num_total} dependencies"
                    );
                }
                anyhow::Ok(())
            },
        )
        .context(info_span!("mirroring loader dependencies", loader))
        .await?;
    for profile in &profiles {
        app.public_blobs
			.put(&profile.path, &profile.json, ContentType::Json)
			.context(
				info_span!("writing loader profile", loader, path = %profile.path),
			)
			.await?;
    }
    info!(
        loader,
        num_profiles = profiles.len(),
        "wrote loader profiles"
    );
    let manifest = make_manifest(games, &groups, profiles)?;
    let json =
        serde_json::to_vec(&manifest).context("serializing loader manifest")?;
    let path = format!("{loader}/v{format_version}/manifest.json");
    app.public_blobs
        .put(&path, &json, ContentType::Json)
        .context(info_span!("writing loader manifest", %path))
        .await?;
    info!(%path, "wrote loader manifest");
    Ok(())
}

/// Normalizes a Fabric or Quilt profile for multiple game versions.
/// Artifacts are concrete; callers should deduplicate them by Maven path.
pub fn normalize_profile(
    mut profile: ProfileMetadata,
    template: &str,
    games: &[String],
    fallback_maven: &Url,
    public_maven: &Url,
) -> Result<(ProfileMetadata, Vec<Artifact>)> {
    profile.id = profile.id.replace(template, GAME_VERSION_PLACEHOLDER);
    profile.inherits_from = profile
        .inherits_from
        .replace(template, GAME_VERSION_PLACEHOLDER);

    let mut artifacts = Vec::new();
    for (index, library) in profile.libraries.iter_mut().enumerate() {
        let name = library.name.clone();
        let coordinate =
            name.parse::<MavenCoordinate>().with_context(|| {
                format!("library {index} has an invalid Maven name")
            })?;

        let repository_source = if name == "net.minecraft:launchwrapper:1.12" {
            "launchwrapper override"
        } else if library.url.is_some() {
            "profile library.url"
        } else {
            "loader Maven fallback"
        };
        let repository = if name == "net.minecraft:launchwrapper:1.12" {
            // special case this because it's not available on Fabric's Maven repo
            Url::parse("https://libraries.minecraft.net/")?
        } else {
            match &library.url {
				None => fallback_maven.clone(),
				Some(value) => Url::parse(value).with_context(|| {
					anyhow!(
						"library {index} has an invalid repository URL {value:?}"
					)
				})?,
			}
        };

        // rewrite the specific `template` (game version)
        // to `GAME_VERSION_PLACEHOLDER`, so that app consumer
        // can replace the placeholder to the real game version later.
        // we only rewrite the coordinate if it's one of the ones
        // that we know contains a game version
        // (like fabric intermediary coordinate).
        if matches!(
            (
                coordinate.group_id.as_str(),
                coordinate.artifact_id.as_str()
            ),
            ("net.fabricmc", "intermediary") | ("org.quiltmc", "hashed")
        ) {
            ensure!(
                coordinate.version.as_str() == template,
                "library {index} mapping version {} does not match template {template:?}",
                coordinate.version
            );
            for game in games {
                let mut concrete = coordinate.clone();
                concrete.version = game.parse().with_context(|| {
                    format!(
                        "library {index} has an invalid game version {game:?}"
                    )
                })?;
                artifacts.push(Artifact {
                    coordinate: concrete,
                    repository: repository.clone(),
                    source: None,
                    repository_source,
                });
            }
            let classifier = coordinate
                .classifier
                .as_ref()
                .map(|value| format!(":{value}"))
                .unwrap_or_default();
            let extension = coordinate
                .extension
                .as_ref()
                .map(|value| format!("@{value}"))
                .unwrap_or_default();
            library.name = format!(
                "{}:{}:{GAME_VERSION_PLACEHOLDER}{classifier}{extension}",
                coordinate.group_id, coordinate.artifact_id
            );
        } else {
            artifacts.push(Artifact {
                coordinate,
                repository,
                source: None,
                repository_source,
            });
        }

        library.url = Some(public_maven.to_string());
    }
    Ok((profile, artifacts))
}

fn artifact_url(artifact: &Artifact) -> Result<Url> {
    let mut url = artifact.repository.clone();

    // some old Fabric artifacts, 0.1.0.48 through 0.1.0.52, have libraries
    // which reference apache maven using HTTP.
    // apache maven no longer accepts that, and tells clients to upgrade.
    // we do this upgrade manually.
    if url.scheme() == "http" {
        warn!(
            %url,
            coordinate = %artifact.coordinate,
            source = artifact.source.as_deref().unwrap_or("unknown"),
            repository = %artifact.repository,
            repository_source = artifact.repository_source,
            "upgraded Maven artifact URL from HTTP to HTTPS",
        );

        url.set_scheme("https").map_err(|()| {
            anyhow!("cannot upgrade Maven repository URL to HTTPS")
        })?;
    }
    let path = artifact.coordinate.to_maven_path();
    let relative = path
        .strip_prefix("maven/")
        .context("missing Maven path prefix")?;
    url.path_segments_mut()
        .map_err(|()| {
            anyhow!("repository URL cannot be a base: {}", artifact.repository)
        })?
        .pop_if_empty()
        .extend(relative.split('/'));
    Ok(url)
}

/// Downloads a concrete dependency and stores it in the public Maven mirror.
pub async fn mirror_artifact(app: &AppState, artifact: Artifact) -> Result<()> {
    let url = artifact_url(&artifact)?;
    let bytes = async {
		app.http
			.get(url.clone())
			.send()
			.await?
			.error_for_status_ext()
			.await?
			.bytes()
			.await
			.map_err(anyhow::Error::from)
	}
	.context(info_span!("fetching Maven artifact", %url, coordinate = %artifact.coordinate, source = artifact.source.as_deref().unwrap_or("unknown"), repository = %artifact.repository, repository_source = artifact.repository_source, scheme = url.scheme()))
	.await?;

    app.maven
		.put(&artifact.coordinate, &bytes)
		.context(
			info_span!("storing Maven artifact", coordinate = %artifact.coordinate),
		)
		.await
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    fn group(id: Option<&str>, template: &str, games: &[&str]) -> ProfileGroup {
        ProfileGroup {
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

    fn catalog_games(ids: &[&str]) -> IndexMap<String, CatalogGame> {
        ids.iter()
            .map(|id| {
                (
                    (*id).into(),
                    CatalogGame {
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
    fn profile_locations_match_published_formats_and_source_encoding() {
        assert_eq!(
            profile_path("fabric", 0, None, "0.19.5"),
            "fabric/v0/versions/0.19.5.json"
        );
        assert_eq!(
            profile_path("quilt", 1, Some("v2"), "0.30.0"),
            "quilt/v1/version-group/v2/loader-version/0.30.0"
        );
        assert_eq!(
            profile_source(
                &group(None, "1.14 Pre-Release 1", &[]),
                "1.14 Pre-Release 1",
                "0.4.1+build.128"
            )
            .unwrap()
            .as_str(),
            "https://meta.example.com/versions/loader/1.14%20Pre-Release%201/0.4.1+build.128/profile/json"
        );
    }

    fn normalize(
        mut profile: Value,
        games: &[String],
    ) -> Result<(Value, Vec<Artifact>)> {
        if let Some(object) = profile.as_object_mut() {
            object.insert("releaseTime".into(), json!("2026-01-01T00:00:00Z"));
            object.insert("time".into(), json!("2026-01-01T00:00:00Z"));
            object.insert("type".into(), json!("release"));
        }
        let (profile, artifacts) = normalize_profile(
            serde_json::from_value(profile)?,
            "GAME",
            games,
            &Url::parse("https://maven.fabricmc.net/").unwrap(),
            &Url::parse("https://example.com/maven/").unwrap(),
        )?;
        Ok((serde_json::to_value(profile)?, artifacts))
    }

    #[test]
    fn substitutes_templates_and_expands_concrete_artifacts() {
        let (profile, artifacts) = normalize(
            json!({
                "id": "fabric-GAME-loader",
                "inheritsFrom": "GAME",
                "libraries": [{"name": "net.fabricmc:intermediary:GAME"}]
            }),
            &["1.21.1".to_owned(), "1.14.2 Pre-Release 4".to_owned()],
        )
        .unwrap();
        assert_eq!(
            profile["id"],
            format!("fabric-{GAME_VERSION_PLACEHOLDER}-loader")
        );
        assert_eq!(profile["inheritsFrom"], GAME_VERSION_PLACEHOLDER);
        assert_eq!(
            profile["libraries"][0]["name"],
            format!("net.fabricmc:intermediary:{GAME_VERSION_PLACEHOLDER}")
        );

        assert_eq!(artifacts.len(), 2);
        assert_eq!(
            artifacts[0].coordinate.to_string(),
            "net.fabricmc:intermediary:1.21.1"
        );
        assert_eq!(
            artifacts[1].coordinate.to_string(),
            "net.fabricmc:intermediary:1.14.2 Pre-Release 4"
        );
        assert_eq!(
            artifacts[0].repository.as_str(),
            "https://maven.fabricmc.net/"
        );
    }

    #[test]
    fn only_mapping_coordinates_are_templated() {
        let (profile, artifacts) = normalize(
            json!({
                "id": "GAME", "inheritsFrom": "GAME",
                "libraries": [
                    {"name": "net.fabricmc:sponge-mixin:0.7.11.21"},
                    {"name": "net.fabricmc:sponge-mixin:GAME"},
                    {"name": "other:intermediary:GAME"},
                    {"name": "net.fabricmc:intermediary:GAME:GAME@jar"},
                    {"name": "org.quiltmc:hashed:GAME"}
                ]
            }),
            &["1.21.1".into(), "26.4-snapshot-3".into()],
        )
        .unwrap();
        assert_eq!(artifacts.len(), 7);
        for (index, name) in [
            "net.fabricmc:sponge-mixin:0.7.11.21",
            "net.fabricmc:sponge-mixin:GAME",
            "other:intermediary:GAME",
        ]
        .iter()
        .enumerate()
        {
            assert_eq!(profile["libraries"][index]["name"], *name);
            assert_eq!(artifacts[index].coordinate.to_string(), *name);
        }
        assert_eq!(
            profile["libraries"][3]["name"],
            format!(
                "net.fabricmc:intermediary:{GAME_VERSION_PLACEHOLDER}:GAME@jar"
            )
        );
        assert_eq!(
            artifacts[3].coordinate.to_string(),
            "net.fabricmc:intermediary:1.21.1:GAME@jar"
        );
        assert_eq!(
            artifacts[4].coordinate.to_string(),
            "net.fabricmc:intermediary:26.4-snapshot-3:GAME@jar"
        );
        assert_eq!(
            profile["libraries"][4]["name"],
            format!("org.quiltmc:hashed:{GAME_VERSION_PLACEHOLDER}")
        );
    }

    #[test]
    fn rejects_mapping_versions_that_do_not_match_template() {
        assert!(
            normalize(
                json!({
                    "id": "GAME", "inheritsFrom": "GAME",
                    "libraries": [{"name": "net.fabricmc:intermediary:OTHER"}]
                }),
                &["1.21.1".into()]
            )
            .is_err()
        );
    }

    #[test]
    fn launchwrapper_uses_minecraft_repository() {
        let (profile, artifacts) = normalize(
            json!({
                "id": "GAME", "inheritsFrom": "GAME",
                "libraries": [{
                    "name": "net.minecraft:launchwrapper:1.12",
                    "url": "https://maven.fabricmc.net/"
                }]
            }),
            &[],
        )
        .unwrap();
        assert_eq!(
            artifacts[0].repository.as_str(),
            "https://libraries.minecraft.net/"
        );
        assert_eq!(
            profile["libraries"][0]["url"],
            "https://example.com/maven/"
        );
    }

    #[test]
    fn preserves_concrete_libraries_and_emits_daedalus_defaults() {
        let (profile, artifacts) = normalize(json!({
			"id": "loader", "inheritsFrom": "1.21.1",
			"libraries": [
				{"name": "org.quiltmc:quilt-loader:1", "url": "https://maven.quiltmc.org/repository/release/", "include_in_classpath": false, "downloadable": false, "rules": []},
				{"name": "net.fabricmc:fabric-loader:1"}
			]
		}), &["1.21.1".to_owned(), "1.21.2".to_owned()]).unwrap();
        assert_eq!(artifacts.len(), 2);
        assert_eq!(
            artifacts[0].repository.as_str(),
            "https://maven.quiltmc.org/repository/release/"
        );
        assert_eq!(
            artifacts[1].repository.as_str(),
            "https://maven.fabricmc.net/"
        );
        assert_eq!(
            profile["libraries"][0]["name"],
            "org.quiltmc:quilt-loader:1"
        );
        assert_eq!(profile["libraries"][0]["include_in_classpath"], false);
        assert_eq!(profile["libraries"][0]["downloadable"], false);
        assert_eq!(profile["libraries"][0]["rules"], json!([]));
        assert_eq!(profile["libraries"][1]["include_in_classpath"], true);
        assert_eq!(profile["libraries"][1]["downloadable"], true);
        assert!(profile.get("arguments").is_none());
        assert!(profile.get("data").is_none());
    }

    #[test]
    fn rejects_invalid_profiles_and_names() {
        for profile in [
            json!(null),
            json!({"id": 1, "inheritsFrom": "GAME", "libraries": []}),
            json!({"id": "GAME", "libraries": []}),
            json!({"id": "GAME", "inheritsFrom": "GAME", "libraries": {}}),
            json!({"id": "GAME", "inheritsFrom": "GAME", "libraries": [null]}),
            json!({"id": "GAME", "inheritsFrom": "GAME", "libraries": [{"name": 1}]}),
            json!({"id": "GAME", "inheritsFrom": "GAME", "libraries": [{"name": "bad/GAME:a:1"}]}),
            json!({"id": "GAME", "inheritsFrom": "GAME", "libraries": [{"name": "g:a:${unknown}"}]}),
        ] {
            assert!(normalize(profile, &[]).is_err());
        }
    }

    #[test]
    fn typed_profiles_round_trip_templates_and_require_metadata() {
        let value = json!({
            "id": "fabric-${modrinth.gameVersion}",
            "inheritsFrom": "${modrinth.gameVersion}",
            "releaseTime": "2026-01-01T00:00:00Z",
            "time": "2026-01-01T00:00:00Z",
            "type": "release",
            "libraries": [{"name": "net.fabricmc:intermediary:${modrinth.gameVersion}"}]
        });
        let profile: ProfileMetadata =
            serde_json::from_value(value.clone()).unwrap();
        let output = serde_json::to_value(&profile).unwrap();
        assert_eq!(
            output["libraries"][0]["name"],
            value["libraries"][0]["name"]
        );
        assert!(profile.libraries[0].include_in_classpath);
        assert!(profile.libraries[0].downloadable);
        assert!(output.get("arguments").is_none());
        assert!(output["libraries"][0].get("downloads").is_none());
        let mut missing = value;
        missing.as_object_mut().unwrap().remove("releaseTime");
        assert!(serde_json::from_value::<ProfileMetadata>(missing).is_err());
    }

    #[test]
    fn records_profile_repository_selection_and_upgrades_http() {
        let (_, artifacts) = normalize(
            json!({
                "id": "GAME", "inheritsFrom": "GAME",
                "libraries": [{
                    "name": "com.github.zafarkhaja:java-semver:0.9.0",
                    "url": "http://repo.maven.apache.org/maven2/"
                }]
            }),
            &[],
        )
        .unwrap();
        assert_eq!(artifacts[0].repository_source, "profile library.url");
        assert_eq!(
            artifact_url(&artifacts[0]).unwrap().as_str(),
            "https://repo.maven.apache.org/maven2/com/github/zafarkhaja/java-semver/0.9.0/java-semver-0.9.0.jar"
        );
    }

    #[test]
    fn artifact_urls_append_encoded_segments_without_store_prefix() {
        let artifact = Artifact {
            coordinate: "net.fabricmc:intermediary:1.14.2 Pre-Release 4"
                .parse()
                .unwrap(),
            repository: Url::parse("https://example.com/repository/release")
                .unwrap(),
            source: None,
            repository_source: "test",
        };
        assert_eq!(
            artifact_url(&artifact).unwrap().as_str(),
            "https://example.com/repository/release/net/fabricmc/intermediary/1.14.2%20Pre-Release%204/intermediary-1.14.2%20Pre-Release%204.jar"
        );
    }
}
