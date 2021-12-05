//! Provides utilties for downloading and parsing modpacks

use daedalus::download_file;
use fs_extra::dir::CopyOptions;
use serde::Deserialize;
use std::{convert::TryFrom, env, io, path::Path};
use tokio::fs;
use uuid::Uuid;
use zip::ZipArchive;

use self::{manifest::Manifest, pack::Modpack};

pub mod pack;
pub mod manifest;
pub mod modrinth_api;

pub const MANIFEST_PATH: &'static str = "index.json";
pub const OVERRIDES_PATH: &'static str = "overrides/";

#[derive(thiserror::Error, Debug)]
pub enum ModpackError {
    #[error("I/O error while reading modpack: {0}")]
    IOError(#[from] io::Error),

    #[error("I/O error while reading modpack: {0}")]
    FSExtraError(#[from] fs_extra::error::Error),

    #[error("Error extracting archive: {0}")]
    ZipError(#[from] zip::result::ZipError),

    #[error("Invalid modpack format: {0}")]
    FormatError(String),

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

    #[error("Versioning Error: {0}")]
    VersionError(String),

    #[error("Error downloading file: {0}")]
    FetchError(#[from] reqwest::Error)
}

type ModpackResult<T> = Result<T, ModpackError>;

/// Realise a modpack from a given URL
pub async fn fetch_modpack(
    url: &str,
    sha1: Option<&str>,
    dest: &Path,
    side: pack::ModpackSide,
) -> ModpackResult<()> {
    let bytes = download_file(url, sha1).await?;
    let mut archive = ZipArchive::new(io::Cursor::new(&bytes as &[u8]))?;
    realise_modpack_zip(&mut archive, dest, side).await
}

/// Realise a given modpack from a zip archive
pub async fn realise_modpack_zip(
    archive: &mut ZipArchive<impl io::Read + io::Seek>,
    dest: &Path,
    side: pack::ModpackSide,
) -> ModpackResult<()> {
    let tmp = env::temp_dir().join(format!("theseus-{}/", Uuid::new_v4()));
    archive.extract(&tmp)?;
    realise_modpack(&tmp, dest, side).await
}

/// Realise a given modpack into an instance
pub async fn realise_modpack(
    dir: &Path,
    dest: &Path,
    side: pack::ModpackSide,
) -> ModpackResult<()> {
    if dest.is_file() {
        return Err(ModpackError::InvalidDirectory(String::from(
            "Output is not a directory",
        )));
    }
    if dest.exists() && std::fs::read_dir(dest).map_or(false, |it| it.count() != 0) {
        return Err(ModpackError::InvalidDirectory(String::from(
            "Output directory is non-empty",
        )));
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
        .ok_or(ModpackError::ManifestError(String::from(
            "Manifest missing or is not a file",
        )))?;
    let manifest_file = std::fs::File::open(manifest_path)?;
    let reader = io::BufReader::new(manifest_file);
    let mut deserializer = serde_json::Deserializer::from_reader(reader);
    let manifest = Manifest::deserialize(&mut deserializer)?;
    let modpack = Modpack::try_from(manifest)?;

    // Realise modpack
    modpack.download_files(dest, side).await?;
    Ok(())
}
