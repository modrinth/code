use crate::validate::{
    SupportedGameVersions, ValidationError, ValidationResult,
};
use chrono::DateTime;
use std::io::Cursor;
use zip::ZipArchive;

pub struct DataPackValidator;

impl super::Validator for DataPackValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["zip"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["datapack"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        // Time since release of 17w43a, 2017-10-25, which introduced datapacks
        SupportedGameVersions::PastDate(
            DateTime::from_timestamp(1508889600, 0).unwrap(),
        )
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        if archive.by_name("pack.mcmeta").is_err() {
            return Ok(ValidationResult::Warning(
                "No pack.mcmeta present for datapack file. Tip: Make sure pack.mcmeta is in the root directory of your datapack!",
            ));
        }

        Ok(ValidationResult::Pass)
    }
}
