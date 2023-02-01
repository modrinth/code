use crate::models::pack::PackFormat;
use crate::models::projects::{FileType, GameVersion, Loader};
use crate::validate::datapack::DataPackValidator;
use crate::validate::fabric::FabricValidator;
use crate::validate::forge::{ForgeValidator, LegacyForgeValidator};
use crate::validate::liteloader::LiteLoaderValidator;
use crate::validate::modpack::ModpackValidator;
use crate::validate::plugin::*;
use crate::validate::quilt::QuiltValidator;
use crate::validate::resourcepack::{PackValidator, TexturePackValidator};
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
pub mod plugin;
mod quilt;
mod resourcepack;
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
}

#[derive(Eq, PartialEq)]
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
    Custom(Vec<GameVersion>),
}

pub trait Validator: Sync {
    fn get_file_extensions(&self) -> &[&str];
    fn get_project_types(&self) -> &[&str];
    fn get_supported_loaders(&self) -> &[&str];
    fn get_supported_game_versions(&self) -> SupportedGameVersions;
    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError>;
}

static VALIDATORS: [&dyn Validator; 16] = [
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
];

/// The return value is whether this file should be marked as primary or not, based on the analysis of the file
pub async fn validate_file(
    data: bytes::Bytes,
    file_extension: String,
    mut project_type: String,
    mut loaders: Vec<Loader>,
    game_versions: Vec<GameVersion>,
    all_game_versions: Vec<crate::database::models::categories::GameVersion>,
    file_type: Option<FileType>,
) -> Result<ValidationResult, ValidationError> {
    actix_web::web::block(move || {
        let reader = Cursor::new(data);
        let mut zip = ZipArchive::new(reader)?;

        if let Some(file_type) = file_type {
            match file_type {
                FileType::RequiredResourcePack
                | FileType::OptionalResourcePack => {
                    project_type = "resourcepack".to_string();
                    loaders = vec![Loader("minecraft".to_string())];
                }
                FileType::Unknown => {}
            }
        }

        let mut visited = false;
        for validator in &VALIDATORS {
            if validator.get_project_types().contains(&&*project_type)
                && loaders
                    .iter()
                    .any(|x| validator.get_supported_loaders().contains(&&*x.0))
                && game_version_supported(
                    &game_versions,
                    &all_game_versions,
                    validator.get_supported_game_versions(),
                )
            {
                if validator.get_file_extensions().contains(&&*file_extension) {
                    return validator.validate(&mut zip);
                } else {
                    visited = true;
                }
            }
        }

        if visited {
            Err(ValidationError::InvalidInput(
                format!(
                    "File extension {file_extension} is invalid for input file"
                )
                .into(),
            ))
        } else {
            Ok(ValidationResult::Pass)
        }
    })
    .await?
}

fn game_version_supported(
    game_versions: &[GameVersion],
    all_game_versions: &[crate::database::models::categories::GameVersion],
    supported_game_versions: SupportedGameVersions,
) -> bool {
    match supported_game_versions {
        SupportedGameVersions::All => true,
        SupportedGameVersions::PastDate(date) => {
            game_versions.iter().any(|x| {
                all_game_versions
                    .iter()
                    .find(|y| y.version == x.0)
                    .map(|x| x.created > date)
                    .unwrap_or(false)
            })
        }
        SupportedGameVersions::Range(before, after) => {
            game_versions.iter().any(|x| {
                all_game_versions
                    .iter()
                    .find(|y| y.version == x.0)
                    .map(|x| x.created > before && x.created < after)
                    .unwrap_or(false)
            })
        }
        SupportedGameVersions::Custom(versions) => {
            versions.iter().any(|x| game_versions.contains(x))
        }
    }
}
