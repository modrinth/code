use crate::validate::{
    filter_out_packs, SupportedGameVersions, ValidationError, ValidationResult,
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
        // 自 1.13 版本发布以来的时间，这是第一个使用新 TOML 系统的 Forge 版本
        SupportedGameVersions::PastDate(
            DateTime::from_timestamp(1540122067, 0).unwrap(),
        )
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        if archive.by_name("manifest.json").is_ok() {
            return Ok(ValidationResult::Pass);
        }
        if archive.by_name("META-INF/mods.toml").is_err()
            && archive.by_name("META-INF/MANIFEST.MF").is_err()
            && !archive.file_names().any(|x| x.ends_with(".class"))
        {
            return Ok(ValidationResult::Warning(
                "未找到 mods.toml 或有效的类文件。",
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
        // 版本 1.5.2 到 1.12.2 之间的时间，这些版本都使用定义模组的传统方式
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
                "Forge 模组文件不包含 mcmod.info 或有效的类文件！",
            ));
        };

        filter_out_packs(archive)?;

        Ok(ValidationResult::Pass)
    }
}