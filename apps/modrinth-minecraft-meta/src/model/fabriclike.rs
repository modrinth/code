use std::collections::HashMap;

use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use toasty::{Embed, Model};
use url::Url;

use crate::{
    model::{Processor, SidedDataEntry},
    upstream::mojang::{
        Argument, ArgumentType, JavaVersion, Library, LibraryDownloads,
        LibraryExtract, LoggingConfiguration, LoggingSide, OperatingSystem,
        Rule, VersionType,
    },
    util::{MavenCoordinate, Sha256},
};

pub const GAME_VERSION_PLACEHOLDER: &str = "${modrinth.gameVersion}";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Embed)]
pub enum FabriclikeLoader {
    Fabric,
    Quilt,
    Ornithe,
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
    /// Empty for Fabric's universal group; Quilt and Ornithe use named groups.
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
    /// Whether the template game version is replaced with a placeholder.
    ///
    /// Groups for a single game whose upstream name differs from Mojang's keep
    /// the upstream name, since the placeholder would be filled with Mojang's.
    #[serde(default = "default_true")]
    pub templated: bool,
    /// Libraries added to every profile in this group.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_libraries: Vec<FabriclikeLibrary>,
    /// Artifacts (`groupId:artifactId`) of the game's libraries which every
    /// profile in this group drops.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub removed_libraries: Vec<String>,
    /// Java version every profile in this group asks for, in place of the
    /// game's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub java_version: Option<JavaVersion>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabriclikeProfileMetadata {
    pub id: String,
    pub inherits_from: String,
    pub release_time: Timestamp,
    pub time: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_arguments: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<HashMap<ArgumentType, Vec<Argument>>>,
    pub libraries: Vec<FabriclikeLibrary>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub removed_libraries: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub java_version: Option<JavaVersion>,
    #[serde(rename = "type")]
    pub ty: VersionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<HashMap<String, SidedDataEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processors: Option<Vec<Processor>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<HashMap<LoggingSide, LoggingConfiguration>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabriclikeLibrary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloads: Option<LibraryDownloads>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extract: Option<LibraryExtract>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub natives: Option<HashMap<OperatingSystem, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<Rule>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksums: Option<Vec<String>>,
    #[serde(default = "default_true")]
    pub include_in_classpath: bool,
    #[serde(default = "default_true")]
    pub downloadable: bool,
}

fn default_true() -> bool {
    true
}

impl From<Library> for FabriclikeLibrary {
    fn from(library: Library) -> Self {
        Self {
            downloads: library.downloads,
            extract: library.extract,
            name: library.name,
            url: library.url,
            natives: library.natives,
            rules: library.rules,
            checksums: library.checksums,
            include_in_classpath: library.include_in_classpath,
            downloadable: library.downloadable,
        }
    }
}
