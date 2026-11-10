use serde::{Deserialize, Serialize};
use toasty::{Embed, Model};
use url::Url;

use crate::util::{MavenCoordinate, Sha256};

pub use crate::meta::fabriclike::{
    FabriclikeLibrary, FabriclikeProfileMetadata,
};

pub const GAME_VERSION_PLACEHOLDER: &str = "${modrinth.gameVersion}";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Embed)]
pub enum FabriclikeLoader {
    Fabric,
    Quilt,
}

/// Selected games, profile groups, and concrete dependencies for one loader.
#[derive(Debug, Clone, Model)]
pub struct FabriclikeCatalog {
    #[key]
    pub loader: FabriclikeLoader,
    #[column(type = json)]
    pub games: toasty::Json<Vec<FabriclikeGameVersion>>,
    #[column(type = json)]
    pub groups: toasty::Json<Vec<FabriclikeProfileGroup>>,
    #[column(type = json)]
    pub artifacts: toasty::Json<Vec<FabriclikeStoredArtifact>>,
}

/// A normalized loader profile, independent of its public store URL.
#[derive(Debug, Clone, Model)]
#[key(loader, version_group, loader_version)]
pub struct FabriclikeProfile {
    pub loader: FabriclikeLoader,
    /// Empty for Fabric's universal group; Quilt uses named groups.
    pub version_group: String,
    pub loader_version: String,
    pub stable: bool,
    pub position: i32,
    pub source_url: String,
    pub source_sha256: Sha256,
    pub template: String,
    #[column(type = json)]
    pub metadata: toasty::Json<FabriclikeProfileMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabriclikeGameVersion {
    pub id: String,
    pub stable: bool,
}

/// Game versions sharing a profile, including its upstream template provenance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabriclikeProfileGroup {
    pub id: Option<String>,
    pub template: String,
    pub fallback_templates: Vec<String>,
    pub games: Vec<String>,
    pub profile_base_url: Url,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabriclikeArtifact {
    pub coordinate: MavenCoordinate,
    pub repository: Url,
    pub source: Option<String>,
    pub repository_source: String,
}

/// A concrete dependency whose bytes are stored in content-addressed storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabriclikeStoredArtifact {
    #[serde(flatten)]
    pub artifact: FabriclikeArtifact,
    pub sha256: Sha256,
}
