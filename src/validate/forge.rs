use crate::validate::{SupportedGameVersions, ValidationError, ValidationResult};
use chrono::{DateTime, NaiveDateTime, Utc};
use std::io::Cursor;
use zip::ZipArchive;

pub struct ForgeValidator {}

impl super::Validator for ForgeValidator {
    fn get_file_extensions<'a>(&self) -> &'a [&'a str] {
        &["jar", "zip"]
    }

    fn get_project_types<'a>(&self) -> &'a [&'a str] {
        &["mod"]
    }

    fn get_supported_loaders<'a>(&self) -> &'a [&'a str] {
        &["forge"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        // Time since release of 1.13, the first forge version which uses the new TOML system
        SupportedGameVersions::PastDate(DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp(1540122067, 0),
            Utc,
        ))
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<&[u8]>>,
    ) -> Result<ValidationResult, ValidationError> {
        archive.by_name("META-INF/mods.toml")?;

        if !archive.file_names().any(|name| name.ends_with(".class")) {
            return Ok(ValidationResult::Warning(
                "Forge mod file is a source file!".to_string(),
            ));
        }

        //TODO: Check if file is a dev JAR?

        Ok(ValidationResult::Pass)
    }
}

pub struct LegacyForgeValidator {}

impl super::Validator for LegacyForgeValidator {
    fn get_file_extensions<'a>(&self) -> &'a [&'a str] {
        &["jar", "zip"]
    }

    fn get_project_types<'a>(&self) -> &'a [&'a str] {
        &["mod"]
    }

    fn get_supported_loaders<'a>(&self) -> &'a [&'a str] {
        &["forge"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        // Times between versions 1.5.2 to 1.12.2, which all use the legacy way of defining mods
        SupportedGameVersions::Range(
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(1366818300, 0), Utc),
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(1505810340, 0), Utc),
        )
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<&[u8]>>,
    ) -> Result<ValidationResult, ValidationError> {
        archive.by_name("mcmod.info")?;

        if !archive.file_names().any(|name| name.ends_with(".class")) {
            return Ok(ValidationResult::Warning(
                "Forge mod file is a source file!".to_string(),
            ));
        }

        //TODO: Check if file is a dev JAR?

        Ok(ValidationResult::Pass)
    }
}
