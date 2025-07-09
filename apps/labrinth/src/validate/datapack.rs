use crate::validate::{
    MaybeProtectedZipFile, PLAUSIBLE_PACK_REGEX, SupportedGameVersions,
    ValidationError, ValidationResult,
};
use chrono::DateTime;

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

    fn validate_maybe_protected_zip(
        &self,
        file: &mut MaybeProtectedZipFile,
    ) -> Result<ValidationResult, ValidationError> {
        if match file {
            MaybeProtectedZipFile::Unprotected(archive) => {
                archive.by_name("pack.mcmeta").is_ok()
            }
            MaybeProtectedZipFile::MaybeProtected { data, .. } => {
                PLAUSIBLE_PACK_REGEX.is_match(data)
            }
        } {
            Ok(ValidationResult::Pass)
        } else {
            Ok(ValidationResult::Warning(
                "No pack.mcmeta present for datapack file. Tip: Make sure pack.mcmeta is in the root directory of your datapack!",
            ))
        }
    }
}
