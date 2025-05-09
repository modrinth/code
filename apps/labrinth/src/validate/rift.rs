use crate::validate::{
    SupportedGameVersions, ValidationError, ValidationResult, filter_out_packs,
};
use std::io::Cursor;
use zip::ZipArchive;

pub struct RiftValidator;

impl super::Validator for RiftValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["jar"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["rift"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        SupportedGameVersions::All
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        if archive.by_name("riftmod.json").is_err() {
            return Ok(ValidationResult::Warning(
                "No riftmod.json present for Rift file.",
            ));
        }

        filter_out_packs(archive)?;

        Ok(ValidationResult::Pass)
    }
}
