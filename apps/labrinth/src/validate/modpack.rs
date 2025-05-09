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

    fn get_supported_loaders(&self) -> &[&str] {
        &["mrpack"]
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
                if let Ok(file) = archive.by_name("modrinth.index.json") {
                    file
                } else {
                    return Ok(ValidationResult::Warning(
                        "Pack manifest is missing.",
                    ));
                };

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

        if pack.files.is_empty()
            && !archive.file_names().any(|x| x.starts_with("overrides/"))
        {
            return Err(ValidationError::InvalidInput(
                "Pack has no files!".into(),
            ));
        }

        for file in &pack.files {
            if !file.hashes.contains_key(&PackFileHash::Sha1) {
                return Err(ValidationError::InvalidInput(
                    "All pack files must provide a SHA1 hash!".into(),
                ));
            }

            if !file.hashes.contains_key(&PackFileHash::Sha512) {
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
                    ));
                }
            };
        }

        Ok(ValidationResult::PassWithPackDataAndFiles {
            format: pack,
            files: archive
                .file_names()
                .filter(|x| {
                    (x.ends_with("jar") || x.ends_with("zip"))
                        && (x.starts_with("overrides/mods")
                            || x.starts_with("client-overrides/mods")
                            || x.starts_with("server-overrides/mods")
                            || x.starts_with("overrides/resourcepacks")
                            || x.starts_with("server-overrides/resourcepacks")
                            || x.starts_with("overrides/shaderpacks")
                            || x.starts_with("client-overrides/shaderpacks"))
                })
                .flat_map(|x| x.rsplit('/').next().map(|x| x.to_string()))
                .collect::<Vec<String>>(),
        })
    }
}
