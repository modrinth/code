use crate::validate::{
    SupportedGameVersions, ValidationError, ValidationResult,
};
use std::io::Cursor;
use time::OffsetDateTime;
use zip::ZipArchive;

pub struct FabricValidator;

impl super::Validator for FabricValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["jar", "zip"]
    }

    fn get_project_types(&self) -> &[&str] {
        &["mod"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["fabric"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        // Time since release of 18w49a, the first fabric version
        SupportedGameVersions::PastDate(OffsetDateTime::from_unix_timestamp(
            1543969469,
        ))
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        archive.by_name("fabric.mod.json").map_err(|_| {
            ValidationError::InvalidInput(
                "No fabric.mod.json present for Fabric file.".into(),
            )
        })?;

        if !archive.file_names().any(|name| {
            name.ends_with("refmap.json") || name.ends_with(".class")
        }) {
            return Ok(ValidationResult::Warning(
                "Fabric mod file is a source file!",
            ));
        }

        //TODO: Check if file is a dev JAR?

        Ok(ValidationResult::Pass)
    }
}
