use crate::validate::{
    SupportedGameVersions, ValidationError, ValidationResult,
};
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
        SupportedGameVersions::All
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        if archive.by_name("pack.mcmeta").is_err() {
            return Ok(ValidationResult::Warning(
                "未找到 pack.mcmeta 文件。提示：确保 pack.mcmeta 位于数据包的根目录中！",
            ));
        }

        Ok(ValidationResult::Pass)
    }
}
