use std::{
    collections::{BTreeMap, HashSet},
    sync::atomic::{AtomicUsize, Ordering},
};

use anyhow::{Context, Result, ensure};
use futures::{TryStreamExt, stream};
use minecraft_meta_manifest::forgelike::{
    GameVersion, LoaderVersion, Manifest,
};
use minecraft_meta_manifest::profile::ProfileMetadata;
use tracing::{info, info_span};
use tracing_anyhow::FutureContext;
use url::Url;

use crate::{
    AppState,
    model::{
        ForgelikeExtract, ForgelikeInstaller, ForgelikeLoader,
        ForgelikeStoredArtifact,
    },
    store::{ContentType, PutStatus},
    upstream::Upstreams,
    util::{MavenCoordinate, TaskContext},
};

pub const FORMAT_VERSION: u32 = 0;

/// Publishes stored dependencies, version profiles, and Forge-like catalogs.
pub async fn export(
    app: &AppState,
    maven: &super::maven::MavenExports<'_>,
    upstreams: Upstreams,
) -> Result<PutStatus> {
    let mut loaders = Vec::new();
    if upstreams.forge {
        loaders.push(ForgelikeLoader::Forge);
    }
    if upstreams.neoforge {
        loaders.push(ForgelikeLoader::Neoforge);
    }
    if loaders.is_empty() {
        return Ok(PutStatus::Unchanged);
    }
    let mut conn = app
        .db
        .connection()
        .context(info_span!("acquiring Forge-like export connection"))
        .await?;
    let mut txn = conn
        .transaction()
        .context(info_span!("starting Forge-like export snapshot"))
        .await?;
    toasty::sql::statement(
        "SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY",
    )
    .exec(&mut txn)
    .context(info_span!("setting Forge-like export snapshot isolation"))
    .await?;
    let fields = ForgelikeInstaller::fields();
    let mut installers = ForgelikeInstaller::all()
        .filter(fields.loader().in_list(loaders.clone()))
        .exec(&mut txn)
        .context(info_span!("fetching selected Forge-like installers"))
        .await?;
    installers.retain(|installer| !installer.is_blacklisted());
    for loader in &loaders {
        ensure!(
            installers
                .iter()
                .any(|installer| installer.loader == *loader),
            "no imported {loader:?} installers; run import before exporting"
        );
    }
    let installer_hashes = installers
        .iter()
        .map(|installer| installer.sha256)
        .collect::<HashSet<_>>();
    let extracts = ForgelikeExtract::all()
        .filter(
            ForgelikeExtract::fields()
                .installer_sha256()
                .in_list(installer_hashes.iter().copied().collect::<Vec<_>>()),
        )
        .exec(&mut txn)
        .context(info_span!("fetching selected Forge-like extracts"))
        .await?;
    let extracted_hashes = extracts
        .iter()
        .map(|extract| extract.installer_sha256)
        .collect::<HashSet<_>>();
    let missing = installer_hashes.difference(&extracted_hashes).count();
    ensure!(
        missing == 0,
        "{missing} Forge-like installers have no extracted metadata; run import before exporting"
    );
    txn.commit()
        .context(info_span!("finishing Forge-like export snapshot"))
        .await?;
    drop(conn);

    let artifacts = deduplicate_artifacts(
        extracts
            .iter()
            .flat_map(|extract| extract.embedded_maven_artifacts.0.clone()),
    )?;
    let extracts = extracts
        .into_iter()
        .map(|extract| (extract.installer_sha256, extract))
        .collect::<BTreeMap<_, _>>();

    installers.sort_by(|a, b| a.name.cmp(&b.name));
    // prepare profiles and catalogs before publishing any files
    let mut profiles = BTreeMap::new();
    let mut catalogs = Vec::new();
    for loader in loaders {
        let mut games = BTreeMap::<String, Vec<LoaderVersion>>::new();
        for installer in installers
            .iter()
            .filter(|installer| installer.loader == loader)
        {
            let extract = &extracts[&installer.sha256];
            let version = loader_version(loader, installer.name.as_str());
            let path = profile_path(loader, version);
            let profile = prepare_profile(
                extract.metadata.0.clone(),
                &extract.embedded_maven_artifacts.0,
                |path| app.public_blobs.url_for(path),
            )?;
            ensure!(
                !profile.inherits_from.is_empty(),
                "installer {} has no game association",
                installer.name
            );
            let json = serde_json::to_vec(&profile)
                .context("serializing Forge-like profile")?;
            if let Some(existing) = profiles.get(&path) {
                ensure!(
                    existing == &json,
                    "conflicting Forge-like profiles at {path}"
                );
            } else {
                profiles.insert(path.clone(), json);
            }
            let entries = games.entry(profile.inherits_from).or_default();
            if !entries.iter().any(|entry| entry.id == version) {
                entries.push(LoaderVersion {
                    id: version.to_owned(),
                    url: app.public_blobs.url_for(&path),
                    stable: false,
                });
            }
        }
        let manifest = make_manifest(games);
        catalogs.push((
            format!("{}/v{FORMAT_VERSION}/manifest.json", loader_path(loader)),
            serde_json::to_vec(&manifest)
                .context("serializing Forge-like catalog")?,
        ));
    }
    // publish dependencies before the profiles which reference them
    let num_total = artifacts.len();
    let num_done = AtomicUsize::new(0);
    info!(
        num_artifacts = num_total,
        "publishing Forge-like embedded artifacts"
    );
    let num_done = &num_done;
    let mut status = stream::iter(
        artifacts.into_iter().map(Ok::<_, anyhow::Error>),
    )
    .map_ok(|artifact| async move {
        let changed = if maven
            .is_published(&artifact.coordinate, artifact.sha256)
            .await
        {
            PutStatus::Unchanged
        } else {
            maven
                .publish(app, &artifact.coordinate, artifact.sha256)
                .transaction_context(|| {
                    info_span!(
                        "publishing Forge-like artifact",
                        coordinate = %artifact.coordinate,
                        sha256 = %artifact.sha256,
                    )
                })
                .await?
        };
        let done = num_done.fetch_add(1, Ordering::Relaxed) + 1;
        if done.is_multiple_of(100) || done == num_total {
            info!("processed {done}/{num_total} Forge-like embedded artifacts");
        }
        anyhow::Ok(changed)
    })
    .try_buffer_unordered(app.concurrency.download.get())
    .try_fold(
        PutStatus::Unchanged,
        |mut status, artifact_status| async move {
            super::latch(&mut status, artifact_status);
            Ok(status)
        },
    )
    .await?;
    info!(
        num_artifacts = num_total,
        "finished publishing Forge-like embedded artifacts"
    );

    // publish profiles before the catalogs which reference them
    super::latch(&mut status, super::publish_json_files(app, profiles).await?);
    for (path, json) in catalogs {
        let changed = app
            .public_blobs
            .put(&path, &json, ContentType::Json)
            .context(info_span!("writing Forge-like catalog", %path))
            .await?;
        super::latch(&mut status, changed);
    }
    Ok(status)
}

fn loader_path(loader: ForgelikeLoader) -> &'static str {
    match loader {
        ForgelikeLoader::Forge => "forge",
        ForgelikeLoader::Neoforge => "neo",
    }
}

fn loader_version(loader: ForgelikeLoader, name: &str) -> &str {
    match loader {
        ForgelikeLoader::Forge => name.split('-').nth(1).unwrap_or(name),
        ForgelikeLoader::Neoforge => {
            let mut parts = name.split('-');
            let prefix = parts.next().unwrap_or(name);
            match parts.next() {
                Some(version)
                    if prefix.contains('.')
                        && prefix.bytes().all(|byte| {
                            byte.is_ascii_digit() || byte == b'.'
                        })
                        && version.contains('.')
                        && version.bytes().all(|byte| {
                            byte.is_ascii_digit() || byte == b'.'
                        }) =>
                {
                    version
                }
                _ => name,
            }
        }
    }
}

fn compare_version_components(left: &str, right: &str) -> std::cmp::Ordering {
    let mut left = left.split('.').map(|part| part.trim_start_matches('0'));
    let mut right = right.split('.').map(|part| part.trim_start_matches('0'));
    loop {
        match (left.next(), right.next()) {
            (Some(left), Some(right)) => {
                let order =
                    left.len().cmp(&right.len()).then_with(|| left.cmp(right));
                if !order.is_eq() {
                    return order;
                }
            }
            (None, None) => return std::cmp::Ordering::Equal,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (None, Some(_)) => return std::cmp::Ordering::Less,
        }
    }
}

fn compare_loader_versions(left: &str, right: &str) -> std::cmp::Ordering {
    let (left_version, left_suffix) = left
        .split_once('-')
        .map_or((left, None), |(version, suffix)| (version, Some(suffix)));
    let (right_version, right_suffix) = right
        .split_once('-')
        .map_or((right, None), |(version, suffix)| (version, Some(suffix)));
    compare_version_components(left_version, right_version)
        .then_with(|| match (left_suffix, right_suffix) {
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(_), None) => std::cmp::Ordering::Less,
            (Some(left), Some(right)) => left.cmp(right),
            (None, None) => std::cmp::Ordering::Equal,
        })
        .then_with(|| left.cmp(right))
}

fn profile_path(loader: ForgelikeLoader, version: &str) -> String {
    format!(
        "{}/v{FORMAT_VERSION}/versions/{version}.json",
        loader_path(loader)
    )
}

fn prepare_profile(
    mut profile: ProfileMetadata,
    artifacts: &[ForgelikeStoredArtifact],
    url_for: impl Fn(&str) -> Url,
) -> Result<ProfileMetadata> {
    let paths = artifacts
        .iter()
        .map(|artifact| artifact.coordinate.to_maven_path())
        .collect::<HashSet<_>>();
    for library in &mut profile.libraries {
        let coordinate = library.name.parse::<MavenCoordinate>()?;
        let path = coordinate.to_maven_path();
        if paths.contains(&path) {
            library.url = Some(url_for("maven/").to_string());
            library.downloadable = true;
            if let Some(artifact) = library
                .downloads
                .as_mut()
                .and_then(|downloads| downloads.artifact.as_mut())
            {
                artifact.url = Some(url_for(&path));
            }
        }
    }
    Ok(profile)
}

fn make_manifest(games: BTreeMap<String, Vec<LoaderVersion>>) -> Manifest {
    Manifest {
        game_versions: games
            .into_iter()
            .map(|(id, mut loaders)| {
                // latest and the stable fallback both select the first loader
                loaders.sort_by(|left, right| {
                    compare_loader_versions(&right.id, &left.id)
                });
                GameVersion {
                    id,
                    stable: true,
                    loaders,
                }
            })
            .collect(),
    }
}

fn deduplicate_artifacts(
    artifacts: impl IntoIterator<Item = ForgelikeStoredArtifact>,
) -> Result<Vec<ForgelikeStoredArtifact>> {
    let mut paths = BTreeMap::<String, ForgelikeStoredArtifact>::new();
    for artifact in artifacts {
        let path = artifact.coordinate.to_maven_path();
        if let Some(existing) = paths.get(&path) {
            ensure!(
                existing.sha256 == artifact.sha256,
                "conflicting embedded Maven artifacts at {path}: {} and {}",
                existing.sha256,
                artifact.sha256
            );
        } else {
            paths.insert(path, artifact);
        }
    }
    Ok(paths.into_values().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::Sha256;

    fn artifact(coordinate: &str, bytes: &[u8]) -> ForgelikeStoredArtifact {
        ForgelikeStoredArtifact {
            coordinate: coordinate.parse().unwrap(),
            sha256: Sha256::from_digest(bytes),
        }
    }

    #[test]
    fn version_names_and_paths_match_consumers() {
        for (raw, expected) in [
            ("1.9-12.16.0.1880-1.9", "12.16.0.1880"),
            ("1.10.2-12.18.1.2016-failtests", "12.18.1.2016"),
            ("26.1-64.0.10", "64.0.10"),
            ("26.1.1-64.0.10-26.1.1", "64.0.10"),
        ] {
            assert_eq!(loader_version(ForgelikeLoader::Forge, raw), expected);
        }
        for (raw, expected) in [
            ("1.20.1-47.1.74", "47.1.74"),
            ("47.1.83", "47.1.83"),
            ("20.2.29-beta", "20.2.29-beta"),
            ("26.1.0.10-beta", "26.1.0.10-beta"),
            ("26.1.1.10", "26.1.1.10"),
        ] {
            assert_eq!(
                loader_version(ForgelikeLoader::Neoforge, raw),
                expected
            );
        }
        assert_eq!(
            profile_path(ForgelikeLoader::Forge, "49.0.1"),
            "forge/v0/versions/49.0.1.json"
        );
        assert_eq!(
            profile_path(ForgelikeLoader::Neoforge, "26.1.0.10-beta"),
            "neo/v0/versions/26.1.0.10-beta.json"
        );
        assert_eq!(
            profile_path(
                ForgelikeLoader::Neoforge,
                "26.1.0.0-alpha.1+snapshot-1"
            ),
            "neo/v0/versions/26.1.0.0-alpha.1+snapshot-1.json"
        );
    }

    #[test]
    fn catalog_serialization_preserves_game_order_and_sorts_loaders_latest_first()
     {
        let entry = |id: &str| LoaderVersion {
            id: id.into(),
            stable: false,
            url: format!("https://meta.example/forge/v0/versions/{id}.json")
                .parse()
                .unwrap(),
        };
        let manifest = make_manifest(BTreeMap::from([
            ("1.9".into(), vec![entry("12.1"), entry("12.2")]),
            ("1.10.2".into(), vec![entry("13.1")]),
        ]));
        let value = serde_json::to_value(manifest).unwrap();
        assert_eq!(value["gameVersions"][0]["id"], "1.10.2");
        assert_eq!(value["gameVersions"][1]["loaders"][0]["id"], "12.2");
        assert_eq!(value["gameVersions"][1]["stable"], true);
        assert_eq!(value["gameVersions"][1]["loaders"][0]["stable"], false);
        assert!(value.get("versionGroups").is_none());
        assert!(value["gameVersions"][0].get("versionGroup").is_none());
    }

    #[test]
    fn numeric_order_selects_latest_for_forge_and_neoforge() {
        for (game, versions, expected) in [
            (
                "1.20.1",
                vec!["47.1.9", "47.1.100", "47.1.10"],
                vec!["47.1.100", "47.1.10", "47.1.9"],
            ),
            (
                "1.20.6",
                vec!["20.6.9", "20.6.119", "20.6.119-beta", "20.6.120-beta"],
                vec!["20.6.120-beta", "20.6.119", "20.6.119-beta", "20.6.9"],
            ),
            (
                "26.1",
                vec!["26.1.0.9-beta", "26.1.0.10-beta", "26.1.0.10"],
                vec!["26.1.0.10", "26.1.0.10-beta", "26.1.0.9-beta"],
            ),
        ] {
            let entries = versions
                .into_iter()
                .map(|id| LoaderVersion {
                    id: id.into(),
                    stable: false,
                    url: "https://meta.example/profile.json".parse().unwrap(),
                })
                .collect();
            let manifest =
                make_manifest(BTreeMap::from([(game.into(), entries)]));
            let loaders = &manifest.game_versions[0].loaders;
            assert_eq!(
                loaders
                    .iter()
                    .map(|loader| loader.id.as_str())
                    .collect::<Vec<_>>(),
                expected
            );
        }
        assert!(
            compare_loader_versions(
                "1.999999999999999999999999999999",
                "1.1000000000000000000000000000000"
            )
            .is_lt()
        );
    }

    #[test]
    fn rewrites_only_stored_dependencies_and_preserves_install_metadata() {
        let profile: ProfileMetadata = serde_json::from_value(serde_json::json!({
            "id": "neo", "inheritsFrom": "26.1", "releaseTime": "2026-01-01T00:00:00Z", "time": "2026-01-01T00:00:00Z", "type": "release",
            "libraries": [
                {"name": "g:a:1@jar", "include_in_classpath": false, "downloads": {"artifact": {"url": "https://upstream.example/a.jar", "sha1": "0000000000000000000000000000000000000000", "size": 12}}},
                {"name": "g:b:1", "downloadable": false, "downloads": {"artifact": {"url": "", "sha1": "0000000000000000000000000000000000000000", "size": 0}}},
                {"name": "g:c:1", "url": "https://libraries.minecraft.net/"}
            ],
            "data": {"BINPATCH": {"client": "/data/client.lzma", "server": "/data/server.lzma"}},
            "processors": [{"jar": "g:a:1", "classpath": [], "args": ["{BINPATCH}"], "sides": ["client"]}]
        })).unwrap();
        let original = serde_json::to_value(&profile).unwrap();
        for base in ["https://meta.example/", "https://meta.example/prefix/"] {
            let base = Url::parse(base).unwrap();
            let published = prepare_profile(
                profile.clone(),
                &[artifact("g:a:1", b"a")],
                |path| base.join(path).unwrap(),
            )
            .unwrap();
            let mut expected = original.clone();
            expected["libraries"][0]["url"] =
                serde_json::json!(base.join("maven/").unwrap());
            expected["libraries"][0]["downloads"]["artifact"]["url"] =
                serde_json::json!(base.join("maven/g/a/1/a-1.jar").unwrap());
            assert_eq!(serde_json::to_value(published).unwrap(), expected);
        }
    }

    #[test]
    fn deduplicates_coordinate_paths_including_default_extension() {
        let artifacts = deduplicate_artifacts([
            artifact("net.example:library:1", b"same"),
            artifact("net.example:library:1@jar", b"same"),
        ])
        .unwrap();
        assert_eq!(artifacts.len(), 1);
    }

    #[test]
    fn rejects_conflicting_content_before_publication() {
        assert!(
            deduplicate_artifacts([
                artifact("net.example:library:1", b"first"),
                artifact("net.example:library:1", b"second"),
            ])
            .is_err()
        );
    }
}
