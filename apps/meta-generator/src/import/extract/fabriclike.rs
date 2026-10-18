use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result, anyhow, ensure};
use futures::{StreamExt, TryStreamExt, stream};
use indexmap::IndexMap;
use tracing::{debug, info, info_span, warn};
use tracing_anyhow::FutureContext;
use url::Url;

use crate::{
    AppState, model,
    model::{
        FabriclikeLoader, FabriclikeStoredArtifact, GAME_VERSION_PLACEHOLDER,
    },
    util::{
        MavenCoordinate, ResponseExt, Sha1, Sha256, TaskContext,
        from_json_slice,
    },
};

pub use crate::model::{
    FabriclikeArtifact as Artifact, FabriclikeGameVersion as CatalogGame,
    FabriclikeProfileGroup as ProfileGroup,
    FabriclikeProfileMetadata as ProfileMetadata,
};

/// Selects and normalizes downloaded profiles without publishing any files.
/// Catalog state and profiles are replaced atomically after all dependencies
/// have been downloaded into CAS.
pub async fn extract_catalogs(
    app: &AppState,
    loader: FabriclikeLoader,
    fallback_maven: &Url,
    catalogs: Vec<CatalogSnapshot>,
    groups: Vec<ProfileGroup>,
) -> Result<()> {
    // merge catalogs, keeping the newest metadata for each version;
    // older catalogs can still contribute versions missing from newer ones
    let required_loaders = catalogs
        .first()
        .into_iter()
        .flat_map(|snapshot| {
            snapshot.loaders.iter().map(|version| version.id.clone())
        })
        .collect::<HashSet<_>>();
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
        "no game versions, loaders, or profile groups to extract"
    );
    // only fetch download records for the profiles we're going to use;
    // include historical templates in case a loader has no current profile
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
        .context(info_span!("fetching downloaded profiles"))
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
    // now we make all the loader profiles and collect their libraries
    let mut selected = Vec::new();
    let mut profiles = Vec::new();
    for group in &groups {
        for version in loaders.values() {
            // prefer the current template, then try older ones from this group;
            // normalize against the template we actually downloaded
            let Some((template, source, sha256)) = select_profile(
                loader,
                group,
                &version.id,
                &profile_hashes,
                required_loaders.contains(&version.id),
            )?
            else {
                warn!(
                    ?loader,
                    loader_version = %version.id,
                    group = ?group.id,
                    "skipping historical loader without a downloaded profile"
                );
                continue;
            };
            selected.push((group, version, template, source, sha256));
        }
    }
    let blobs =
        super::read_blobs(app, selected.iter().map(|entry| entry.4)).await?;
    for (group, version, template, source, sha256) in selected {
        let bytes = &blobs[&sha256];
        let profile = from_json_slice::<ProfileMetadata>(bytes)
            .context("parsing loader profile")?;
        let (profile, dependencies) =
            normalize_profile(profile, template, &group.games, fallback_maven)
                .with_context(|| {
                    format!("normalizing profile {} from {source}", version.id)
                })?;
        for mut artifact in dependencies {
            artifact.source =
                Some(format!("{source} (profile SHA256 {sha256})"));
            artifacts
                .entry(artifact.coordinate.to_maven_path())
                .or_insert(artifact);
        }
        profiles.push(SelectedProfile {
            loader_version: version.id.clone(),
            stable: version.stable,
            version_group: group.id.clone().unwrap_or_default(),
            source_url: source.to_string(),
            source_sha256: sha256,
            template: template.to_owned(),
            metadata: profile,
        });
    }
    drop(blobs);
    validate_groups(&games, &groups)?;
    drop(conn);

    // store dependencies before committing profiles which reference them;
    // a failed download leaves the previous extracted dataset intact
    let num_total = artifacts.len();
    let mut num_done = 0usize;
    info!(num_artifacts = num_total, "downloading loader dependencies");
    let mut artifacts = stream::iter(artifacts.into_values().enumerate())
        .map(|(index, artifact)| async move {
            let coordinate = artifact.coordinate.to_string();
            download_artifact(app, artifact)
                .transaction_context(
                    || info_span!("downloading Maven artifact", %coordinate),
                )
                .await
                .map(|artifact| (index, artifact))
        })
        .buffer_unordered(app.concurrency.download.get())
        .inspect_ok(|_| {
            num_done += 1;
            if num_done.is_multiple_of(100) || num_done == num_total {
                info!("downloaded {num_done}/{num_total} dependencies");
            }
        })
        .try_collect::<Vec<_>>()
        .context(info_span!("downloading loader dependencies"))
        .await?;
    artifacts.sort_unstable_by_key(|(index, _)| *index);
    let artifacts = artifacts
        .into_iter()
        .map(|(_, artifact)| artifact)
        .collect::<Vec<_>>();

    let mut conn = app
        .db
        .connection()
        .context(info_span!("acquiring connection"))
        .await?;
    let mut txn = conn
        .transaction()
        .context(info_span!("starting extraction transaction"))
        .await?;
    model::FabriclikeProfile::all()
        .filter(model::FabriclikeProfile::fields().loader().eq(loader))
        .delete()
        .exec(&mut txn)
        .context(info_span!("clearing previous loader profiles", ?loader))
        .await?;
    let num_profiles = profiles.len();
    let mut batch = model::FabriclikeProfile::create_many();
    for (position, profile) in profiles.into_iter().enumerate() {
        batch = batch.item(
            model::FabriclikeProfile::create()
                .loader(loader)
                .version_group(profile.version_group)
                .loader_version(profile.loader_version)
                .stable(profile.stable)
                .position(
                    i32::try_from(position)
                        .context("too many loader profiles")?,
                )
                .source_url(profile.source_url)
                .source_sha256(profile.source_sha256)
                .template(profile.template)
                .metadata(toasty::Json(profile.metadata)),
        );
        if (position + 1).is_multiple_of(100) {
            std::mem::take(&mut batch)
                .exec(&mut txn)
                .context(info_span!(
                    "storing extracted loader profiles",
                    ?loader
                ))
                .await?;
        }
    }
    batch
        .exec(&mut txn)
        .context(info_span!("storing extracted loader profiles", ?loader))
        .await?;
    model::FabriclikeCatalog::upsert_by_loader(loader)
        .games(toasty::Json(games.into_values().collect::<Vec<_>>()))
        .groups(toasty::Json(groups))
        .artifacts(toasty::Json(artifacts))
        .exec(&mut txn)
        .context(info_span!("storing extracted loader catalog", ?loader))
        .await?;
    txn.commit()
        .context(info_span!("committing extracted loader metadata", ?loader))
        .await?;
    info!(?loader, num_profiles, "stored loader metadata");
    Ok(())
}

fn artifact_url(artifact: &Artifact) -> Result<Url> {
    let mut url = artifact.repository.clone();

    // old profiles reference HTTP repositories which now require HTTPS
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

async fn download_artifact(
    app: &AppState,
    artifact: Artifact,
) -> Result<FabriclikeStoredArtifact> {
    let url = artifact_url(&artifact)?;

    let mut checksum_url = url.clone();
    checksum_url.set_path(&format!("{}.sha256", url.path()));

    let response = app.http.get(checksum_url.clone()).send()
        .context(info_span!("fetching Maven artifact checksum", url = %checksum_url, coordinate = %artifact.coordinate))
        .await?;
    let use_sha1 = response.status() == reqwest::StatusCode::NOT_FOUND;
    let response = if use_sha1 {
        debug!(%url, coordinate = %artifact.coordinate, "Maven artifact SHA256 checksum unavailable; using SHA1");
        checksum_url.set_path(&format!("{}.sha1", url.path()));
        app.http.get(checksum_url.clone()).send()
            .context(info_span!("fetching Maven artifact SHA1 checksum", url = %checksum_url, coordinate = %artifact.coordinate))
            .await?
    } else {
        response
    };

    let checksum = async {
        response
            .error_for_status_ext()
            .await?
            .text()
            .await
            .map_err(anyhow::Error::from)
    }
    .context(info_span!("reading Maven artifact checksum", url = %checksum_url))
    .await?;
    let (expected_sha256, expected_sha1) = if use_sha1 {
        let mut digest = [0; 20];
        hex::decode_to_slice(checksum.trim(), &mut digest).with_context(
            || format!("invalid Maven artifact SHA1 at {checksum_url}"),
        )?;
        (None, Some(Sha1(digest)))
    } else {
        let mut digest = [0; 32];
        hex::decode_to_slice(checksum.trim(), &mut digest).with_context(
            || format!("invalid Maven artifact SHA256 at {checksum_url}"),
        )?;
        (Some(Sha256(digest)), None)
    };

    let mut conn = app
        .db
        .connection()
        .context(info_span!("acquiring connection"))
        .await?;
    let query = model::BlobHash::all();
    let query = if let Some(sha256) = expected_sha256 {
        query.filter(model::BlobHash::fields().sha256().eq(sha256))
    } else {
        query
            .filter(model::BlobHash::fields().sha1().eq(expected_sha1.unwrap()))
    };
    let existing = query
        .exec(&mut conn)
        .context(info_span!("checking Maven artifact CAS hash"))
        .await?;
    drop(conn);
    let expected_sha256 =
        expected_sha256.or_else(|| existing.first().map(|blob| blob.sha256));
    if let Some(existing) = existing.first() {
        match app.cas.get(existing.sha256).await {
            Ok(_) => {
                return Ok(FabriclikeStoredArtifact {
                    artifact,
                    sha256: existing.sha256,
                });
            }
            Err(error) => {
                warn!(%error, sha256 = %existing.sha256, coordinate = %artifact.coordinate,
					"cached Maven artifact unavailable; restoring from upstream");
            }
        }
    }

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
    .context(info_span!("fetching Maven artifact", %url, coordinate = %artifact.coordinate,
        source = artifact.source.as_deref().unwrap_or("unknown"),
        repository = %artifact.repository, repository_source = artifact.repository_source,
        scheme = url.scheme()))
    .await?;

    if let Some(expected) = expected_sha256 {
        ensure!(
            Sha256::from_digest(&bytes) == expected,
            "Maven artifact SHA256 mismatch for {url}: expected {expected}, got {}",
            Sha256::from_digest(&bytes)
        );
    }
    if let Some(expected) = expected_sha1 {
        ensure!(
            Sha1::from_digest(&bytes) == expected,
            "Maven artifact SHA1 mismatch for {url}: expected {expected}, got {}",
            Sha1::from_digest(&bytes)
        );
    }
    let blob = app.cas
		.store(&bytes)
		.context(info_span!("storing Maven artifact in CAS", coordinate = %artifact.coordinate))
		.await?;
    let mut conn = app
        .db
        .connection()
        .context(info_span!("acquiring artifact connection"))
        .await?;
    blob.register(&mut conn).await?;
    let sha256 = blob.sha256;
    Ok(FabriclikeStoredArtifact { artifact, sha256 })
}

fn validate_groups(
    games: &IndexMap<String, CatalogGame>,
    groups: &[ProfileGroup],
) -> Result<()> {
    let mut assigned = std::collections::HashSet::new();
    let mut ids = std::collections::HashSet::new();
    for group in groups {
        ensure!(
            ids.insert(&group.id),
            "duplicate profile group {:?}",
            group.id
        );
        for game in &group.games {
            ensure!(
                games.contains_key(game),
                "profile group references unknown game {game}"
            );
            ensure!(
                assigned.insert(game),
                "game version {game} belongs to multiple profile groups"
            );
        }
    }
    for game in games.keys() {
        ensure!(
            assigned.contains(game),
            "game version {game} has no profile group"
        );
    }
    Ok(())
}

/// Takes a downloaded loader profile, replaces its game-specific metadata with
/// placeholders, retaining source repositories for later publication.
///
/// Returns the processed profile and the concrete artifacts needed by its games.
/// Callers should dedup artifacts by Maven path before mirroring them.
pub fn normalize_profile(
    mut profile: ProfileMetadata,
    template: &str,
    games: &[String],
    fallback_maven: &Url,
) -> Result<(ProfileMetadata, Vec<Artifact>)> {
    if let Some(prefix) = profile.id.strip_suffix(template) {
        profile.id = format!("{prefix}{GAME_VERSION_PLACEHOLDER}");
    }
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
                    repository_source: repository_source.to_owned(),
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
                repository_source: repository_source.to_owned(),
            });
        }
    }
    Ok((profile, artifacts))
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

struct SelectedProfile {
    loader_version: String,
    version_group: String,
    stable: bool,
    source_url: String,
    source_sha256: Sha256,
    template: String,
    metadata: ProfileMetadata,
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

fn select_profile<'a>(
    loader: FabriclikeLoader,
    group: &'a ProfileGroup,
    loader_version: &str,
    hashes: &HashMap<String, Sha256>,
    required: bool,
) -> Result<Option<(&'a str, Url, Sha256)>> {
    for template in
        std::iter::once(&group.template).chain(&group.fallback_templates)
    {
        let source = profile_source(group, template, loader_version)?;
        if let Some(sha256) = hashes.get(source.as_str()) {
            return Ok(Some((template.as_str(), source, *sha256)));
        }
    }
    ensure!(
        !required,
        "no downloaded {loader:?} profile for {loader_version} in group {:?}",
        group.id,
    );
    Ok(None)
}

#[cfg(test)]
mod tests {
    use std::slice;

    use super::*;
    use serde_json::{Value, json};

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

    #[test]
    fn source_urls_encode_game_and_loader_versions() {
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

    #[test]
    fn selects_current_template_before_historical_profiles() {
        let mut group = group(Some("v2"), "26.3", &["26.3", "26.2"]);
        group.fallback_templates = vec!["26.2".into()];
        let current = profile_source(&group, "26.3", "0.30.0").unwrap();
        let historical = profile_source(&group, "26.2", "0.30.0").unwrap();
        let current_hash = Sha256::from_digest(b"current");
        let historical_hash = Sha256::from_digest(b"historical");
        let mut hashes =
            HashMap::from([(historical.to_string(), historical_hash)]);
        let selected = select_profile(
            FabriclikeLoader::Quilt,
            &group,
            "0.30.0",
            &hashes,
            true,
        )
        .unwrap()
        .unwrap();
        assert_eq!(selected, ("26.2", historical, historical_hash));
        hashes.insert(current.to_string(), current_hash);
        let selected = select_profile(
            FabriclikeLoader::Quilt,
            &group,
            "0.30.0",
            &hashes,
            true,
        )
        .unwrap()
        .unwrap();
        assert_eq!(selected, ("26.3", current, current_hash));
    }

    #[test]
    fn missing_profiles_are_optional_only_for_historical_loaders() {
        let legacy = group(Some("v1"), "1.21", &[]);
        let modern = group(Some("v2"), "26.3", &[]);
        let sha256 = Sha256::from_digest(b"historical profile");
        let hashes = HashMap::from([(
            profile_source(&legacy, "1.21", "old").unwrap().to_string(),
            sha256,
        )]);
        assert!(
            select_profile(
                FabriclikeLoader::Quilt,
                &legacy,
                "old",
                &hashes,
                false
            )
            .unwrap()
            .is_some()
        );
        assert!(
            select_profile(
                FabriclikeLoader::Quilt,
                &modern,
                "old",
                &hashes,
                false
            )
            .unwrap()
            .is_none()
        );
        assert!(
            select_profile(
                FabriclikeLoader::Quilt,
                &modern,
                "old",
                &hashes,
                true
            )
            .is_err()
        );
        let mut invalid = modern;
        invalid.profile_base_url =
            Url::parse("mailto:invalid@example.com").unwrap();
        assert!(
            select_profile(
                FabriclikeLoader::Quilt,
                &invalid,
                "old",
                &hashes,
                false
            )
            .is_err()
        );
    }

    #[test]
    fn groups_cover_each_catalog_game_exactly_once() {
        let games = ["1.21", "26.3"]
            .into_iter()
            .map(|id| {
                (
                    id.to_owned(),
                    CatalogGame {
                        id: id.into(),
                        stable: true,
                    },
                )
            })
            .collect();
        let legacy = group(Some("v1"), "1.21", &["1.21"]);
        let modern = group(Some("v2"), "26.3", &["26.3"]);
        assert!(
            validate_groups(&games, &[legacy.clone(), modern.clone()]).is_ok()
        );
        assert!(validate_groups(&games, slice::from_ref(&legacy)).is_err());
        assert!(validate_groups(&games, &[legacy.clone(), legacy]).is_err());
        assert!(
            validate_groups(
                &games,
                &[group(None, "1.21", &["1.21", "26.3"]), modern]
            )
            .is_err()
        );
        assert!(
            validate_groups(
                &games,
                &[group(None, "1.21", &["1.21", "26.3", "unknown"])]
            )
            .is_err()
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
        )?;
        Ok((serde_json::to_value(profile)?, artifacts))
    }

    #[test]
    fn profile_id_substitution_preserves_loader_version() {
        let profile: ProfileMetadata = serde_json::from_value(json!({
            "id": "quilt-loader-0.26.3-26.3",
            "inheritsFrom": "26.3",
            "releaseTime": "2026-01-01T00:00:00Z",
            "time": "2026-01-01T00:00:00Z",
            "type": "release",
            "libraries": []
        }))
        .unwrap();
        let (profile, _) = normalize_profile(
            profile,
            "26.3",
            &[],
            &Url::parse("https://maven.quiltmc.org/repository/release/")
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            profile.id,
            format!("quilt-loader-0.26.3-{GAME_VERSION_PLACEHOLDER}")
        );
    }

    #[test]
    fn substitutes_templates_and_expands_concrete_artifacts() {
        let (profile, artifacts) = normalize(
            json!({
                "id": "fabric-loader-GAME",
                "inheritsFrom": "GAME",
                "libraries": [{"name": "net.fabricmc:intermediary:GAME"}]
            }),
            &["1.21.1".to_owned(), "1.14.2 Pre-Release 4".to_owned()],
        )
        .unwrap();
        assert_eq!(
            profile["id"],
            format!("fabric-loader-{GAME_VERSION_PLACEHOLDER}")
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
            "https://maven.fabricmc.net/"
        );
    }

    #[test]
    fn preserves_concrete_libraries_and_emits_minecraft_meta_defaults() {
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
        assert!(output["libraries"][0].get("downloads").is_none());
        let mut missing = value;
        missing.as_object_mut().unwrap().remove("releaseTime");
        assert!(serde_json::from_value::<ProfileMetadata>(missing).is_err());
    }

    #[test]
    fn artifact_urls_upgrade_http_repositories() {
        let artifact = Artifact {
            coordinate: "com.github.zafarkhaja:java-semver:0.9.0"
                .parse()
                .unwrap(),
            repository: "http://repo.maven.apache.org/maven2/".parse().unwrap(),
            source: Some("downloaded profile".into()),
            repository_source: "profile library.url".into(),
        };
        assert_eq!(
            artifact_url(&artifact).unwrap().as_str(),
            "https://repo.maven.apache.org/maven2/com/github/zafarkhaja/java-semver/0.9.0/java-semver-0.9.0.jar"
        );
    }

    #[test]
    fn artifact_urls_append_encoded_segments_without_store_prefix() {
        let artifact = Artifact {
            coordinate: "net.fabricmc:intermediary:1.14.2 Pre-Release 4"
                .parse()
                .unwrap(),
            repository: "https://example.com/repository/release"
                .parse()
                .unwrap(),
            source: None,
            repository_source: "test".into(),
        };
        assert_eq!(
            artifact_url(&artifact).unwrap().as_str(),
            "https://example.com/repository/release/net/fabricmc/intermediary/1.14.2%20Pre-Release%204/intermediary-1.14.2%20Pre-Release%204.jar"
        );
    }

    #[test]
    fn records_profile_repository_selection() {
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
            artifacts[0].repository.as_str(),
            "http://repo.maven.apache.org/maven2/"
        );
    }
}
