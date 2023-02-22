use crate::validate::{
    SupportedGameVersions, ValidationError, ValidationResult,
};
use chrono::{DateTime, NaiveDateTime, Utc};
use std::io::Cursor;
use zip::ZipArchive;

pub struct QuiltValidator;

impl super::Validator for QuiltValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["jar", "zip"]
    }

    fn get_project_types(&self) -> &[&str] {
        &["mod"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["quilt"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        SupportedGameVersions::PastDate(DateTime::from_utc(
            NaiveDateTime::from_timestamp_opt(1646070100, 0).unwrap(),
            Utc,
        ))
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        archive.by_name("quilt.mod.json").map_err(|_| {
            ValidationError::InvalidInput(
                "No quilt.mod.json present for Quilt file.".into(),
            )
        })?;

        if !archive.file_names().any(|name| {
            name.ends_with("refmap.json") || name.ends_with(".class")
        }) {
            return Ok(ValidationResult::Warning(
                "Quilt mod file is a source file!",
            ));
        }

        Ok(ValidationResult::Pass)
    }
}
