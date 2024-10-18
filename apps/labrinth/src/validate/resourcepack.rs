use crate::validate::{
    SupportedGameVersions, ValidationError, ValidationResult,
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

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        if archive.by_name("pack.mcmeta").is_err() {
            return Ok(ValidationResult::Warning(
                "No pack.mcmeta present for pack file. Tip: Make sure pack.mcmeta is in the root directory of your pack!",
            ));
        }

        Ok(ValidationResult::Pass)
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
                "No pack.txt present for pack file.",
            ));
        }

        Ok(ValidationResult::Pass)
    }
}
