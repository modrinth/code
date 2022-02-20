//! Provides utilties for downloading and parsing modpacks

use daedalus::download_file;
use fs_extra::dir::CopyOptions;
use std::{convert::TryFrom, env, io, path::Path};
use tokio::{fs, try_join};
use uuid::Uuid;
use zip::ZipArchive;
use zip_extensions::ZipWriterExtensions;

use self::{
    manifest::Manifest,
    pack::{Modpack, ModpackGame},
};

pub mod manifest;
pub mod modrinth_api;
pub mod pack;

pub const COMPILED_PATH: &str = "compiled/";
pub const COMPILED_ZIP: &str = "compiled.mrpack";
pub const MANIFEST_PATH: &str = "modrinth.index.json";
pub const OVERRIDES_PATH: &str = "overrides/";
pub const PACK_JSON5_PATH: &str = "modpack.json5";
const PACK_GITIGNORE: &'static str = const_format::formatcp!(r#"
{COMPILED_PATH}
{COMPILED_ZIP}
"#);

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

    #[error("Error parsing json5: {0}")]
    Json5Error(#[from] json5::Error),

    #[error("Error joining futures: {0}")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("Versioning Error: {0}")]
    VersionError(String),

    #[error("Error downloading file: {0}")]
    FetchError(#[from] reqwest::Error),

    #[error("Invalid modpack source: {0} (set the WHITELISTED_MODPACK_DOMAINS environment variable to override)")]
    SourceWhitelistError(String),
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
    let mut tmp = env::temp_dir();
    tmp.push(format!("theseus-{}/", Uuid::new_v4()));
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
    if std::fs::read_dir(dest).map_or(false, |it| it.count() != 0) {
        return Err(ModpackError::InvalidDirectory(String::from(
            "Output directory is non-empty",
        )));
    }
    if !dest.exists() {
        fs::create_dir_all(dest).await?;
    }

    // Copy overrides
    let overrides = dir.join(OVERRIDES_PATH);
    if overrides.is_dir() {
        fs_extra::dir::copy(overrides, dest, &CopyOptions::new())?;
    }

    // Parse manifest
    // NOTE: I'm using standard files here, since Serde does not support async readers
    let manifest_path = Some(dir.join(MANIFEST_PATH))
        .filter(|it| it.is_file())
        .ok_or_else(|| {
            ModpackError::ManifestError(String::from("Manifest missing or is not a file"))
        })?;
    let manifest_file = std::fs::File::open(manifest_path)?;
    let reader = io::BufReader::new(manifest_file);

    let manifest: Manifest = serde_json::from_reader(reader)?;
    let modpack = Modpack::try_from(manifest)?;

    // Realise modpack
    modpack.download_files(dest, side).await?;
    Ok(())
}

pub fn to_pack_json5(pack: &Modpack) -> ModpackResult<String> {
    let json5 = json5::to_string(pack)?;
    Ok(format!("// This modpack is managed using Theseus. It can be edited using either a Theseus-compatible launcher or manually.\n{json5}"))
}

pub async fn create_modpack(
    name: &str,
    game: ModpackGame,
    summary: Option<&str>,
) -> ModpackResult<()> {
    let output_dir = Path::new("./").join(name);
    let pack = Modpack::new(game, "0.1.0", name, summary);

    try_join!(
        fs::create_dir(&output_dir),
        fs::create_dir(output_dir.join(OVERRIDES_PATH)),
        fs::write(output_dir.join(".gitignore"), PACK_GITIGNORE),
        fs::write(output_dir.join(PACK_JSON5_PATH), to_pack_json5(&pack)?),
    )?;

    Ok(())
}

pub async fn compile_modpack(dir: &Path) -> ModpackResult<()> {
    let result_dir = dir.join(COMPILED_PATH);
    let pack: Modpack = json5::from_str(&fs::read_to_string(dir.join(PACK_JSON5_PATH)).await?)?;

    fs::create_dir(&result_dir).await?;
    if dir.join(OVERRIDES_PATH).exists() {
        fs_extra::dir::copy(
            dir.join(OVERRIDES_PATH),
            result_dir.join(OVERRIDES_PATH),
            &CopyOptions::new(),
        )?;
    }
    let manifest = Manifest::try_from(pack)?;
    fs::write(
        result_dir.join(MANIFEST_PATH),
        serde_json::to_string(&manifest)?,
    )
    .await?;

    let result_zip = fs::File::create(dir.join(COMPILED_ZIP))
        .await?
        .into_std()
        .await;
    let mut zip = zip::ZipWriter::new(&result_zip);
    zip.create_from_directory(&result_dir)?;

    Ok(())
}
