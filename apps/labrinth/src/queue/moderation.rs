use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MissingMetadata {
    pub identified: HashMap<String, IdentifiedFile>,
    pub flame_files: HashMap<String, MissingMetadataFlame>,
    pub unknown_files: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct IdentifiedFile {
    pub file_name: String,
    pub status: ApprovalType,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MissingMetadataFlame {
    pub title: String,
    pub file_name: String,
    pub url: String,
    pub id: u32,
}

#[derive(
    Deserialize, Serialize, Copy, Clone, PartialEq, Eq, Debug, utoipa::ToSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum ApprovalType {
    Yes,
    WithAttributionAndSource,
    WithAttribution,
    No,
    PermanentNo,
    Unidentified,
}

impl ApprovalType {
    pub fn approved(&self) -> bool {
        match self {
            ApprovalType::Yes => true,
            ApprovalType::WithAttributionAndSource => true,
            ApprovalType::WithAttribution => true,
            ApprovalType::No => false,
            ApprovalType::PermanentNo => false,
            ApprovalType::Unidentified => false,
        }
    }

    pub fn from_string(string: &str) -> Option<Self> {
        match string {
            "yes" => Some(ApprovalType::Yes),
            "with-attribution-and-source" => {
                Some(ApprovalType::WithAttributionAndSource)
            }
            "with-attribution" => Some(ApprovalType::WithAttribution),
            "no" => Some(ApprovalType::No),
            "permanent-no" => Some(ApprovalType::PermanentNo),
            "unidentified" => Some(ApprovalType::Unidentified),
            _ => None,
        }
    }

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            ApprovalType::Yes => "yes",
            ApprovalType::WithAttributionAndSource => {
                "with-attribution-and-source"
            }
            ApprovalType::WithAttribution => "with-attribution",
            ApprovalType::No => "no",
            ApprovalType::PermanentNo => "permanent-no",
            ApprovalType::Unidentified => "unidentified",
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct FlameResponse<T> {
    pub data: T,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FingerprintResponse {
    pub exact_matches: Vec<FingerprintMatch>,
}

#[derive(Deserialize, Serialize)]
pub struct FingerprintMatch {
    pub id: u32,
    pub file: FlameFile,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlameFile {
    pub id: u32,
    pub mod_id: u32,
    pub hashes: Vec<FlameFileHash>,
    pub file_fingerprint: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FlameFileHash {
    pub value: String,
    pub algo: u32,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlameProjectResponse {
    pub id: u32,
    pub name: String,
    pub slug: String,
    pub links: FlameLinks,
    pub logo: FlameLogo,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlameLogo {
    pub thumbnail_url: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlameLinks {
    pub website_url: String,
}
