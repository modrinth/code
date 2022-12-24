use crate::validate::{
    SupportedGameVersions, ValidationError, ValidationResult,
};
use std::io::Cursor;
use zip::ZipArchive;

pub struct DataPackValidator;

impl super::Validator for DataPackValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["zip"]
    }

    fn get_project_types(&self) -> &[&str] {
        &["mod"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["datapack"]
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
                "No pack.mcmeta present for datapack file. Tip: Make sure pack.mcmeta is in the root directory of your datapack!".into(),
            )
        })?;

        Ok(ValidationResult::Pass)
    }
}
