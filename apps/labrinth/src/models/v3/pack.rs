use crate::{
    models::v2::projects::LegacySideType, util::env::parse_strings_from_var,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Validate, Eq, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PackFormat {
    pub game: String,
    pub format_version: i32,
    #[validate(length(min = 1, max = 512))]
    pub version_id: String,
    #[validate(length(min = 1, max = 512))]
    pub name: String,
    #[validate(length(max = 2048))]
    pub summary: Option<String>,
    #[validate(nested)]
    pub files: Vec<PackFile>,
    pub dependencies: std::collections::HashMap<PackDependency, String>,
}

#[derive(Serialize, Deserialize, Validate, Eq, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PackFile {
    pub path: String,
    pub hashes: std::collections::HashMap<PackFileHash, String>,
    pub env: Option<std::collections::HashMap<EnvType, LegacySideType>>, // TODO: Should this use LegacySideType? Will probably require a overhaul of mrpack format to change this
    #[validate(custom(function = "validate_download_url"))]
    pub downloads: Vec<String>,
    pub file_size: u32,
}

fn validate_download_url(
    values: &[String],
) -> Result<(), validator::ValidationError> {
    for value in values {
        let url = url::Url::parse(value)
            .ok()
            .ok_or_else(|| validator::ValidationError::new("invalid URL"))?;

        if url.as_str() != value {
            return Err(validator::ValidationError::new("invalid URL"));
        }

        let domains = parse_strings_from_var("WHITELISTED_MODPACK_DOMAINS")
            .unwrap_or_default();
        if !domains.contains(
            &url.domain()
                .ok_or_else(|| validator::ValidationError::new("invalid URL"))?
                .to_string(),
        ) {
            return Err(validator::ValidationError::new(
                "File download source is not from allowed sources",
            ));
        }
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Debug, Clone)]
#[serde(rename_all = "camelCase", from = "String")]
pub enum PackFileHash {
    Sha1,
    Sha512,
    Unknown(String),
}

impl From<String> for PackFileHash {
    fn from(s: String) -> Self {
        match s.as_str() {
            "sha1" => PackFileHash::Sha1,
            "sha512" => PackFileHash::Sha512,
            _ => PackFileHash::Unknown(s),
        }
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum EnvType {
    Client,
    Server,
}

#[derive(Serialize, Deserialize, Clone, Hash, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum PackDependency {
    Forge,
    Neoforge,
    FabricLoader,
    QuiltLoader,
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
            PackDependency::Neoforge => "neoforge",
            PackDependency::FabricLoader => "fabric-loader",
            PackDependency::Minecraft => "minecraft",
            PackDependency::QuiltLoader => "quilt-loader",
        }
    }
}
