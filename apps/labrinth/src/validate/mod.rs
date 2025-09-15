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
use ariadne::i18n_enum;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use derive_more::Display;
use std::io::{self, Cursor};
use std::mem;
use std::sync::LazyLock;
use thiserror::Error;
use zip::ZipArchive;
use zip::result::ZipError;

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
    Zip(#[from] ZipError),
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),
    #[error("Error while validating JSON for uploaded file: {0}")]
    SerDe(#[from] serde_json::Error),
    #[error("Invalid Input: {0}")]
    InvalidInput(std::borrow::Cow<'static, str>), // TODO: Use I18nEnum
    #[error("Error while managing threads")]
    Blocking(#[from] actix_web::error::BlockingError),
    #[error("Error while querying database")]
    Database(#[from] DatabaseError),
}

i18n_enum!(
    ValidationError,
    root_key: "labrinth.error.file_validation",
    Zip(cause) => "zip",
    Io(cause) => "io",
    SerDe(cause) => "serialization",
    InvalidInput(cause) => "invalid_input",
    Blocking(..) => "blocking",
    Database(..) => "database",
);

#[derive(Eq, PartialEq, Debug, Display)]
pub enum ValidationWarning {
    #[display(
        "No pack.mcmeta present for datapack file. Tip: Make sure pack.mcmeta is in the root directory of your datapack!"
    )]
    MissingDatapackPackMcmeta,
    #[display("No fabric.mod.json present for Fabric file.")]
    MissingFabricModJson,
    #[display("No mods.toml or valid class files present for Forge file.")]
    MissingModsToml,
    #[display(
        "Forge mod file does not contain mcmod.info or valid class files!"
    )]
    MissingMcmodInfo,
    #[display("No litemod.json present for LiteLoader file.")]
    MissingLitemodJson,
    #[display("Pack manifest is missing.")]
    MissingPackManifest,
    #[display(
        "No neoforge.mods.toml, mods.toml, or valid class files present for NeoForge file."
    )]
    MissingNeoforgeModsToml,
    #[display("No plugin.yml or paper-plugin.yml present for plugin file.")]
    MissingBukkitPluginYml,
    #[display("No plugin.yml or bungee.yml present for plugin file.")]
    MissingBungeecordPluginYml,
    #[display("No velocity-plugin.json present for plugin file.")]
    MissingVelocityPluginJson,
    #[display(
        "No sponge_plugins.json or mcmod.info present for Sponge plugin."
    )]
    MissingSpongePluginsJson,
    #[display("No quilt.mod.json present for Quilt file.")]
    MissingQuiltModJson,
    #[display(
        "No pack.mcmeta present for resource pack file. Tip: Make sure pack.mcmeta is in the root directory of your pack!"
    )]
    MissingResourcePackPackMcmeta,
    #[display("No pack.txt present for pack file.")]
    MissingPackTxt,
    #[display("No riftmod.json present for Rift file.")]
    MissingRiftmodJson,
    #[display("No shaders folder present for OptiFine/Iris shader.")]
    MissingShadersFolder,
    #[display(
        "No pack.mcmeta present for pack file. Tip: Make sure pack.mcmeta is in the root directory of your pack!"
    )]
    MissingCanvasPackMcmeta,
    #[display("No pipeline shaders folder present for canvas shaders.")]
    MissingPipelinesFolder,
    #[display(
        "No pack.mcmeta or vanilla shaders folder present for pack file. Tip: Make sure pack.mcmeta is in the root directory of your pack!"
    )]
    MissingVanillaShadersFolder,
    #[display("File extension is invalid for input file")]
    WrongFileExtension,
    #[display("Invalid modpack file. You must upload a valid .mrpack file.")]
    InvalidMrpack,
}

i18n_enum!(
    ValidationWarning,
    root_key: "labrinth.warning.file_validation",
    MissingDatapackPackMcmeta! => "missing_datapack_pack_mcmeta",
    MissingFabricModJson! => "missing_fabric_mod_json",
    MissingModsToml! => "missing_mods_toml",
    MissingMcmodInfo! => "missing_mcmod_info",
    MissingLitemodJson! => "missing_litemod_json",
    MissingPackManifest! => "missing_pack_manifest",
    MissingNeoforgeModsToml! => "missing_neoforge_mods_toml",
    MissingBukkitPluginYml! => "missing_bukkit_plugin_yml",
    MissingBungeecordPluginYml! => "missing_bungeecord_plugin_yml",
    MissingVelocityPluginJson! => "missing_velocity_plugin_json",
    MissingSpongePluginsJson! => "missing_sponge_plugins_json",
    MissingQuiltModJson! => "missing_quilt_mod_json",
    MissingResourcePackPackMcmeta! => "missing_resource_pack_pack_mcmeta",
    MissingPackTxt! => "missing_pack_txt",
    MissingRiftmodJson! => "missing_riftmod_json",
    MissingShadersFolder! => "missing_shaders_folder",
    MissingCanvasPackMcmeta! => "missing_canvas_pack_mcmeta",
    MissingPipelinesFolder! => "missing_pipelines_folder",
    MissingVanillaShadersFolder! => "missing_vanilla_shaders_folder",
    WrongFileExtension! => "wrong_file_extension",
    InvalidMrpack! => "invalid_mrpack",
);

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
    Warning(ValidationWarning),
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
    Custom(Vec<MinecraftGameVersion>),
}

pub enum MaybeProtectedZipFile {
    Unprotected(ZipArchive<Cursor<Bytes>>),
    MaybeProtected { read_error: ZipError, data: Bytes },
}

pub trait Validator: Sync {
    fn get_file_extensions(&self) -> &[&str];
    fn get_supported_loaders(&self) -> &[&str];
    fn get_supported_game_versions(&self) -> SupportedGameVersions;

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        // By default, any non-protected ZIP archive is valid
        let _ = archive;
        Ok(ValidationResult::Pass)
    }

    fn validate_maybe_protected_zip(
        &self,
        file: &mut MaybeProtectedZipFile,
    ) -> Result<ValidationResult, ValidationError> {
        // By default, validate that the ZIP file is not protected, and if so,
        // delegate to the inner validate method with a known good archive
        match file {
            MaybeProtectedZipFile::Unprotected(archive) => {
                self.validate(archive)
            }
            MaybeProtectedZipFile::MaybeProtected { read_error, .. } => {
                Err(ValidationError::Zip(mem::replace(
                    read_error,
                    ZipError::Io(io::Error::other("ZIP archive reading error")),
                )))
            }
        }
    }
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

/// A regex that matches a potentially protected ZIP archive containing
/// a vanilla Minecraft pack, with a requisite `pack.mcmeta` file.
///
/// Please note that this regex avoids false negatives at the cost of false
/// positives being possible, i.e. it may match files that are not actually
/// Minecraft packs, but it will not miss packs that the game can load.
static PLAUSIBLE_PACK_REGEX: LazyLock<regex::bytes::Regex> =
    LazyLock::new(|| {
        regex::bytes::RegexBuilder::new(concat!(
            r"\x50\x4b\x01\x02", // CEN signature
            r".{24}",            // CEN fields
            r"[\x0B\x0C]\x00",   // CEN file name length
            r".{16}",            // More CEN fields
            r"pack\.mcmeta/?",   // CEN file name
            r".*",               // Rest of CEN entries and records
            r"\x50\x4b\x05\x06", // EOCD signature
        ))
        .unicode(false)
        .dot_matches_new_line(true)
        .build()
        .unwrap()
    });

/// The return value is whether this file should be marked as primary or not, based on the analysis of the file
#[allow(clippy::too_many_arguments)]
pub async fn validate_file(
    data: Bytes,
    file_extension: String,
    loaders: Vec<Loader>,
    file_type: Option<FileType>,
    version_fields: Vec<VersionField>,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    redis: &RedisPool,
) -> Result<ValidationResult, ValidationError> {
    let game_versions = version_fields
        .into_iter()
        .find_map(|v| MinecraftGameVersion::try_from_version_field(v).ok())
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
    data: Bytes,
    file_extension: String,
    loaders: Vec<Loader>,
    game_versions: Vec<MinecraftGameVersion>,
    all_game_versions: Vec<MinecraftGameVersion>,
    file_type: Option<FileType>,
) -> Result<ValidationResult, ValidationError> {
    actix_web::web::block(move || {
        let mut zip = match ZipArchive::new(Cursor::new(Bytes::clone(&data))) {
            Ok(zip) => MaybeProtectedZipFile::Unprotected(zip),
            Err(read_error) => MaybeProtectedZipFile::MaybeProtected {
                read_error,
                data,
            },
        };

        if let Some(file_type) = file_type {
            match file_type {
                FileType::RequiredResourcePack | FileType::OptionalResourcePack => {
                    return PackValidator.validate_maybe_protected_zip(&mut zip);
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
                    let result = validator.validate_maybe_protected_zip(&mut zip)?;
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
                Ok(ValidationResult::Warning(ValidationWarning::WrongFileExtension))
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
                    .is_some_and(|x| x.created > date)
            })
        }
        SupportedGameVersions::Range(before, after) => {
            game_versions.iter().any(|x| {
                all_game_versions
                    .iter()
                    .find(|y| y.version == x.version)
                    .is_some_and(|x| x.created > before && x.created < after)
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
    archive: &mut ZipArchive<Cursor<Bytes>>,
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
        return Ok(ValidationResult::Warning(ValidationWarning::InvalidMrpack));
    }

    Ok(ValidationResult::Pass)
}
