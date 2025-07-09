use crate::validate::{
    SupportedGameVersions, ValidationError, ValidationResult, filter_out_packs,
};
use std::io::Cursor;
use zip::ZipArchive;

pub struct NeoForgeValidator;

impl super::Validator for NeoForgeValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["jar", "zip"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["neoforge"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        SupportedGameVersions::All
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        if archive.by_name("META-INF/mods.toml").is_err()
            && archive.by_name("META-INF/neoforge.mods.toml").is_err()
            && archive.by_name("META-INF/MANIFEST.MF").is_err()
            && !archive.file_names().any(|x| x.ends_with(".class"))
        {
            return Ok(ValidationResult::Warning(
                "No neoforge.mods.toml, mods.toml, or valid class files present for NeoForge file.",
            ));
        }

        filter_out_packs(archive)?;

        Ok(ValidationResult::Pass)
    }
}
