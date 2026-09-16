use crate::validate::{
    SupportedGameVersions, ValidationError, ValidationResult,
};
use std::io::Cursor;
use zip::ZipArchive;

pub struct RisugamiValidator;

impl super::Validator for RisugamiValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["zip", "jar"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["modloader"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        SupportedGameVersions::All
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        if archive.file_names().any(|name| name == "instance.cfg")
            && archive.file_names().any(|name| name == "mmc-pack.json")
        {
            return Err(ValidationError::InvalidInput(
				"multimc and prism modpacks cannot be uploaded as risugami's modloader mods".into(),
			));
        }

        if archive.file_names().any(|name| name == "level.dat") {
            return Err(ValidationError::InvalidInput(
				"minecraft worlds cannot be uploaded as risugami's modloader mods".into(),
			));
        }

        for name in archive.file_names() {
            let Some((_, extension)) = name.rsplit_once('.') else {
                continue;
            };
            if ["mcpack", "mcworld", "mctemplate"]
                .iter()
                .any(|blocked| extension.eq_ignore_ascii_case(blocked))
            {
                return Err(ValidationError::InvalidInput(
					"bedrock packs cannot be uploaded as risugami's modloader mods".into(),
				));
            }
            if extension.eq_ignore_ascii_case("sk") {
                return Err(ValidationError::InvalidInput(
					"skript packs cannot be uploaded as risugami's modloader mods".into(),
				));
            }
        }

        Ok(ValidationResult::Pass)
    }
}
