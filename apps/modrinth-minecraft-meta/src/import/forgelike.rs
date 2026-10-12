use std::{
    any::type_name,
    collections::{HashMap, HashSet},
};

use anyhow::{Context, Result, anyhow, ensure};
use futures::{StreamExt, TryStreamExt, stream};
use jiff::Timestamp;
use serde::{Deserialize, de::DeserializeOwned};
use tracing::{debug, info, info_span, warn};
use tracing_anyhow::FutureContext;

use crate::{
    AppState, model,
    upstream::{self, mojang},
    util::{MavenCoordinate, from_json_slice, from_json_value},
};

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
    let unprocessed_installers = model::ForgelikeInstaller::all()
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
            .context(info_span!("extracting from installer", %name, %sha256))
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
    data: HashMap<String, model::SidedDataEntry>,
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
struct LegacyInstall {
    minecraft: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyVersionInfo {
    id: String,
    release_time: Timestamp,
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
            HashMap::from([(
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

type ZipFileReader = async_zip::base::read::mem::ZipFileReader;

async fn extract_from_installer(
    app: &AppState,
    installer: model::ForgelikeInstaller,
) -> Result<()> {
    let installer_blob = app
        .cas
        .get(installer.sha256)
        .context(info_span!("fetching installer blob"))
        .await?;
    let zip = ZipFileReader::new(installer_blob)
        .context(info_span!("opening installer zip for reading"))
        .await?;

    let install_profile =
        read_json::<serde_json::Value>(&zip, "install_profile.json")
            .context(info_span!("reading install profile"))
            .await?
            .context("installer is missing install_profile.json")?;
    let version =
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
                profile.into_metadata()
            }
        };
    let referenced_artifacts = referenced_maven_artifacts(&version.libraries)?;
    let embedded_maven_artifacts = {
        let embedded_paths = zip
            .file()
            .entries()
            .iter()
            .map(|entry| entry.filename().as_str())
            .collect::<Result<HashSet<_>, _>>()?;
        referenced_artifacts
            .into_iter()
            .filter(|coordinate| {
                embedded_paths.contains(coordinate.to_maven_path().as_str())
            })
            .collect::<Vec<_>>()
    };

    debug!(
        %version.id,
        num_embedded_maven_artifacts = embedded_maven_artifacts.len(),
        "read installer metadata"
    );

    let stored_artifacts = stream::iter(embedded_maven_artifacts.into_iter())
        .map(|coordinate| {
                let zip = &zip;
                async move {
                    let path = coordinate.to_maven_path();
                    let bytes = read_file(zip, &path)
                        .context(info_span!(
                            "extracting embedded Maven artifact",
                            path
                        ))
                        .await?
                        .context(
                            "embedded Maven artifact is missing from installer",
                        )?;
                    let mut conn = app.db.connection()
                        .context(info_span!("acquiring artifact connection"))
                        .await?;
                    let sha256 = app.cas
                        .put(&mut conn, &bytes)
                        .context(info_span!("storing embedded Maven artifact in CAS", path, %coordinate))
                        .await?;
                    anyhow::Ok(model::ForgelikeStoredArtifact { coordinate, sha256 })
                }
        })
        .buffered(app.concurrency.extract_files.get())
        .try_collect::<Vec<_>>()
        .await?;

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
    let index = zip
        .file()
        .entries()
        .iter()
        .position(|entry| entry.filename().as_str().ok() == Some(path));
    let Some(index) = index else {
        return Ok(None);
    };

    let mut reader = zip
        .reader_with_entry(index)
        .context(info_span!("opening installer entry", path))
        .await?;
    let mut bytes = Vec::new();
    reader
        .read_to_end_checked(&mut bytes)
        .context(info_span!("reading installer entry", path))
        .await?;
    Ok(Some(bytes))
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
