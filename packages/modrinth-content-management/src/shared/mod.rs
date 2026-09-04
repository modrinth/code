//! Content identifiers and errors shared by content-management features.

use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("unknown content type `{0}`")]
    UnknownContentType(String),
    #[error(
        "project `{project_id}` has conflicting versions `{before}` and `{after}`"
    )]
    ConflictingProjectVersions {
        project_id: String,
        before: String,
        after: String,
    },
    #[error("metadata provider error: {0}")]
    Provider(String),
    #[error("project `{0}` was not found")]
    ProjectNotFound(String),
    #[error("version `{0}` was not found")]
    VersionNotFound(String),
    #[error("version `{version_id}` does not belong to project `{project_id}`")]
    VersionProjectMismatch {
        version_id: String,
        project_id: String,
    },
    #[error("no compatible version was found for project `{0}`")]
    NoCompatibleVersion(String),
}

#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Mod,
    Plugin,
    DataPack,
    ResourcePack,
    Shader,
    ModPack,
}

impl std::str::FromStr for ContentType {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "mod" => Ok(Self::Mod),
            "plugin" => Ok(Self::Plugin),
            "datapack" => Ok(Self::DataPack),
            "resourcepack" => Ok(Self::ResourcePack),
            "shader" => Ok(Self::Shader),
            "modpack" => Ok(Self::ModPack),
            _ => Err(Error::UnknownContentType(value.to_owned())),
        }
    }
}

impl ContentType {
    /// Returns the canonical content type used by shared-instance manifests.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mod => "mod",
            Self::Plugin => "plugin",
            Self::DataPack => "datapack",
            Self::ResourcePack => "resourcepack",
            Self::Shader => "shader",
            Self::ModPack => "modpack",
        }
    }
}
