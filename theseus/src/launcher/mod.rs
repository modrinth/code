use daedalus::minecraft::{ArgumentType, VersionInfo};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::{Command, Stdio};
use thiserror::Error;

pub use crate::launcher::auth::provider::Credentials;

mod args;
mod auth;
mod download;
mod java;
mod rules;

#[derive(Error, Debug)]
pub enum LauncherError {
    #[error("Failed to violate file checksum at url {url} with hash {hash} after {tries} tries")]
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
}

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

pub async fn launch_minecraft(
    version_name: &str,
    mod_loader: Option<ModLoader>,
    root_dir: &Path,
    credentials: &Credentials,
) -> Result<(), LauncherError> {
    let metadata = crate::data::Metadata::get().await?;
    let settings = crate::data::Settings::get().await?;

    let versions_path = crate::util::absolute_path(root_dir.join("versions"))?;
    let libraries_path = crate::util::absolute_path(root_dir.join("libraries"))?;
    let assets_path = crate::util::absolute_path(root_dir.join("assets"))?;
    let legacy_assets_path = crate::util::absolute_path(root_dir.join("resources"))?;

    let version = metadata
        .minecraft
        .versions
        .iter()
        .find(|x| x.id == version_name)
        .ok_or_else(|| {
            LauncherError::InvalidInput(format!("Version {} does not exist", version_name))
        })?;

    let loader_version = match mod_loader.unwrap_or_default() {
        ModLoader::Vanilla => None,
        ModLoader::Forge | ModLoader::Fabric => {
            let loaders = if mod_loader.unwrap_or_default() == ModLoader::Forge {
                &metadata
                    .forge
                    .game_versions
                    .iter()
                    .find(|x| x.id == version_name)
                    .ok_or_else(|| {
                        LauncherError::InvalidInput(format!(
                            "Version {} for mod loader Forge does not exist",
                            version_name
                        ))
                    })?
                    .loaders
            } else {
                &metadata
                    .fabric
                    .game_versions
                    .iter()
                    .find(|x| x.id == version_name)
                    .ok_or_else(|| {
                        LauncherError::InvalidInput(format!(
                            "Version {} for mod loader Fabric does not exist",
                            version_name
                        ))
                    })?
                    .loaders
            };

            let loader = if let Some(version) = loaders.iter().find(|x| x.stable) {
                Some(version.clone())
            } else {
                loaders.first().cloned()
            };

            Some(loader.ok_or_else(|| {
                LauncherError::InvalidInput(format!(
                    "No mod loader version found for version {}",
                    version_name
                ))
            })?)
        }
    };

    let version_jar_name = if let Some(loader) = &loader_version {
        loader.id.clone()
    } else {
        version.id.clone()
    };

    let mut version =
        download::download_version_info(&versions_path, version, loader_version.as_ref()).await?;

    let java_path = if let Some(java) = &version.java_version {
        if java.major_version == 17 || java.major_version == 16 {
            settings.java_17_path.as_deref().ok_or_else(|| LauncherError::JavaError("Please install Java 17 or select your Java 17 installation settings before launching this version!".to_string()))?
        } else {
            &settings.java_8_path.as_deref().ok_or_else(|| LauncherError::JavaError("Please install Java 8 or select your Java 8 installation settings before launching this version!".to_string()))?
        }
    } else {
        &settings.java_8_path.as_deref().ok_or_else(|| LauncherError::JavaError("Please install Java 8 or select your Java 8 installation settings before launching this version!".to_string()))?
    };

    let client_path = crate::util::absolute_path(
        root_dir
            .join("versions")
            .join(&version.id)
            .join(format!("{}.jar", &version.id)),
    )?;
    let natives_path = crate::util::absolute_path(root_dir.join("natives").join(&version.id))?;

    download_minecraft(
        &version,
        &versions_path,
        &assets_path,
        &legacy_assets_path,
        &libraries_path,
        &natives_path,
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
                    client: version_name.to_string(),
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

    let mut command = Command::new(if let Some(wrapper) = &settings.hooks.wrapper {
        wrapper.clone()
    } else {
        String::from(java_path.to_string_lossy())
    });

    if settings.hooks.wrapper.is_some() {
        command.arg(java_path);
    }

    command
        .args(args::get_jvm_arguments(
            arguments.get(&ArgumentType::Jvm).map(|x| x.as_slice()),
            &natives_path,
            &libraries_path,
            &args::get_class_paths(&libraries_path, version.libraries.as_slice(), &client_path)?,
            &version_jar_name,
            settings.memory,
            settings.custom_java_args.iter().map(String::from).collect(),
        )?)
        .arg(version.main_class)
        .args(args::get_minecraft_arguments(
            arguments.get(&ArgumentType::Game).map(|x| x.as_slice()),
            version.minecraft_arguments.as_deref(),
            credentials,
            &version.id,
            &version.asset_index.id,
            root_dir,
            &assets_path,
            &version.type_,
            settings.game_resolution,
        )?)
        .current_dir(root_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let mut child = command.spawn().map_err(|err| LauncherError::ProcessError {
        inner: err,
        process: "minecraft".to_string(),
    })?;

    child.wait().map_err(|err| LauncherError::ProcessError {
        inner: err,
        process: "minecraft".to_string(),
    })?;

    Ok(())
}

pub async fn download_minecraft(
    version: &VersionInfo,
    versions_dir: &Path,
    assets_dir: &Path,
    legacy_assets_dir: &Path,
    libraries_dir: &Path,
    natives_dir: &Path,
) -> Result<(), LauncherError> {
    let assets_index = download::download_assets_index(assets_dir, version).await?;

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
        download::download_libraries(libraries_dir, natives_dir, version.libraries.as_slice()),
    )
    .await;

    a?;
    b?;
    c?;

    Ok(())
}
