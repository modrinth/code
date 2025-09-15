use crate::validate::{
    MaybeProtectedZipFile, PLAUSIBLE_PACK_REGEX, SupportedGameVersions,
    ValidationError, ValidationResult, ValidationWarning,
};
use std::{io::Cursor, sync::LazyLock};
use zip::ZipArchive;

pub struct ShaderValidator;

impl super::Validator for ShaderValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["zip"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["optifine", "iris"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        SupportedGameVersions::All
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        if !archive.file_names().any(|x| x.starts_with("shaders/")) {
            return Ok(ValidationResult::Warning(
                ValidationWarning::MissingShadersFolder,
            ));
        }

        Ok(ValidationResult::Pass)
    }
}

pub struct CanvasShaderValidator;

impl super::Validator for CanvasShaderValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["zip"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["canvas"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        SupportedGameVersions::All
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        if archive.by_name("pack.mcmeta").is_err() {
            return Ok(ValidationResult::Warning(
                ValidationWarning::MissingCanvasPackMcmeta,
            ));
        };

        if !archive.file_names().any(|x| x.contains("/pipelines/")) {
            return Ok(ValidationResult::Warning(
                ValidationWarning::MissingPipelinesFolder,
            ));
        }

        Ok(ValidationResult::Pass)
    }
}

pub struct CoreShaderValidator;

impl super::Validator for CoreShaderValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["zip"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["vanilla"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        SupportedGameVersions::All
    }

    fn validate_maybe_protected_zip(
        &self,
        file: &mut MaybeProtectedZipFile,
    ) -> Result<ValidationResult, ValidationError> {
        static VANILLA_SHADER_CEN_ENTRY_REGEX: LazyLock<regex::bytes::Regex> =
            LazyLock::new(|| {
                regex::bytes::RegexBuilder::new(concat!(
                    r"\x50\x4b\x01\x02",          // CEN signature
                    r".{24}",                     // CEN fields
                    r".{2}",                      // CEN file name length
                    r".{16}",                     // More CEN fields
                    r"assets/minecraft/shaders/", // CEN file name
                ))
                .unicode(false)
                .dot_matches_new_line(true)
                .build()
                .unwrap()
            });

        if match file {
            MaybeProtectedZipFile::Unprotected(archive) => {
                archive.by_name("pack.mcmeta").is_ok()
                    && archive
                        .file_names()
                        .any(|x| x.starts_with("assets/minecraft/shaders/"))
            }
            MaybeProtectedZipFile::MaybeProtected { data, .. } => {
                PLAUSIBLE_PACK_REGEX.is_match(data)
                    && VANILLA_SHADER_CEN_ENTRY_REGEX.is_match(data)
            }
        } {
            Ok(ValidationResult::Pass)
        } else {
            Ok(ValidationResult::Warning(
                ValidationWarning::MissingVanillaShadersFolder,
            ))
        }
    }
}
