use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkUpdatePreview {
    pub disable_candidates: Vec<ContentDiffItem>,
    pub disable_paths: Vec<String>,
    pub requires_confirmation: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentDiffItem {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_version_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_version_name: Option<String>,
}
