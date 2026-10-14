use indexmap::IndexMap;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use toasty::Model;

use crate::{
    model::{MinecraftLoader, MinecraftVersionName},
    upstream::mojang::{Argument, ArgumentType, Library, VersionType},
};

/// Normalized loader profile to be published as a version JSON artifact.
#[derive(Debug, Clone, Model)]
#[key(loader, minecraft_version, loader_version)]
pub struct Profile {
    pub loader: MinecraftLoader,
    #[index]
    pub minecraft_version: MinecraftVersionName,
    pub loader_version: String,
    #[column(type = json)]
    pub metadata: toasty::Json<ProfileMetadata>,
}

/// Launch metadata combined with optional Forge-style installation metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileMetadata {
    pub id: String,
    pub inherits_from: String,
    #[serde(deserialize_with = "crate::util::deserialize_timestamp")]
    pub release_time: Timestamp,
    #[serde(deserialize_with = "crate::util::deserialize_timestamp")]
    pub time: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_arguments: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<IndexMap<ArgumentType, Vec<Argument>>>,
    pub libraries: Vec<Library>,
    #[serde(rename = "type")]
    pub ty: VersionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<IndexMap<String, SidedDataEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processors: Option<Vec<Processor>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidedDataEntry {
    pub client: String,
    pub server: String,
}

/// Installation step executed by the launcher after downloading libraries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Processor {
    pub jar: String,
    pub classpath: Vec<String>,
    pub args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs: Option<IndexMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sides: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::ProfileMetadata;

    #[test]
    fn independent_profile_deserializations_preserve_nested_map_order() {
        let input = r#"{
			"id": "test-profile",
			"inheritsFrom": "1.21.1",
			"releaseTime": "2024-08-08T00:00:00Z",
			"time": "2024-08-08T00:00:00Z",
			"arguments": {"jvm": ["-Xmx2G"], "game": ["--demo"], "default-user-jvm": []},
			"libraries": [{
				"name": "test:library:1.0",
				"natives": {"windows": "native-windows", "osx": "native-osx", "linux": "native-linux"},
				"downloads": {"classifiers": {
					"z-native": {"sha1": "0000000000000000000000000000000000000000", "size": 1},
					"a-native": {"sha1": "0000000000000000000000000000000000000000", "size": 2},
					"m-native": {"sha1": "0000000000000000000000000000000000000000", "size": 3}
				}}
			}],
			"type": "release",
			"data": {
				"Z_DATA": {"client": "z", "server": "z"},
				"A_DATA": {"client": "a", "server": "a"},
				"M_DATA": {"client": "m", "server": "m"}
			},
			"processors": [{
				"jar": "test:processor:1.0",
				"classpath": [],
				"args": [],
				"outputs": {"z-output": "z", "a-output": "a", "m-output": "m"}
			}]
		}"#;
        let expected = serde_json::to_vec(
            &serde_json::from_str::<ProfileMetadata>(input).unwrap(),
        )
        .unwrap();

        for _ in 0..32 {
            let profile: ProfileMetadata = serde_json::from_str(input).unwrap();
            let bytes = serde_json::to_vec(&profile).unwrap();
            assert_eq!(bytes, expected);
            let json = std::str::from_utf8(&bytes).unwrap();
            for keys in [
                ["jvm", "game", "default-user-jvm"],
                ["windows", "osx", "linux"],
                ["z-native", "a-native", "m-native"],
                ["Z_DATA", "A_DATA", "M_DATA"],
                ["z-output", "a-output", "m-output"],
            ] {
                let positions =
                    keys.map(|key| json.find(&format!("\"{key}\":")).unwrap());
                assert!(
                    positions[0] < positions[1] && positions[1] < positions[2],
                    "source order was not retained for {keys:?}: {json}"
                );
            }
        }
    }
}
