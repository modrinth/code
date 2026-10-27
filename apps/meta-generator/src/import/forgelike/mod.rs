use std::{
    any::type_name,
    collections::HashSet,
    io::{Cursor, Read},
    sync::Arc,
};

use anyhow::{Context, Result, anyhow, ensure};
use futures::{StreamExt, TryStreamExt, stream};
use indexmap::IndexMap;
use jiff::Timestamp;
use serde::{Deserialize, de::DeserializeOwned};
use tracing::{debug, info, info_span, warn};
use tracing_anyhow::FutureContext;

use crate::{
    AppState, model,
    upstream::{self, mojang},
    util::{
        MavenCoordinate, ResponseExt, Sha1, TaskContext, from_json_slice,
        from_json_value,
    },
};

mod data;

pub async fn extract_installers(
    app: &AppState,
    upstreams: upstream::Upstreams,
) -> Result<()> {
    let mut loaders = Vec::new();
    if upstreams.forge {
        loaders.push(model::ForgelikeLoader::Forge);
    }
    if upstreams.neoforge {
        loaders.push(model::ForgelikeLoader::Neoforge);
    }
    if loaders.is_empty() {
        return Ok(());
    }
    let mut conn = app
        .db
        .connection()
        .context(info_span!("acquiring extraction connection"))
        .await?;
    let extracted_hashes = model::ForgelikeExtract::all()
        .select(model::ForgelikeExtract::fields().installer_sha256());
    let mut unprocessed_installers = model::ForgelikeInstaller::all()
        .filter(
            model::ForgelikeInstaller::fields()
                .sha256()
                .in_query(extracted_hashes)
                .not(),
        )
        .filter(
            model::ForgelikeInstaller::fields()
                .loader()
                .in_list(loaders),
        )
        .exec(&mut conn)
        .context(info_span!("fetching unprocessed Forge-like installers"))
        .await?;
    unprocessed_installers.retain(|installer| !installer.is_blacklisted());
    info!(
        "found {} installers left to process",
        unprocessed_installers.len()
    );

    drop(conn);
    let mut num_done = 0usize;
    let mut num_failed = 0usize;
    let mut seen = HashSet::new();
    for installer in unprocessed_installers {
        if !seen.insert(installer.sha256) {
            continue;
        }
        let name = installer.name.clone();
        let sha256 = installer.sha256;
        if let Err(err) = extract_from_installer(app, installer)
            .transaction_context(
                || info_span!("extracting from installer", %name, %sha256),
            )
            .await
        {
            num_failed += 1;
            warn!(%name, %sha256, "installer extraction failed: {err:?}");
        }

        num_done += 1;
        if num_done.is_multiple_of(10) {
            info!("processed {num_done} installers");
        }
    }

    info!(
        num_done,
        num_failed, "finished Forge-like installer extraction"
    );
    ensure!(
        num_failed == 0,
        "failed to extract {num_failed} Forge-like installers"
    );
    Ok(())
}

#[derive(Debug, Deserialize)]
struct ModernInstallProfile {
    data: IndexMap<String, model::SidedDataEntry>,
    libraries: Vec<mojang::Library>,
    processors: Vec<model::Processor>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyInstallProfile {
    install: LegacyInstall,
    version_info: LegacyVersionInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyInstall {
    minecraft: String,
    path: MavenCoordinate,
    file_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyVersionInfo {
    id: String,
    #[serde(deserialize_with = "crate::util::deserialize_timestamp")]
    release_time: Timestamp,
    #[serde(deserialize_with = "crate::util::deserialize_timestamp")]
    time: Timestamp,
    main_class: Option<String>,
    minecraft_arguments: Option<String>,
    libraries: Vec<mojang::Library>,
    #[serde(rename = "type")]
    ty: mojang::VersionType,
}

impl LegacyInstallProfile {
    fn into_metadata(self) -> model::ProfileMetadata {
        let version = self.version_info;
        let arguments = version.minecraft_arguments.as_ref().map(|args| {
            IndexMap::from([(
                mojang::ArgumentType::Game,
                args.split(' ')
                    .map(|arg| mojang::Argument::Normal(arg.to_owned()))
                    .collect(),
            )])
        });

        model::ProfileMetadata {
            id: version.id,
            inherits_from: self.install.minecraft,
            release_time: version.release_time,
            time: version.time,
            main_class: version.main_class,
            minecraft_arguments: version.minecraft_arguments,
            arguments,
            libraries: version.libraries,
            ty: version.ty,
            data: None,
            processors: None,
        }
    }
}

type ZipFileReader = zip::ZipArchive<Cursor<Arc<[u8]>>>;

async fn extract_from_installer(
    app: &AppState,
    installer: model::ForgelikeInstaller,
) -> Result<()> {
    let installer_blob = app
        .cas
        .get(installer.sha256)
        .context(info_span!("fetching installer blob"))
        .await?;
    let zip = tokio::task::spawn_blocking(move || {
        ZipFileReader::new(Cursor::new(Arc::from(installer_blob)))
    })
    .await
    .context("joining installer ZIP reader")?
    .context("opening installer zip for reading")?;

    let install_profile =
        read_json::<serde_json::Value>(&zip, "install_profile.json")
            .context(info_span!("reading install profile"))
            .await?
            .context("installer is missing install_profile.json")?;
    let mut legacy_artifact = None;
    let mut version =
        match read_json::<model::ProfileMetadata>(&zip, "version.json")
            .context(info_span!("reading installer version"))
            .await?
        {
            Some(mut version) => {
                let profile =
                    from_json_value::<ModernInstallProfile>(&install_profile)
                        .context("parsing modern install profile")?;
                version.data = Some(profile.data);
                version.processors = Some(profile.processors);
                version.libraries.extend(profile.libraries.into_iter().map(
                    |mut library| {
                        library.include_in_classpath = false;
                        library
                    },
                ));
                version
            }
            None => {
                let profile =
                    from_json_value::<LegacyInstallProfile>(&install_profile)
                        .context(
                        "parsing legacy install profile without version.json",
                    )?;
                legacy_artifact = Some((
                    profile.install.path.clone(),
                    profile.install.file_path.clone(),
                ));
                profile.into_metadata()
            }
        };
    let modern = legacy_artifact.is_none();
    let entry_names = zip.file_names().collect::<Result<Vec<_>, _>>()?;
    let embedded_entries = entry_names
        .iter()
        .map(|name| name.as_ref())
        .collect::<HashSet<_>>();
    let data_artifacts = data::normalize(
        &mut version,
        installer.loader,
        &installer.name,
        &embedded_entries,
    )?;
    let referenced_artifacts = referenced_maven_artifacts(&version.libraries)?;
    let mut embedded_maven_artifacts = {
        referenced_artifacts
            .into_iter()
            .filter(|coordinate| {
                embedded_entries.contains(coordinate.to_maven_path().as_str())
            })
            .map(|coordinate| {
                let path = coordinate.to_maven_path();
                (coordinate, path)
            })
            .collect::<Vec<_>>()
    };
    if let Some((coordinate, path)) = legacy_artifact {
        embedded_maven_artifacts.retain(|(existing, _)| {
            existing.to_maven_path() != coordinate.to_maven_path()
        });
        embedded_maven_artifacts.push((coordinate, path));
    }

    embedded_maven_artifacts.extend(data_artifacts);

    debug!(
        %version.id,
        num_embedded_maven_artifacts = embedded_maven_artifacts.len(),
        "read installer metadata"
    );

    let embedded_paths = embedded_maven_artifacts
        .iter()
        .map(|(coordinate, _)| coordinate.to_maven_path())
        .collect::<HashSet<_>>();
    let mut external_artifacts = Vec::new();
    for library in &mut version.libraries {
        let coordinate: MavenCoordinate = library.name.parse()?;
        let source =
            classify_library(library, &coordinate, &embedded_paths, modern)?;
        normalize_library(library, &source);
        if let LibrarySource::External { urls, hashes } = source {
            external_artifacts.push((coordinate, urls, hashes));
        }
    }
    if modern {
        embedded_maven_artifacts.retain(|(coordinate, _)| {
            !external_artifacts.iter().any(|(external, _, _)| {
                external.to_maven_path() == coordinate.to_maven_path()
            })
        });
    }

    let mut stored_artifacts =
        stream::iter(embedded_maven_artifacts.into_iter().enumerate())
            .map(|(index, (coordinate, path))| {
                let zip = &zip;
				let artifact_path = path.clone();
                async move {
                    let bytes = read_file(zip, &path)
                        .context(info_span!(
                            "extracting embedded Maven artifact",
                            path
                        ))
                        .await?
                        .context(
                            "embedded Maven artifact is missing from installer",
                        )?;
					let blob = app.cas
						.store(&bytes)
						.context(info_span!("storing embedded Maven artifact in CAS", path, %coordinate))
						.await?;
					let mut conn = app.db.connection()
						.context(info_span!("acquiring artifact connection"))
						.await?;
					blob.register(&mut conn).await?;
					let sha256 = blob.sha256;
                    anyhow::Ok((
                        index,
                        model::ForgelikeStoredArtifact { coordinate, sha256 },
                    ))
                }
				.transaction_context(move || info_span!("extracting installer Maven artifact", path = %artifact_path))
            })
            .buffer_unordered(app.concurrency.extract_files.get())
            .try_collect::<Vec<_>>()
            .await?;
    stored_artifacts.sort_unstable_by_key(|(index, _)| *index);
    let mut stored_artifacts = stored_artifacts
        .into_iter()
        .map(|(_, artifact)| artifact)
        .collect::<Vec<_>>();

    let fallback = match installer.loader {
        model::ForgelikeLoader::Forge => "https://maven.minecraftforge.net/",
        model::ForgelikeLoader::Neoforge => {
            "https://maven.neoforged.net/releases/"
        }
    };
    let mut downloaded =
        stream::iter(external_artifacts.into_iter().enumerate())
            .map(|(index, (coordinate, mut urls, hashes))| async move {
                if urls.len() > 1 {
                    urls.push(repository_artifact_url(fallback, &coordinate)?);
                }
				let artifact_name = coordinate.to_string();
                download_library(app, coordinate, urls, hashes)
					.transaction_context(|| info_span!("downloading installer dependency", coordinate = %artifact_name))
                    .await
                    .map(|artifact| (index, artifact))
            })
            .buffer_unordered(app.concurrency.download.get())
            .try_collect::<Vec<_>>()
            .await?;
    downloaded.sort_unstable_by_key(|(index, _)| *index);
    stored_artifacts
        .extend(downloaded.into_iter().map(|(_, artifact)| artifact));

    let mut conn = app
        .db
        .connection()
        .context(info_span!("acquiring extract connection"))
        .await?;
    model::ForgelikeExtract::upsert_by_installer_sha256(installer.sha256)
        .metadata(toasty::Json(version))
        .embedded_maven_artifacts(toasty::Json(stored_artifacts))
        .or_ignore()
        .exec(&mut conn)
        .context(info_span!("storing Forge-like installer extract"))
        .await?;
    Ok(())
}

#[derive(Debug)]
enum LibrarySource {
    External {
        urls: Vec<url::Url>,
        hashes: Vec<Sha1>,
    },
    Embedded,
    Generated,
    Upstream,
}

fn classify_library(
    library: &mojang::Library,
    coordinate: &MavenCoordinate,
    embedded_paths: &HashSet<String>,
    modern: bool,
) -> Result<LibrarySource> {
    let embedded = embedded_paths.contains(&coordinate.to_maven_path());
    if !modern && embedded {
        return Ok(LibrarySource::Embedded);
    }
    if modern
        && let Some(artifact) = library
            .downloads
            .as_ref()
            .and_then(|downloads| downloads.artifact.as_ref())
    {
        if let Some(url) = &artifact.url {
            return Ok(LibrarySource::External {
                urls: vec![url.clone()],
                hashes: vec![artifact.sha1],
            });
        }
        return Ok(if embedded {
            LibrarySource::Embedded
        } else {
            LibrarySource::Generated
        });
    }
    if let Some(repository) =
        library.url.as_deref().filter(|url| !url.is_empty())
    {
        let url = repository_artifact_url(repository, coordinate)?;
        if !modern && url.host_str() == Some("libraries.minecraft.net") {
            return Ok(LibrarySource::Upstream);
        }
        let hashes = library
            .checksums
            .as_ref()
            .map(|checksums| {
                checksums
                    .iter()
                    .map(|checksum| {
                        let mut digest = [0; 20];
                        hex::decode_to_slice(checksum, &mut digest)?;
                        Ok::<_, hex::FromHexError>(Sha1(digest))
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()
            .context("parsing library SHA1 checksums")?
            .unwrap_or_default();
        let mut urls = vec![url];
        if modern {
            urls.push(repository_artifact_url(
                "https://libraries.minecraft.net/",
                coordinate,
            )?);
        }
        urls.push(repository_artifact_url(
            "https://maven.creeperhost.net/",
            coordinate,
        )?);
        return Ok(LibrarySource::External { urls, hashes });
    }
    Ok(if embedded {
        LibrarySource::Embedded
    } else if modern {
        LibrarySource::Generated
    } else {
        LibrarySource::Upstream
    })
}

fn normalize_library(library: &mut mojang::Library, source: &LibrarySource) {
    match source {
        LibrarySource::External { .. } | LibrarySource::Embedded => {
            library.downloadable = true
        }
        LibrarySource::Generated => library.downloadable = false,
        LibrarySource::Upstream => {}
    }
}

fn repository_artifact_url(
    repository: &str,
    coordinate: &MavenCoordinate,
) -> Result<url::Url> {
    let mut url = url::Url::parse(repository)
        .context("parsing library repository URL")?;
    if url.scheme() == "http" {
        url.set_scheme("https").map_err(|()| {
            anyhow!("cannot upgrade library repository to HTTPS")
        })?;
    }
    let path = coordinate.to_maven_path();
    url.path_segments_mut()
        .map_err(|()| anyhow!("library repository cannot be a base URL"))?
        .pop_if_empty()
        .extend(
            path.strip_prefix("maven/")
                .context("missing Maven path prefix")?
                .split('/'),
        );
    Ok(url)
}

fn verify_library_hashes(bytes: &[u8], hashes: &[Sha1]) -> Result<()> {
    let actual = Sha1::from_digest(bytes);
    ensure!(
        hashes.is_empty() || hashes.contains(&actual),
        "library SHA1 mismatch: got {actual}, expected one of {hashes:?}"
    );
    Ok(())
}

async fn download_library(
    app: &AppState,
    coordinate: MavenCoordinate,
    urls: Vec<url::Url>,
    hashes: Vec<Sha1>,
) -> Result<model::ForgelikeStoredArtifact> {
    if !hashes.is_empty() {
        let mut conn = app.db.connection().await?;
        let existing = model::BlobHash::all()
            .filter(model::BlobHash::fields().sha1().in_list(hashes.clone()))
            .exec(&mut conn)
            .await?;
        drop(conn);
        for blob in existing {
            if let Ok(bytes) = app.cas.get(blob.sha256).await {
                verify_library_hashes(&bytes, &hashes)?;
                return Ok(model::ForgelikeStoredArtifact {
                    coordinate,
                    sha256: blob.sha256,
                });
            }
        }
    }
    let mut failures = Vec::new();
    for url in urls {
        let result = async {
            let bytes = app
                .http
                .get(url.clone())
                .send()
                .await?
                .error_for_status_ext()
                .await?
                .bytes()
                .await?;
            verify_library_hashes(&bytes, &hashes)?;
            anyhow::Ok(bytes)
        }
        .await;
        match result {
            Ok(bytes) => {
                let blob = app.cas.store(&bytes).await?;
                let mut conn = app
                    .db
                    .connection()
                    .context(info_span!("acquiring artifact connection"))
                    .await?;
                blob.register(&mut conn).await?;
                let sha256 = blob.sha256;
                return Ok(model::ForgelikeStoredArtifact {
                    coordinate,
                    sha256,
                });
            }
            Err(error) => failures.push(format!("{url}: {error:#}")),
        }
    }
    Err(anyhow!(
        "failed to download library {coordinate}: {}",
        failures.join("; ")
    ))
}

fn referenced_maven_artifacts(
    libraries: &[mojang::Library],
) -> Result<Vec<MavenCoordinate>> {
    let mut seen = HashSet::new();
    let mut coordinates = Vec::new();
    for library in libraries {
        let coordinate: MavenCoordinate =
            library.name.parse().with_context(|| {
                format!("parsing referenced library {}", library.name)
            })?;
        let path = coordinate.to_maven_path();
        if seen.insert(path) {
            coordinates.push(coordinate);
        }
    }
    Ok(coordinates)
}

async fn read_file(zip: &ZipFileReader, path: &str) -> Result<Option<Vec<u8>>> {
    let Some(index) = zip.index_for_name(path) else {
        return Ok(None);
    };
    let mut zip = zip.clone();
    let path = path.to_owned();
    tokio::task::spawn_blocking(move || {
        let mut reader = zip
            .by_index(index)
            .with_context(|| format!("opening installer entry {path}"))?;
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .with_context(|| format!("reading installer entry {path}"))?;
        Ok(Some(bytes))
    })
    .await
    .context("joining installer entry reader")?
}

async fn read_json<T: DeserializeOwned>(
    zip: &ZipFileReader,
    path: &str,
) -> Result<Option<T>> {
    let Some(bytes) = read_file(zip, path).await? else {
        return Ok(None);
    };
    let t = from_json_slice::<T>(&bytes).with_context(|| {
        anyhow!("entry {path} must be a `{}`", type_name::<T>())
    })?;
    Ok(Some(t))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;

    fn installer_zip(
        entries: &[(&str, &[u8])],
        compression: zip::CompressionMethod,
    ) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(compression);
        for (path, bytes) in entries {
            writer.start_file(*path, options).unwrap();
            writer.write_all(bytes).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    #[tokio::test]
    async fn reads_installer_entries_concurrently_and_handles_missing_files() {
        let bytes = installer_zip(
            &[
                ("version.json", br#"{"id":"test"}"#),
                ("maven/library.jar", b"jar"),
            ],
            zip::CompressionMethod::Deflated,
        );
        let zip = ZipFileReader::new(Cursor::new(Arc::from(bytes))).unwrap();
        let (version, library) = tokio::join!(
            read_json::<serde_json::Value>(&zip, "version.json"),
            read_file(&zip, "maven/library.jar"),
        );
        assert_eq!(version.unwrap(), Some(json!({"id": "test"})));
        assert_eq!(library.unwrap().as_deref(), Some(b"jar".as_slice()));
        assert!(read_file(&zip, "missing.jar").await.unwrap().is_none());
        assert!(
            read_json::<serde_json::Value>(&zip, "missing.json")
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn rejects_invalid_installer_json() {
        let bytes = installer_zip(
            &[("version.json", b"not JSON")],
            zip::CompressionMethod::Stored,
        );
        let zip = ZipFileReader::new(Cursor::new(Arc::from(bytes))).unwrap();
        assert!(
            read_json::<serde_json::Value>(&zip, "version.json")
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn rejects_installer_entry_checksum_mismatch() {
        let contents = b"embedded library contents";
        let mut bytes = installer_zip(
            &[("maven/library.jar", contents)],
            zip::CompressionMethod::Stored,
        );
        let offset = bytes
            .windows(contents.len())
            .position(|window| window == contents)
            .unwrap();
        bytes[offset] ^= 1;
        let zip = ZipFileReader::new(Cursor::new(Arc::from(bytes))).unwrap();
        assert!(read_file(&zip, "maven/library.jar").await.is_err());
    }

    fn library(value: serde_json::Value) -> mojang::Library {
        serde_json::from_value(value).unwrap()
    }

    fn classify(
        library: &mojang::Library,
        embedded: bool,
        modern: bool,
    ) -> LibrarySource {
        let coordinate = library.name.parse::<MavenCoordinate>().unwrap();
        let paths = if embedded {
            HashSet::from([coordinate.to_maven_path()])
        } else {
            HashSet::new()
        };
        classify_library(library, &coordinate, &paths, modern).unwrap()
    }

    #[test]
    fn direct_artifact_takes_precedence_over_repository_and_embedding() {
        let lib = library(json!({
            "name": "g:a:1", "url": "https://repo.example/",
            "downloads": { "artifact": {
                "url": "https://files.example/a.jar", "size": 1,
                "sha1": "0000000000000000000000000000000000000000"
            }}
        }));
        let LibrarySource::External { urls, hashes } =
            classify(&lib, true, true)
        else {
            panic!("expected external artifact")
        };
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0].as_str(), "https://files.example/a.jar");
        assert_eq!(hashes, vec![Sha1([0; 20])]);
    }

    #[test]
    fn empty_artifact_url_does_not_fall_back_to_repository() {
        for url in [json!(""), json!(null)] {
            let lib = library(json!({
                "name": "g:a:1", "url": "https://repo.example/",
                "downloads": { "artifact": {
                    "url": url, "size": 0,
                    "sha1": "0000000000000000000000000000000000000000"
                }}
            }));
            assert!(matches!(
                classify(&lib, true, true),
                LibrarySource::Embedded
            ));
            assert!(matches!(
                classify(&lib, false, true),
                LibrarySource::Generated
            ));
        }
    }

    #[test]
    fn repository_fallbacks_and_legacy_mojang_behavior() {
        let lib = library(
            json!({ "name": "g:a:1", "url": "https://libraries.minecraft.net/" }),
        );
        assert!(matches!(
            classify(&lib, false, false),
            LibrarySource::Upstream
        ));
        let LibrarySource::External { urls, .. } = classify(&lib, true, true)
        else {
            panic!("expected external artifact")
        };
        assert_eq!(urls.len(), 3);
        assert_eq!(
            urls[0].as_str(),
            "https://libraries.minecraft.net/g/a/1/a-1.jar"
        );
        assert!(matches!(
            classify(&lib, true, false),
            LibrarySource::Embedded
        ));
    }

    #[test]
    fn normalization_preserves_processor_classpath_and_marks_generated_artifacts()
     {
        let mut lib =
            library(json!({ "name": "g:a:1", "include_in_classpath": false }));
        for url in [None, Some("")] {
            lib.url = url.map(str::to_owned);
            let source = classify(&lib, false, true);
            normalize_library(&mut lib, &source);
            assert!(!lib.downloadable);
            assert!(!lib.include_in_classpath);
            let source = classify(&lib, true, true);
            normalize_library(&mut lib, &source);
            assert!(lib.downloadable);
            assert!(!lib.include_in_classpath);
        }
        normalize_library(
            &mut lib,
            &LibrarySource::External {
                urls: Vec::new(),
                hashes: Vec::new(),
            },
        );
        assert!(lib.downloadable);
        assert!(!lib.include_in_classpath);
    }

    #[test]
    fn repository_url_preserves_prefix_and_normalizes_maven_path() {
        let coordinate = "net.example:library:1:client@zip".parse().unwrap();
        for repository in [
            "http://repo.example/releases",
            "https://repo.example/releases/",
        ] {
            assert_eq!(
                repository_artifact_url(repository, &coordinate)
                    .unwrap()
                    .as_str(),
                "https://repo.example/releases/net/example/library/1/library-1-client.zip"
            );
        }
        assert!(repository_artifact_url("invalid", &coordinate).is_err());
    }

    #[test]
    fn supplied_checksums_are_validated() {
        let expected = Sha1::from_digest(b"expected");
        assert!(verify_library_hashes(b"expected", &[expected]).is_ok());
        assert!(verify_library_hashes(b"wrong", &[expected]).is_err());
        assert!(verify_library_hashes(b"anything", &[]).is_ok());
        let mut lib = library(
            json!({ "name": "g:a:1", "url": "https://repo.example/", "checksums": [expected.to_string()] }),
        );
        let LibrarySource::External { hashes, .. } =
            classify(&lib, false, true)
        else {
            panic!("expected external artifact")
        };
        assert_eq!(hashes, vec![expected]);
        lib.checksums = Some(vec!["invalid".to_owned()]);
        assert!(
            classify_library(
                &lib,
                &lib.name.parse().unwrap(),
                &HashSet::new(),
                true
            )
            .is_err()
        );
    }

    #[test]
    fn legacy_install_has_explicit_source_path_and_coordinate() {
        let install: LegacyInstall = serde_json::from_value(json!({
            "minecraft": "1.7.10",
            "path": "net.minecraftforge:forge:1.7.10-10.13.4.1614",
            "filePath": "forge-universal.jar"
        }))
        .unwrap();
        assert_eq!(install.file_path, "forge-universal.jar");
        assert_eq!(
            install.path.to_maven_path(),
            "maven/net/minecraftforge/forge/1.7.10-10.13.4.1614/forge-1.7.10-10.13.4.1614.jar"
        );
    }

    #[test]
    fn legacy_install_requires_valid_coordinate_and_source_path() {
        for value in [
            json!({"minecraft": "1.7.10", "path": "invalid", "filePath": "forge.jar"}),
            json!({"minecraft": "1.7.10", "path": "g:a:1"}),
            json!({"minecraft": "1.7.10", "filePath": "forge.jar"}),
        ] {
            assert!(serde_json::from_value::<LegacyInstall>(value).is_err());
        }
    }
}
