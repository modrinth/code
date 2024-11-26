use crate::validate::{
    filter_out_packs, SupportedGameVersions, ValidationError, ValidationResult,
};
use std::io::Cursor;
use zip::ZipArchive;

pub struct LiteLoaderValidator;

impl super::Validator for LiteLoaderValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["litemod", "jar"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["liteloader"]
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
        if archive.by_name("litemod.json").is_err() {
            return Ok(ValidationResult::Warning(
                "LiteLoader 文件中没有 litemod.json 文件",
            ));
        }

        filter_out_packs(archive)?;

        Ok(ValidationResult::Pass)
    }
}
