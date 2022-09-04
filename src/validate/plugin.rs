use crate::validate::{
    SupportedGameVersions, ValidationError, ValidationResult,
};
use std::io::Cursor;
use zip::ZipArchive;

pub struct PluginYmlValidator;

impl super::Validator for PluginYmlValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["zip", "jar"]
    }

    fn get_project_types(&self) -> &[&str] {
        &["mod"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &[
            "bukkit",
            "spigot",
            "paper",
            "purpur",
            "bungeecord",
            "waterfall",
        ]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        SupportedGameVersions::All
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        if archive.by_name("plugin.yml").is_err() {
            return Ok(ValidationResult::Warning(
                "No plugin.yml present for plugin file.",
            ));
        }

        Ok(ValidationResult::Pass)
    }
}

pub struct BungeeCordValidator;

impl super::Validator for BungeeCordValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["zip", "jar"]
    }

    fn get_project_types(&self) -> &[&str] {
        &["mod"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["bungeecord", "waterfall"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        SupportedGameVersions::All
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        archive.by_name("bungee.yml").map_err(|_| {
            ValidationError::InvalidInput(
                "No bungee.yml present for plugin file.".into(),
            )
        })?;

        Ok(ValidationResult::Pass)
    }
}

pub struct VelocityValidator;

impl super::Validator for VelocityValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["zip", "jar"]
    }

    fn get_project_types(&self) -> &[&str] {
        &["mod"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["velocity"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        SupportedGameVersions::All
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        if archive.by_name("velocity-plugin.json").is_err() {
            return Ok(ValidationResult::Warning(
                "No velocity-plugin.json present for plugin file.",
            ));
        }

        Ok(ValidationResult::Pass)
    }
}

pub struct SpongeValidator;

impl super::Validator for SpongeValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["zip", "jar"]
    }

    fn get_project_types(&self) -> &[&str] {
        &["mod"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["sponge"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        SupportedGameVersions::All
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        if !archive
            .file_names()
            .any(|name| name == "sponge_plugins.json" || name == "mcmod.info")
        {
            return Ok(ValidationResult::Warning(
                "No sponge_plugins.json or mcmod.info present for Sponge plugin.",
            ));
        };

        Ok(ValidationResult::Pass)
    }
}
