use crate::validate::{
    SupportedGameVersions, ValidationError, ValidationResult,
};
use std::io::Cursor;
use zip::ZipArchive;

pub struct ResourcePackValidator;

impl super::Validator for ResourcePackValidator {
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
        SupportedGameVersions::All
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        archive.by_name("pack.mcmeta").map_err(|_| {
            ValidationError::InvalidInput(
                "No pack.mcmeta present for resourcepack file.".into(),
            )
        })?;

        Ok(ValidationResult::Pass)
    }
}
