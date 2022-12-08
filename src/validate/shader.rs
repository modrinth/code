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

    fn get_project_types(&self) -> &[&str] {
        &["shader"]
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
            return Err(ValidationError::InvalidInput(
                "No shaders folder present for OptiFine/Iris shader.".into(),
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

    fn get_project_types(&self) -> &[&str] {
        &["shader"]
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
        archive.by_name("pack.mcmeta").map_err(|_| {
            ValidationError::InvalidInput(
                "No pack.mcmeta present for pack file. Tip: Make sure pack.mcmeta is in the root directory of your pack!".into(),
            )
        })?;

        if !archive.file_names().any(|x| x.contains("/pipelines/")) {
            return Err(ValidationError::InvalidInput(
                "No pipeline shaders folder present for canvas shaders.".into(),
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

    fn get_project_types(&self) -> &[&str] {
        &["shader"]
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
        archive.by_name("pack.mcmeta").map_err(|_| {
            ValidationError::InvalidInput(
                "No pack.mcmeta present for pack file. Tip: Make sure pack.mcmeta is in the root directory of your pack!".into(),
            )
        })?;

        if !archive
            .file_names()
            .any(|x| x.starts_with("assets/minecraft/shaders/"))
        {
            return Err(ValidationError::InvalidInput(
                "No shaders folder present for vanilla shaders.".into(),
            ));
        }

        Ok(ValidationResult::Pass)
    }
}
