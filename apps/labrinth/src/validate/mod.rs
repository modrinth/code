use crate::database::models::DatabaseError;
use crate::database::models::legacy_loader_fields::MinecraftGameVersion;
use crate::database::models::loader_fields::VersionField;
use crate::database::redis::RedisPool;
use crate::models::pack::PackFormat;
use crate::models::projects::{FileType, Loader};
use crate::validate::datapack::DataPackValidator;
use crate::validate::fabric::FabricValidator;
use crate::validate::forge::{ForgeValidator, LegacyForgeValidator};
use crate::validate::liteloader::LiteLoaderValidator;
use crate::validate::modpack::ModpackValidator;
use crate::validate::neoforge::NeoForgeValidator;
use crate::validate::plugin::*;
use crate::validate::quilt::QuiltValidator;
use crate::validate::resourcepack::{PackValidator, TexturePackValidator};
use crate::validate::rift::RiftValidator;
use crate::validate::shader::{
    CanvasShaderValidator, CoreShaderValidator, ShaderValidator,
};
use chrono::{DateTime, Utc};
use std::io::Cursor;
use thiserror::Error;
use zip::ZipArchive;

mod datapack;
mod fabric;
mod forge;
mod liteloader;
mod modpack;
mod neoforge;
pub mod plugin;
mod quilt;
mod resourcepack;
mod rift;
mod shader;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Unable to read Zip Archive: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Error while validating JSON for uploaded file: {0}")]
    SerDe(#[from] serde_json::Error),
    #[error("Invalid Input: {0}")]
    InvalidInput(std::borrow::Cow<'static, str>),
    #[error("Error while managing threads")]
    Blocking(#[from] actix_web::error::BlockingError),
    #[error("Error while querying database")]
    Database(#[from] DatabaseError),
}

#[derive(Eq, PartialEq, Debug)]
pub enum ValidationResult {
    /// File should be marked as primary with pack file data
    PassWithPackDataAndFiles {
        format: PackFormat,
        files: Vec<String>,
    },
    /// File should be marked as primary
    Pass,
    /// File should not be marked primary, the reason for which is inside the String
    Warning(&'static str),
}

impl ValidationResult {
    pub fn is_passed(&self) -> bool {
        match self {
            ValidationResult::PassWithPackDataAndFiles { .. } => true,
            ValidationResult::Pass => true,
            ValidationResult::Warning(_) => false,
        }
    }
}

pub enum SupportedGameVersions {
    All,
    PastDate(DateTime<Utc>),
    Range(DateTime<Utc>, DateTime<Utc>),
    #[allow(dead_code)]
    Custom(Vec<MinecraftGameVersion>),
}

pub trait Validator: Sync {
    fn get_file_extensions(&self) -> &[&str];
    fn get_supported_loaders(&self) -> &[&str];
    fn get_supported_game_versions(&self) -> SupportedGameVersions;
    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError>;
}

static ALWAYS_ALLOWED_EXT: &[&str] = &["zip", "txt"];

static VALIDATORS: &[&dyn Validator] = &[
    &ModpackValidator,
    &FabricValidator,
    &ForgeValidator,
    &LegacyForgeValidator,
    &QuiltValidator,
    &LiteLoaderValidator,
    &PackValidator,
    &TexturePackValidator,
    &PluginYmlValidator,
    &BungeeCordValidator,
    &VelocityValidator,
    &SpongeValidator,
    &CanvasShaderValidator,
    &ShaderValidator,
    &CoreShaderValidator,
    &DataPackValidator,
    &RiftValidator,
    &NeoForgeValidator,
];

/// The return value is whether this file should be marked as primary or not, based on the analysis of the file
#[allow(clippy::too_many_arguments)]
pub async fn validate_file(
    data: bytes::Bytes,
    file_extension: String,
    loaders: Vec<Loader>,
    file_type: Option<FileType>,
    version_fields: Vec<VersionField>,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    redis: &RedisPool,
) -> Result<ValidationResult, ValidationError> {
    let game_versions = version_fields
        .into_iter()
        .find_map(|v| MinecraftGameVersion::try_from_version_field(&v).ok())
        .unwrap_or_default();
    let all_game_versions =
        MinecraftGameVersion::list(None, None, &mut *transaction, redis)
            .await?;

    validate_minecraft_file(
        data,
        file_extension,
        loaders,
        game_versions,
        all_game_versions,
        file_type,
    )
    .await
}

async fn validate_minecraft_file(
    data: bytes::Bytes,
    file_extension: String,
    loaders: Vec<Loader>,
    game_versions: Vec<MinecraftGameVersion>,
    all_game_versions: Vec<MinecraftGameVersion>,
    file_type: Option<FileType>,
) -> Result<ValidationResult, ValidationError> {
    actix_web::web::block(move || {
        let reader = Cursor::new(data);
        let mut zip = ZipArchive::new(reader)?;

        if let Some(file_type) = file_type {
            match file_type {
                FileType::RequiredResourcePack | FileType::OptionalResourcePack => {
                    return PackValidator.validate(&mut zip);
                }
                FileType::Unknown => {}
            }
        }

        let mut visited = false;
        let mut saved_result = None;
        for validator in VALIDATORS {
            if loaders
                .iter()
                .any(|x| validator.get_supported_loaders().contains(&&*x.0))
                && game_version_supported(
                    &game_versions,
                    &all_game_versions,
                    validator.get_supported_game_versions(),
                )
            {
                if validator.get_file_extensions().contains(&&*file_extension) {
                    let result = validator.validate(&mut zip)?;
                    match result {
                        ValidationResult::PassWithPackDataAndFiles { .. } => {
                            saved_result = Some(result);
                        }
                        ValidationResult::Pass => {
                            if saved_result.is_none() {
                                saved_result = Some(result);
                            }
                        }
                        ValidationResult::Warning(_) => {
                            return Ok(result);
                        }
                    }
                } else {
                    visited = true;
                }
            }
        }

        if let Some(result) = saved_result {
            return Ok(result);
        }

        if visited {
            if ALWAYS_ALLOWED_EXT.contains(&&*file_extension) {
                Ok(ValidationResult::Warning(
                    "File extension is invalid for input file",
                ))
            } else {
                Err(ValidationError::InvalidInput(
                    format!("File extension {file_extension} is invalid for input file").into(),
                ))
            }
        } else {
            Ok(ValidationResult::Pass)
        }
    })
    .await?
}

// Write tests for this
fn game_version_supported(
    game_versions: &[MinecraftGameVersion],
    all_game_versions: &[MinecraftGameVersion],
    supported_game_versions: SupportedGameVersions,
) -> bool {
    match supported_game_versions {
        SupportedGameVersions::All => true,
        SupportedGameVersions::PastDate(date) => {
            game_versions.iter().any(|x| {
                all_game_versions
                    .iter()
                    .find(|y| y.version == x.version)
                    .map(|x| x.created > date)
                    .unwrap_or(false)
            })
        }
        SupportedGameVersions::Range(before, after) => {
            game_versions.iter().any(|x| {
                all_game_versions
                    .iter()
                    .find(|y| y.version == x.version)
                    .map(|x| x.created > before && x.created < after)
                    .unwrap_or(false)
            })
        }
        SupportedGameVersions::Custom(versions) => {
            let version_ids =
                versions.iter().map(|gv| gv.id).collect::<Vec<_>>();
            let game_version_ids: Vec<_> =
                game_versions.iter().map(|gv| gv.id).collect::<Vec<_>>();
            version_ids.iter().any(|x| game_version_ids.contains(x))
        }
    }
}

pub fn filter_out_packs(
    archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
) -> Result<ValidationResult, ValidationError> {
    if (archive.by_name("modlist.html").is_ok()
        && archive.by_name("manifest.json").is_ok())
        || archive
            .file_names()
            .any(|x| x.starts_with("mods/") && x.ends_with(".jar"))
        || archive
            .file_names()
            .any(|x| x.starts_with("override/mods/") && x.ends_with(".jar"))
    {
        return Ok(ValidationResult::Warning(
            "Invalid modpack file. You must upload a valid .MRPACK file.",
        ));
    }

    Ok(ValidationResult::Pass)
}
