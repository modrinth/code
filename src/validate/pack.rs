use crate::models::projects::SideType;
use crate::util::validate::validation_errors_to_string;
use crate::validate::{SupportedGameVersions, ValidationError, ValidationResult};
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read};
use validator::Validate;
use zip::ZipArchive;

#[derive(Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct PackFormat<'a> {
    pub game: &'a str,
    pub format_version: i32,
    #[validate(length(min = 3, max = 512))]
    pub version_id: &'a str,
    #[validate(length(min = 3, max = 512))]
    pub name: &'a str,
    #[validate(length(max = 2048))]
    pub summary: Option<&'a str>,
    #[validate]
    pub files: Vec<PackFile<'a>>,
    pub dependencies: std::collections::HashMap<PackDependency, &'a str>,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct PackFile<'a> {
    pub path: &'a str,
    pub hashes: std::collections::HashMap<FileHash, &'a str>,
    pub env: std::collections::HashMap<EnvType, SideType>,
    #[validate(custom(function = "validate_download_url"))]
    pub downloads: Vec<&'a str>,
}

fn validate_download_url(values: &Vec<&str>) -> Result<(), validator::ValidationError> {
    for value in values {
        if !validator::validate_url(*value) {
            return Err(validator::ValidationError::new("invalid URL"));
        }
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum FileHash {
    Sha1,
    Sha512,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum EnvType {
    Client,
    Server,
}

#[derive(Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PackDependency {
    Forge,
    FabricLoader,
    Minecraft,
}

impl std::fmt::Display for PackDependency {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str(self.as_str())
    }
}

impl PackDependency {
    // These are constant, so this can remove unnecessary allocations (`to_string`)
    pub fn as_str(&self) -> &'static str {
        match self {
            PackDependency::Forge => "forge",
            PackDependency::FabricLoader => "fabric-loader",
            PackDependency::Minecraft => "minecraft",
        }
    }
}

pub struct PackValidator;

impl super::Validator for PackValidator {
    fn get_file_extensions(&self) -> &[&str] {
        &["zip"]
    }

    fn get_project_types(&self) -> &[&str] {
        &["modpack"]
    }

    fn get_supported_loaders(&self) -> &[&str] {
        &["forge", "fabric"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        SupportedGameVersions::All
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<bytes::Bytes>>,
    ) -> Result<ValidationResult, ValidationError> {
        let mut file = archive
            .by_name("index.json")
            .map_err(|_| ValidationError::InvalidInputError("Pack manifest is missing.".into()))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let pack: PackFormat = serde_json::from_str(&contents)?;

        pack.validate().map_err(|err| {
            ValidationError::InvalidInputError(validation_errors_to_string(err, None).into())
        })?;

        if pack.game != "minecraft" {
            return Err(ValidationError::InvalidInputError(
                format!("Game {0} does not exist!", pack.game).into(),
            ));
        }

        Ok(ValidationResult::Pass)
    }
}
