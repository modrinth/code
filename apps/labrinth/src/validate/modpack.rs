use crate::models::exp::minecraft::MrpackLoader;
use crate::models::pack::{PackDependency, PackFileHash, PackFormat};
use crate::util::validate::validation_errors_to_string;
use crate::validate::{
    SupportedGameVersions, ValidationError, ValidationResult,
};
use std::io::{Cursor, Read};
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
            let Ok(mut file) = archive.by_name("modrinth.index.json") else {
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

        mrpack_loaders_from_dependencies(&pack.dependencies)?;

        if pack.files.is_empty()
            && !crate::validate::validated_file_names(archive)?
                .into_iter()
                .any(|x| x.starts_with("overrides/"))
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
        }

        Ok(ValidationResult::PassWithPackDataAndFiles {
            format: pack,
            files: crate::validate::validated_file_names(archive)?
                .into_iter()
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
                .filter_map(|x| x.rsplit('/').next().map(|x| x.to_string()))
                .collect::<Vec<String>>(),
        })
    }
}

/// Empty vec = vanilla. More than one loader key is invalid.
pub(crate) fn mrpack_loaders_from_dependencies(
    dependencies: &std::collections::HashMap<PackDependency, String>,
) -> Result<Vec<MrpackLoader>, ValidationError> {
    let loaders = [
        dependencies
            .contains_key(&PackDependency::QuiltLoader)
            .then_some(MrpackLoader::Quilt),
        dependencies
            .contains_key(&PackDependency::FabricLoader)
            .then_some(MrpackLoader::Fabric),
        dependencies
            .contains_key(&PackDependency::Forge)
            .then_some(MrpackLoader::Forge),
        dependencies
            .contains_key(&PackDependency::Neoforge)
            .then_some(MrpackLoader::Neoforge),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    if loaders.len() > 1 {
        return Err(ValidationError::InvalidInput(
            "modpack declares more than one loader".into(),
        ));
    }
    Ok(loaders)
}
