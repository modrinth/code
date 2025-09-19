use crate::validate::{
    SupportedGameVersions, ValidationError, ValidationResult, ValidationWarning,
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
                ValidationWarning::WrongFileExtension,
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
                ValidationWarning::MissingBungeecordPluginYml,
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
                ValidationWarning::MissingVelocityPluginJson,
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
                ValidationWarning::MissingSpongePluginsJson,
            ));
        };

        Ok(ValidationResult::Pass)
    }
}
