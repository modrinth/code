use crate::validate::{
    SupportedGameVersions, ValidationError, ValidationResult, filter_out_packs,
};
use chrono::DateTime;
use std::io::Cursor;
use zip::ZipArchive;

pub struct ForgeValidator;

impl super::Validator for ForgeValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["jar", "zip"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["forge"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        // Time since release of 1.13, the first forge version which uses the new TOML system
        SupportedGameVersions::PastDate(
            DateTime::from_timestamp(1540122067, 0).unwrap(),
        )
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        if archive.by_name("META-INF/mods.toml").is_err()
            && archive.by_name("META-INF/MANIFEST.MF").is_err()
            && !archive.file_names().any(|x| x.ends_with(".class"))
        {
            return Ok(ValidationResult::Warning(
                "No mods.toml or valid class files present for Forge file.",
            ));
        }

        filter_out_packs(archive)?;

        Ok(ValidationResult::Pass)
    }
}

pub struct LegacyForgeValidator;

impl super::Validator for LegacyForgeValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["jar", "zip"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["forge"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        // Times between versions 1.5.2 to 1.12.2, which all use the legacy way of defining mods
        SupportedGameVersions::Range(
            DateTime::from_timestamp(0, 0).unwrap(),
            DateTime::from_timestamp(1540122066, 0).unwrap(),
        )
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        if archive.by_name("mcmod.info").is_err()
            && archive.by_name("META-INF/MANIFEST.MF").is_err()
            && !archive.file_names().any(|x| x.ends_with(".class"))
        {
            return Ok(ValidationResult::Warning(
                "Forge mod file does not contain mcmod.info or valid class files!",
            ));
        };

        filter_out_packs(archive)?;

        Ok(ValidationResult::Pass)
    }
}
