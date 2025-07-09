use crate::validate::{
    SupportedGameVersions, ValidationError, ValidationResult, filter_out_packs,
};
use chrono::DateTime;
use std::io::Cursor;
use zip::ZipArchive;

pub struct QuiltValidator;

impl super::Validator for QuiltValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["jar", "zip"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["quilt"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        SupportedGameVersions::PastDate(
            DateTime::from_timestamp(1646070100, 0).unwrap(),
        )
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        if archive.by_name("quilt.mod.json").is_err()
            && archive.by_name("fabric.mod.json").is_err()
        {
            return Ok(ValidationResult::Warning(
                "No quilt.mod.json present for Quilt file.",
            ));
        }

        filter_out_packs(archive)?;

        Ok(ValidationResult::Pass)
    }
}
