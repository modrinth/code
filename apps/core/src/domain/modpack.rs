use serde::{Deserialize, Serialize};

/// Root of a `.mrpack` index file (`modrinth.index.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackFormat {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format_version: Option<u32>,
    pub game: String,
    pub name: String,
    pub version_id: String,
    pub summary: Option<String>,
    pub files: Vec<PackFile>,
    pub dependencies: std::collections::HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overrides: Vec<PackOverride>,
}

/// A single file entry in a `.mrpack`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackFile {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version_id: Option<String>,
    pub hashes: PackFileHashes,
    pub env: Option<PackFileEnv>,
    pub downloads: Vec<String>,
    pub file_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackFileHashes {
    pub sha1: Option<String>,
    pub sha512: Option<String>,
}

/// Per-side environment requirement for a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackFileEnv {
    pub client: EnvType,
    pub server: EnvType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EnvType {
    Required,
    Optional,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackOverride {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client: Option<PackOverrideSideState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server: Option<PackOverrideSideState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PackOverrideSideState {
    Enabled,
    Disabled,
    Removed,
}

#[cfg(test)]
mod tests {
    use super::{PackFormat, PackOverrideSideState};

    #[test]
    fn parses_manifest_without_overrides() {
        let pack: PackFormat = serde_json::from_value(serde_json::json!({
            "formatVersion": 1,
            "game": "minecraft",
            "versionId": "v1",
            "name": "Pack",
            "files": [],
            "dependencies": {}
        }))
        .unwrap();

        assert_eq!(pack.format_version, Some(1));
        assert!(pack.overrides.is_empty());
    }

    #[test]
    fn parses_and_preserves_top_level_overrides() {
        let pack: PackFormat = serde_json::from_value(serde_json::json!({
            "formatVersion": 1,
            "game": "minecraft",
            "versionId": "v2",
            "name": "Pack",
            "files": [],
            "dependencies": {},
            "overrides": [
                {
                    "projectId": "project-one",
                    "path": "mods/example.jar",
                    "client": "enabled",
                    "server": "disabled"
                }
            ]
        }))
        .unwrap();

        assert_eq!(pack.overrides.len(), 1);
        assert_eq!(
            pack.overrides[0].project_id.as_deref(),
            Some("project-one")
        );
        assert_eq!(pack.overrides[0].path.as_deref(), Some("mods/example.jar"));
        assert_eq!(
            pack.overrides[0].server,
            Some(PackOverrideSideState::Disabled)
        );

        let encoded = serde_json::to_value(pack).unwrap();
        assert_eq!(encoded["overrides"][0]["projectId"], "project-one");
        assert_eq!(encoded["overrides"][0]["server"], "disabled");
    }

    #[test]
    fn ignores_unknown_extra_manifest_fields() {
        let pack: PackFormat = serde_json::from_value(serde_json::json!({
            "formatVersion": 1,
            "game": "minecraft",
            "versionId": "v3",
            "name": "Pack",
            "files": [],
            "dependencies": {},
            "unknownField": { "nested": true }
        }))
        .unwrap();

        assert_eq!(pack.name, "Pack");
    }
}
