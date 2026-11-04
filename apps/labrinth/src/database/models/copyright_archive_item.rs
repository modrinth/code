use serde::{Deserialize, Serialize};

use crate::models::ids::{CopyrightArchiveId, ProjectId, VersionId};

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CopyrightArchive {
    pub id: CopyrightArchiveId,
    #[serde(flatten)]
    pub meta: CopyrightArchiveMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CopyrightArchiveMeta {
    pub name: String,
    pub modrinth_metadata: Option<CopyrightArchiveModrinthMetadata>,
    pub minecraft_metadata: Option<CopyrightArchiveMinecraftMetadata>,
    pub license: String,
    pub license_terms: Option<CopyrightArchiveLicenseTerms>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CopyrightArchiveModrinthMetadata {
    pub project_id: ProjectId,
    pub version_id: VersionId,
    pub project_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CopyrightArchiveMinecraftMetadata {
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CopyrightArchiveLicenseTerms {
    pub requires_source: Option<String>,
    pub required_redistribution_license: Option<Vec<String>>,
    pub redistribution_allowed: Option<bool>,
    pub commercial_use_allowed: Option<bool>,
    pub attribution_required: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CopyrightArchiveFile {
    pub path: String,
    /// Hex-encoded SHA-512 digest.
    #[serde(with = "hex::serde")]
    #[schema(value_type = String)]
    pub sha512: Vec<u8>,
}
