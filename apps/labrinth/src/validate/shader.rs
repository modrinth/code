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
                "No shaders folder present for OptiFine/Iris shader.",
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
                "No pack.mcmeta present for pack file. Tip: Make sure pack.mcmeta is in the root directory of your pack!",
            ));
        };

        if !archive.file_names().any(|x| x.contains("/pipelines/")) {
            return Ok(ValidationResult::Warning(
                "No pipeline shaders folder present for canvas shaders.",
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
                "No pack.mcmeta present for pack file. Tip: Make sure pack.mcmeta is in the root directory of your pack!",
            ));
        };

        if !archive
            .file_names()
            .any(|x| x.starts_with("assets/minecraft/shaders/"))
        {
            return Ok(ValidationResult::Warning(
                "No shaders folder present for vanilla shaders.",
            ));
        }

        Ok(ValidationResult::Pass)
    }
}
