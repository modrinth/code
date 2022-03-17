use daedalus::minecraft::{ArgumentType, VersionInfo};
use daedalus::modded::LoaderVersion;
use serde::{Deserialize, Serialize};
use std::{path::Path, process::Stdio};
use thiserror::Error;
use tokio::process::{Child, Command};

pub use crate::launcher::auth::provider::Credentials;

mod args;
pub mod auth;
mod download;
mod rules;

#[derive(Error, Debug)]
pub enum LauncherError {
    #[error("Failed to validate file checksum at url {url} with hash {hash} after {tries} tries")]
    ChecksumFailure {
        hash: String,
        url: String,
        tries: u32,
    },

    #[error("Failed to run processor: {0}")]
    ProcessorError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Error while managing asynchronous tasks")]
    TaskError(#[from] tokio::task::JoinError),

    #[error("Error while reading/writing to the disk: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Error while spawning child process {process}")]
    ProcessError {
        inner: std::io::Error,
        process: String,
    },

    #[error("Error while deserializing JSON")]
    SerdeError(#[from] serde_json::Error),

    #[error("Unable to fetch {item}")]
    FetchError { inner: reqwest::Error, item: String },

    #[error("{0}")]
    ParseError(String),

    #[error("Error while fetching metadata: {0}")]
    DaedalusError(#[from] daedalus::Error),

    #[error("Error while reading metadata: {0}")]
    MetaError(#[from] crate::data::DataError),

    #[error("Java error: {0}")]
    JavaError(String),

    #[error("Command exited with non-zero exit code: {0}")]
    ExitError(i32),
}

// TODO: this probably should be in crate::data
#[derive(Debug, Eq, PartialEq, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ModLoader {
    Vanilla,
    Forge,
    Fabric,
}

impl Default for ModLoader {
    fn default() -> Self {
        ModLoader::Vanilla
    }
}

impl std::fmt::Display for ModLoader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let repr = match self {
            &Self::Vanilla => "Vanilla",
            &Self::Forge => "Forge",
            &Self::Fabric => "Fabric",
        };

        f.write_str(repr)
    }
}

pub async fn launch_minecraft(
    game_version: &str,
    loader_version: &Option<LoaderVersion>,
    root_dir: &Path,
    java: &Path,
    java_args: &Vec<String>,
    wrapper: &Option<String>,
    memory: &crate::data::profiles::MemorySettings,
    resolution: &crate::data::profiles::WindowSize,
    credentials: &Credentials,
) -> Result<Child, LauncherError> {
    let (metadata, settings) = futures::try_join! {
        crate::data::Metadata::get(),
        crate::data::Settings::get(),
    }?;
    let root_dir = root_dir.canonicalize()?;
    let metadata_dir = &settings.metadata_dir;

    let (
        versions_path,
        libraries_path,
        assets_path,
        legacy_assets_path,
        natives_path,
    ) = (
        metadata_dir.join("versions"),
        metadata_dir.join("libraries"),
        metadata_dir.join("assets"),
        metadata_dir.join("resources"),
        metadata_dir.join("natives"),
    );

    let version = metadata
        .minecraft
        .versions
        .iter()
        .find(|it| it.id == game_version)
        .ok_or_else(|| {
            LauncherError::InvalidInput(format!(
                "Invalid game version: {game_version}",
            ))
        })?;

    let version_jar = loader_version
        .as_ref()
        .map_or(version.id.clone(), |it| it.id.clone());

    let mut version = download::download_version_info(
        &versions_path,
        version,
        loader_version.as_ref(),
    )
    .await?;

    let client_path = versions_path
        .join(&version.id)
        .join(format!("{}.jar", &version_jar));
    let version_natives_path = natives_path.join(&version.id);

    download_minecraft(
        &version,
        &versions_path,
        &assets_path,
        &legacy_assets_path,
        &libraries_path,
        &version_natives_path,
    )
    .await?;

    if let Some(processors) = &version.processors {
        if let Some(ref mut data) = version.data {
            data.insert(
                "SIDE".to_string(),
                daedalus::modded::SidedDataEntry {
                    client: "client".to_string(),
                    server: "".to_string(),
                },
            );
            data.insert(
                "MINECRAFT_JAR".to_string(),
                daedalus::modded::SidedDataEntry {
                    client: client_path.to_string_lossy().to_string(),
                    server: "".to_string(),
                },
            );
            data.insert(
                "MINECRAFT_VERSION".to_string(),
                daedalus::modded::SidedDataEntry {
                    client: game_version.to_string(),
                    server: "".to_string(),
                },
            );
            data.insert(
                "ROOT".to_string(),
                daedalus::modded::SidedDataEntry {
                    client: root_dir.to_string_lossy().to_string(),
                    server: "".to_string(),
                },
            );
            data.insert(
                "LIBRARY_DIR".to_string(),
                daedalus::modded::SidedDataEntry {
                    client: libraries_path.to_string_lossy().to_string(),
                    server: "".to_string(),
                },
            );

            for processor in processors {
                if let Some(sides) = &processor.sides {
                    if !sides.contains(&"client".to_string()) {
                        continue;
                    }
                }

                let mut cp = processor.classpath.clone();
                cp.push(processor.jar.clone());

                let child = Command::new("java")
                    .arg("-cp")
                    .arg(args::get_class_paths_jar(&libraries_path, &cp)?)
                    .arg(
                        args::get_processor_main_class(args::get_lib_path(
                            &libraries_path,
                            &processor.jar,
                        )?)
                        .await?
                        .ok_or_else(|| {
                            LauncherError::ProcessorError(format!(
                                "Could not find processor main class for {}",
                                processor.jar
                            ))
                        })?,
                    )
                    .args(args::get_processor_arguments(
                        &libraries_path,
                        &processor.args,
                        data,
                    )?)
                    .output()
                    .await
                    .map_err(|err| LauncherError::ProcessError {
                        inner: err,
                        process: "java".to_string(),
                    })?;

                if !child.status.success() {
                    return Err(LauncherError::ProcessorError(
                        String::from_utf8_lossy(&child.stderr).to_string(),
                    ));
                }
            }
        }
    }

    let arguments = version.arguments.unwrap_or_default();
    let mut command = match wrapper {
        Some(hook) => {
            let mut cmd = Command::new(hook);
            cmd.arg(java);
            cmd
        }
        None => Command::new(java.to_string_lossy().to_string()),
    };

    command
        .args(args::get_jvm_arguments(
            arguments.get(&ArgumentType::Jvm).map(|x| x.as_slice()),
            &version_natives_path,
            &libraries_path,
            &args::get_class_paths(
                &libraries_path,
                version.libraries.as_slice(),
                &client_path,
            )?,
            &version_jar,
            *memory,
            java_args.clone(),
        )?)
        .arg(version.main_class)
        .args(args::get_minecraft_arguments(
            arguments.get(&ArgumentType::Game).map(|x| x.as_slice()),
            version.minecraft_arguments.as_deref(),
            credentials,
            &version.id,
            &version.asset_index.id,
            &root_dir,
            &assets_path,
            &version.type_,
            *resolution,
        )?)
        .current_dir(root_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    command.spawn().map_err(|err| LauncherError::ProcessError {
        inner: err,
        process: "minecraft".to_string(),
    })
}

pub async fn download_minecraft(
    version: &VersionInfo,
    versions_dir: &Path,
    assets_dir: &Path,
    legacy_assets_dir: &Path,
    libraries_dir: &Path,
    natives_dir: &Path,
) -> Result<(), LauncherError> {
    let assets_index =
        download::download_assets_index(assets_dir, version).await?;

    let (a, b, c) = futures::future::join3(
        download::download_client(versions_dir, version),
        download::download_assets(
            assets_dir,
            if version.assets == "legacy" {
                Some(legacy_assets_dir)
            } else {
                None
            },
            &assets_index,
        ),
        download::download_libraries(
            libraries_dir,
            natives_dir,
            version.libraries.as_slice(),
        ),
    )
    .await;

    a?;
    b?;
    c?;

    Ok(())
}
