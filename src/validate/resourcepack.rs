use crate::validate::{
    SupportedGameVersions, ValidationError, ValidationResult,
};
use chrono::{DateTime, NaiveDateTime, Utc};
use std::io::Cursor;
use zip::ZipArchive;

pub struct PackValidator;

impl super::Validator for PackValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["zip"]
    }

    fn get_project_types(&self) -> &[&str] {
        &["resourcepack"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["minecraft"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        // Time since release of 13w24a which replaced texture packs with resource packs
        SupportedGameVersions::PastDate(DateTime::from_utc(
            NaiveDateTime::from_timestamp_opt(1371137542, 0).unwrap(),
            Utc,
        ))
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        archive.by_name("pack.mcmeta").map_err(|_| {
            ValidationError::InvalidInput(
                "No pack.mcmeta present for pack file. Tip: Make sure pack.mcmeta is in the root directory of your pack!".into(),
            )
        })?;

        Ok(ValidationResult::Pass)
    }
}

pub struct TexturePackValidator;

impl super::Validator for TexturePackValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["zip"]
    }

    fn get_project_types(&self) -> &[&str] {
        &["resourcepack"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["minecraft"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        // a1.2.2a to 13w23b
        SupportedGameVersions::Range(
            DateTime::from_utc(
                NaiveDateTime::from_timestamp_opt(1289339999, 0).unwrap(),
                Utc,
            ),
            DateTime::from_utc(
                NaiveDateTime::from_timestamp_opt(1370651522, 0).unwrap(),
                Utc,
            ),
        )
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        archive.by_name("pack.txt").map_err(|_| {
            ValidationError::InvalidInput(
                "No pack.txt present for pack file.".into(),
            )
        })?;

        Ok(ValidationResult::Pass)
    }
}
