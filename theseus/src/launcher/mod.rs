use crate::launcher::auth::provider::Credentials;
use std::path::Path;
use std::process::{Command, Stdio};
use thiserror::Error;

pub mod args;
pub mod auth;
pub mod download;
pub mod java;
pub mod meta;
pub mod rules;

#[derive(Error, Debug)]
pub enum LauncherError {
    #[error("Failed to violate file checksum at url {url} with hash {hash} after {tries} tries")]
    ChecksumFailure {
        hash: String,
        url: String,
        tries: u32,
    },
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Error while managing asynchronous tasks")]
    TaskError(#[from] tokio::task::JoinError),
    #[error("Error while reading/writing to the disk")]
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
}

pub async fn launch_minecraft(
    version_name: &str,
    root_dir: &Path,
    credentials: &Credentials,
) -> Result<(), LauncherError> {
    let manifest = meta::fetch_version_manifest().await.unwrap();

    let version = download::download_version_info(
        &*root_dir.join("versions"),
        manifest
            .versions
            .iter()
            .find(|x| x.id == version_name)
            .ok_or_else(|| {
                LauncherError::InvalidInput(format!("Version {} does not exist", version_name))
            })?,
    )
    .await?;

    download_minecraft(&version, root_dir).await?;

    let arguments = version.arguments.unwrap();

    let mut child = Command::new("java")
        .args(args::get_jvm_arguments(
            arguments
                .get(&meta::ArgumentType::Jvm)
                .map(|x| x.as_slice()),
            &*root_dir.join("natives").join(&version.id),
            &*args::get_class_paths(
                &*root_dir.join("libraries"),
                version.libraries.as_slice(),
                &*root_dir
                    .join("versions")
                    .join(&version.id)
                    .join(format!("{}.jar", &version.id)),
            )?,
        )?)
        .arg(version.main_class)
        .args(args::get_minecraft_arguments(
            arguments
                .get(&meta::ArgumentType::Game)
                .map(|x| x.as_slice()),
            version.minecraft_arguments.as_deref(),
            credentials,
            &*version.id,
            &version.asset_index.id,
            root_dir,
            &*root_dir.join("assets"),
            &version.type_,
        )?)
        .current_dir(root_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|err| LauncherError::ProcessError {
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
    version: &meta::VersionInfo,
    root_dir: &Path,
) -> Result<(), LauncherError> {
    let assets_index = download::download_assets_index(&*root_dir.join("assets"), &version).await?;

    let legacy_dir = root_dir.join("resources");

    let (a, b, c) = futures::future::join3(
        download::download_client(&*root_dir.join("versions"), &version),
        download::download_assets(
            &*root_dir.join("assets"),
            if version.assets == "legacy" {
                Some(legacy_dir.as_path())
            } else {
                None
            },
            &assets_index,
        ),
        download::download_libraries(
            &*root_dir.join("libraries"),
            &*root_dir.join("natives").join(&version.id),
            version.libraries.as_slice(),
        ),
    )
    .await;

    a?;
    b?;
    c?;

    Ok(())
}
