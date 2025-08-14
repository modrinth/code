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

    fn get_supported_loaders(&self) -> &[&str] {
        &["bukkit", "spigot", "paper", "purpur", "folia"]
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
            .any(|name| name == "plugin.yml" || name == "paper-plugin.yml")
        {
            return Ok(ValidationResult::Warning(
                "No plugin.yml or paper-plugin.yml present for plugin file.",
            ));
        };

        Ok(ValidationResult::Pass)
    }
}

pub struct BungeeCordValidator;

impl super::Validator for BungeeCordValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["zip", "jar"]
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
        if !archive
            .file_names()
            .any(|name| name == "plugin.yml" || name == "bungee.yml")
        {
            return Ok(ValidationResult::Warning(
                "No plugin.yml or bungee.yml present for plugin file.",
            ));
        };

        Ok(ValidationResult::Pass)
    }
}

pub struct VelocityValidator;

impl super::Validator for VelocityValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["zip", "jar"]
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
        if !archive.file_names().any(|name| {
            name == "sponge_plugins.json"
                || name == "mcmod.info"
                || name == "META-INF/sponge_plugins.json"
        }) {
            return Ok(ValidationResult::Warning(
                "No sponge_plugins.json or mcmod.info present for Sponge plugin.",
            ));
        };

        Ok(ValidationResult::Pass)
    }
}
