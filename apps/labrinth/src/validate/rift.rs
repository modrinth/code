use crate::validate::{
    filter_out_packs, SupportedGameVersions, ValidationError, ValidationResult,
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
        if archive.by_name("manifest.json").is_ok() {

            return Ok(ValidationResult::Pass);
        }
        if archive.by_name("riftmod.json").is_err() {
            return Ok(ValidationResult::Warning(
                "Rift 文件中没有 riftmod.json 文件。",
            ));
        }

        filter_out_packs(archive)?;

        Ok(ValidationResult::Pass)
    }
}