use crate::validate::{
    SupportedGameVersions, ValidationError, ValidationResult,
};
use std::io::Cursor;
use time::OffsetDateTime;
use zip::ZipArchive;

pub struct ForgeValidator;

impl super::Validator for ForgeValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["jar", "zip"]
    }

    fn get_project_types(&self) -> &[&str] {
        &["mod"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["forge"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        // Time since release of 1.13, the first forge version which uses the new TOML system
        SupportedGameVersions::PastDate(OffsetDateTime::from_unix_timestamp(
            1540122067,
        ))
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        archive.by_name("META-INF/mods.toml").map_err(|_| {
            ValidationError::InvalidInput(
                "No mods.toml present for Forge file.".into(),
            )
        })?;

        if !archive.file_names().any(|name| name.ends_with(".class")) {
            return Ok(ValidationResult::Warning(
                "Forge mod file is a source file!",
            ));
        }

        //TODO: Check if file is a dev JAR?

        Ok(ValidationResult::Pass)
    }
}

pub struct LegacyForgeValidator;

impl super::Validator for LegacyForgeValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["jar", "zip"]
    }

    fn get_project_types(&self) -> &[&str] {
        &["mod"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["forge"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        // Times between versions 1.5.2 to 1.12.2, which all use the legacy way of defining mods
        SupportedGameVersions::Range(
            OffsetDateTime::from_unix_timestamp(1366818300),
            OffsetDateTime::from_unix_timestamp(1505810340),
        )
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        if archive.by_name("mcmod.info").is_err() {
            return Ok(ValidationResult::Warning(
                "Forge mod file does not contain mcmod.info!",
            ));
        };

        if !archive.file_names().any(|name| name.ends_with(".class")) {
            return Ok(ValidationResult::Warning(
                "Forge mod file is a source file!",
            ));
        }

        //TODO: Check if file is a dev JAR?

        Ok(ValidationResult::Pass)
    }
}
