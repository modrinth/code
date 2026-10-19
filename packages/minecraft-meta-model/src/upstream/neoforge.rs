use derive_more::Display;
use serde::{Deserialize, Serialize};

pub const FORGE_CATALOG_URL: &str = "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/forge";
pub const NEOFORGE_CATALOG_URL: &str = "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Catalog {
    pub is_snapshot: bool,
    pub versions: Vec<VersionName>,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct VersionName(pub String);
