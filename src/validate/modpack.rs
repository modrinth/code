use crate::models::pack::{PackFileHash, PackFormat};
use crate::util::validate::validation_errors_to_string;
use crate::validate::{
    SupportedGameVersions, ValidationError, ValidationResult,
};
use std::io::{Cursor, Read};
use std::path::Component;
use validator::Validate;
use zip::ZipArchive;

pub struct ModpackValidator;

impl super::Validator for ModpackValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["mrpack"]
    }

    fn get_project_types(&self) -> &[&str] {
        &["modpack"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["forge", "fabric", "quilt"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        SupportedGameVersions::All
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        let pack: PackFormat = {
            let mut file =
                archive.by_name("modrinth.index.json").map_err(|_| {
                    ValidationError::InvalidInput(
                        "Pack manifest is missing.".into(),
                    )
                })?;

            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            serde_json::from_str(&contents)?
        };

        pack.validate().map_err(|err| {
            ValidationError::InvalidInput(
                validation_errors_to_string(err, None).into(),
            )
        })?;

        if pack.game != "minecraft" {
            return Err(ValidationError::InvalidInput(
                format!("Game {0} does not exist!", pack.game).into(),
            ));
        }

        for file in &pack.files {
            if file.hashes.get(&PackFileHash::Sha1).is_none() {
                return Err(ValidationError::InvalidInput(
                    "All pack files must provide a SHA1 hash!".into(),
                ));
            }

            if file.hashes.get(&PackFileHash::Sha512).is_none() {
                return Err(ValidationError::InvalidInput(
                    "All pack files must provide a SHA512 hash!".into(),
                ));
            }

            let path = std::path::Path::new(&file.path)
                .components()
                .next()
                .ok_or_else(|| {
                    ValidationError::InvalidInput(
                        "Invalid pack file path!".into(),
                    )
                })?;

            match path {
                Component::CurDir | Component::Normal(_) => {}
                _ => {
                    return Err(ValidationError::InvalidInput(
                        "Invalid pack file path!".into(),
                    ))
                }
            };
        }

        Ok(ValidationResult::PassWithPackDataAndFiles {
            format: pack,
            files: archive
                .file_names()
                .filter(|x| {
                    x.starts_with("overrides/mods")
                        || x.starts_with("client-overrides/mods")
                        || x.starts_with("server-overrides/mods")
                })
                .flat_map(|x| x.rsplit('/').next().map(|x| x.to_string()))
                .collect::<Vec<String>>(),
        })
    }
}
