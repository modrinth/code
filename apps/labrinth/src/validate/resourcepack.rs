use crate::validate::{
    MaybeProtectedZipFile, PLAUSIBLE_PACK_REGEX, SupportedGameVersions,
    ValidationError, ValidationResult, ValidationWarning,
};
use chrono::DateTime;
use std::io::Cursor;
use zip::ZipArchive;

pub struct PackValidator;

impl super::Validator for PackValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["zip"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["minecraft"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        // Time since release of 13w24a which replaced texture packs with resource packs
        SupportedGameVersions::PastDate(
            DateTime::from_timestamp(1371137542, 0).unwrap(),
        )
    }

    fn validate_maybe_protected_zip(
        &self,
        file: &mut MaybeProtectedZipFile,
    ) -> Result<ValidationResult, ValidationError> {
        if match file {
            MaybeProtectedZipFile::Unprotected(archive) => {
                archive.by_name("pack.mcmeta").is_ok()
            }
            MaybeProtectedZipFile::MaybeProtected { data, .. } => {
                PLAUSIBLE_PACK_REGEX.is_match(data)
            }
        } {
            Ok(ValidationResult::Pass)
        } else {
            Ok(ValidationResult::Warning(
                ValidationWarning::MissingResourcePackPackMcmeta,
            ))
        }
    }
}

pub struct TexturePackValidator;

impl super::Validator for TexturePackValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["zip"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["minecraft"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        // a1.2.2a to 13w23b
        SupportedGameVersions::Range(
            DateTime::from_timestamp(1289339999, 0).unwrap(),
            DateTime::from_timestamp(1370651522, 0).unwrap(),
        )
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        if archive.by_name("pack.txt").is_err() {
            return Ok(ValidationResult::Warning(
                ValidationWarning::MissingPackTxt,
            ));
        }

        Ok(ValidationResult::Pass)
    }
}
