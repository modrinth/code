//! Provides utilties for downloading and parsing modpacks

use std::{convert::TryFrom, io, path::Path};
use fs_extra::dir::CopyOptions;
use tokio::fs;

use self::manifest::Manifest;

mod manifest;

pub const MANIFEST_PATH: &'static str = "index.json";
pub const OVERRIDES_PATH: &'static str = "overrides/";

#[derive(thiserror::Error, Debug)]
pub enum ModpackError {
    #[error("I/O error while reading modpack: {0}")]
    IOError(#[from] io::Error),

    #[error("I/O error while reading modpack: {0}")]
    FSExtraError(#[from] fs_extra::error::Error),

    #[error("Invalid output directory: {0}")]
    InvalidDirectory(String),

    #[error("Error parsing manifest: {0}")]
    ManifestError(String),

    #[error("Daedalus error: {0}")]
    DaedalusError(#[from] daedalus::Error),

    #[error("Error parsing json: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Error joining futures: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}

/// Realise a given modpack into an instance
pub async fn realise_modpack(dir: &Path, dest: &Path, side: &manifest::ModpackSide) -> Result<(), ModpackError> {
    if dest.is_file() {
        return Err(ModpackError::InvalidDirectory(String::from("Output is not a directory")));
    }
    if dest.exists() && std::fs::read_dir(dest).map_or(false, |it| it.count() != 0) {
        return Err(ModpackError::InvalidDirectory(String::from("Output directory is non-empty")));
    }
    if !dest.exists() {
        fs::create_dir_all(dest).await?;
    }

    // Copy overrides
    let overrides = Some(dir.join(OVERRIDES_PATH)).filter(|it| it.exists() && it.is_dir());
    if let Some(overrides) = overrides {
        fs_extra::dir::copy(overrides, dest, &CopyOptions::new())?;
    }

    // Parse manifest
    // NOTE: I'm using standard files here, since Serde does not support async readers
    let manifest_path = Some(dir.join(MANIFEST_PATH))
        .filter(|it| it.exists() && it.is_file())
        .ok_or(ModpackError::ManifestError(String::from("Manifest missing or is not a file")))?;
    let manifest_file = std::fs::File::open(manifest_path)?;
    let manifest_json: serde_json::Value = serde_json::from_reader(io::BufReader::new(manifest_file))?;
    let manifest = Manifest::try_from(manifest_json)?;

    // Realise manifest
    manifest.download_files(dest, side).await?;
    Ok(())
}
