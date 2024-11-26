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
                "插件文件中没有 plugin.yml 或 paper-plugin.yml 文件。",
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
                "插件文件中没有 plugin.yml 或 bungee.yml 文件。",
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
                "插件文件中没有 velocity-plugin.json 文件。",
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
                "Sponge 插件中没有 sponge_plugins.json 或 mcmod.info 文件。",
            ));
        };

        Ok(ValidationResult::Pass)
    }
}