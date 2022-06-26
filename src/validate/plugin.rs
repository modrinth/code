use crate::validate::{
    SupportedGameVersions, ValidationError, ValidationResult,
};
use std::io::Cursor;
use zip::ZipArchive;

pub struct PluginValidator;

impl super::Validator for PluginValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["zip", "jar"]
    }

    fn get_project_types(&self) -> &[&str] {
        &["mod"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["bukkit", "spigot", "paper", "purpur"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        SupportedGameVersions::All
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        archive.by_name("plugin.yml").map_err(|_| {
            ValidationError::InvalidInput(
                "No plugin.yml present for plugin file.".into(),
            )
        })?;

        Ok(ValidationResult::Pass)
    }
}
