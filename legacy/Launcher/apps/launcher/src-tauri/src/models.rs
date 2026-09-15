use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PacksResponse {
    pub packs: Vec<PackSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackSummary {
    pub id: String,
    pub name: String,
    pub minecraft: String,
    pub loader: String,
    #[serde(default)]
    pub loader_version: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    pub manifest_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackManifest {
    pub id: String,
    pub version: String,
    pub minecraft: String,
    pub loader: String,
    #[serde(default)]
    pub main_class: Option<String>,
    #[serde(default)]
    pub java: Option<JavaRequirement>,
    #[serde(default)]
    pub asset_index: Option<String>,
    pub files: Vec<ManifestFile>,
    #[serde(default)]
    pub jvm_args: Vec<String>,
    #[serde(default)]
    pub game_args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaRequirement {
    pub min_major: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestFile {
    pub path: String,
    pub url: String,
    pub sha256: String,
    #[serde(default)]
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackLocalStatus {
    pub pack_id: String,
    pub installed: bool,
    pub version: Option<String>,
    pub ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncProgress {
    pub pack_id: String,
    pub phase: String,
    pub current_file: Option<String>,
    pub done_files: u32,
    pub total_files: u32,
    pub message: String,
    /// Bytes received for the current file (None when not actively streaming).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_bytes_done: Option<u64>,
    /// Expected / Content-Length total for the current file, if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_bytes_total: Option<u64>,
}
