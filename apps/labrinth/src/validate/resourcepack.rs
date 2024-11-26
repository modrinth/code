use crate::validate::{
    SupportedGameVersions, ValidationError, ValidationResult,
};
use chrono::DateTime;
use std::io::Cursor;
use zip::ZipArchive;

pub struct PackValidator;

impl super::Validator for PackValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["zip"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["minecraft"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        // 自 13w24a 版本发布以来的时间，该版本将材质包替换为资源包
        SupportedGameVersions::PastDate(
            DateTime::from_timestamp(1371137542, 0).unwrap(),
        )
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        if archive.by_name("pack.mcmeta").is_err() {
            return Ok(ValidationResult::Warning(
                "资源包文件中没有 pack.mcmeta 文件。提示：确保 pack.mcmeta 位于资源包的根目录中！",
            ));
        }

        Ok(ValidationResult::Pass)
    }
}

pub struct TexturePackValidator;

impl super::Validator for TexturePackValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["zip"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["minecraft"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        // a1.2.2a 到 13w23b
        SupportedGameVersions::Range(
            DateTime::from_timestamp(1289339999, 0).unwrap(),
            DateTime::from_timestamp(1370651522, 0).unwrap(),
        )
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        if archive.by_name("pack.txt").is_err() {
            return Ok(ValidationResult::Warning(
                "材质包文件中没有 pack.txt 文件。",
            ));
        }

        Ok(ValidationResult::Pass)
    }
}