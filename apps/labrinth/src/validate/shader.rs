use crate::validate::{
    SupportedGameVersions, ValidationError, ValidationResult,
};
use std::io::Cursor;
use zip::ZipArchive;

pub struct ShaderValidator;

impl super::Validator for ShaderValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["zip"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["optifine", "iris"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        SupportedGameVersions::All
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        if !archive.file_names().any(|x| x.starts_with("shaders/")) {
            return Ok(ValidationResult::Warning(
                "OptiFine/Iris shader 中没有 shaders 文件夹。",
            ));
        }

        Ok(ValidationResult::Pass)
    }
}

pub struct CanvasShaderValidator;

impl super::Validator for CanvasShaderValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["zip"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["canvas"]
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
                "pack 文件中没有 pack.mcmeta 文件。提示：确保 pack.mcmeta 位于 pack 的根目录中！",
            ));
        };

        if !archive.file_names().any(|x| x.contains("/pipelines/")) {
            return Ok(ValidationResult::Warning(
                "canvas shaders 中没有 pipeline shaders 文件夹。",
            ));
        }

        Ok(ValidationResult::Pass)
    }
}

pub struct CoreShaderValidator;

impl super::Validator for CoreShaderValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["zip"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["vanilla"]
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
                "pack 文件中没有 pack.mcmeta 文件。提示：确保 pack.mcmeta 位于 pack 的根目录中！",
            ));
        };

        if !archive
            .file_names()
            .any(|x| x.starts_with("assets/minecraft/shaders/"))
        {
            return Ok(ValidationResult::Warning(
                "vanilla shaders 中没有 shaders 文件夹。",
            ));
        }

        Ok(ValidationResult::Pass)
    }
}