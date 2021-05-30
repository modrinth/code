use crate::models::projects::SideType;
use crate::validate::{SupportedGameVersions, ValidationError, ValidationResult};
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read};
use zip::ZipArchive;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackFormat {
    pub game: String,
    pub format_version: i32,
    pub version_id: String,
    pub name: String,
    pub summary: Option<String>,
    pub dependencies: std::collections::HashMap<PackDependency, String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackFile {
    pub path: String,
    pub hashes: std::collections::HashMap<String, String>,
    pub env: Environment,
    pub downloads: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Environment {
    pub client: SideType,
    pub server: SideType,
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
        write!(fmt, "{}", self.as_str())
    }
}

impl PackDependency {
    // These are constant, so this can remove unneccessary allocations (`to_string`)
    pub fn as_str(&self) -> &'static str {
        match self {
            PackDependency::Forge => "forge",
            PackDependency::FabricLoader => "fabric-loader",
            PackDependency::Minecraft => "minecraft",
        }
    }
}

pub struct PackValidator {}

impl super::Validator for PackValidator {
    fn get_file_extensions<'a>(&self) -> &'a [&'a str] {
        &["zip"]
    }

    fn get_project_types<'a>(&self) -> &'a [&'a str] {
        &["modpack"]
    }

    fn get_supported_loaders<'a>(&self) -> &'a [&'a str] {
        &["forge", "fabric"]
    }

    fn get_supported_game_versions(&self) -> SupportedGameVersions {
        SupportedGameVersions::All
    }

    fn validate(
        &self,
        archive: &mut ZipArchive<Cursor<&[u8]>>,
    ) -> Result<ValidationResult, ValidationError> {
        let mut file = archive.by_name("index.json")?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let pack: PackFormat = serde_json::from_str(&*contents)?;

        if pack.game != *"minecraft" {
            return Err(ValidationError::InvalidInputError(format!(
                "Game {0} does not exist!",
                pack.game
            )));
        }

        Ok(ValidationResult::Pass)
    }
}
