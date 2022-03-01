use daedalus::modded::Version as LoaderVersion;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// TODO: possibly add defaults to some of these values
pub const CURRENT_FORMAT_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Profile {
    pub metadata: Metadata,
    pub java: JavaSettings,
    pub memory: MemorySettings,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<WindowSize>,
    pub hooks: ProfileHooks,
}

pub enum IconPath {
    Launcher(String),
    Custom(PathBuf),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Metadata {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<IconPath>,
    pub path: PathBuf,
    pub version: LoaderVersion,
    pub format_version: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JavaSettings {
    pub install: PathBuf,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub extra_arguments: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MemorySettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<String>,
    pub maximum: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WindowSize(u16, u16);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProfileHooks {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_launch: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrapper: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_exit: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_str_eq};

    #[test]
    fn profile_test() -> Result<(), serde_json::Error> {
        let profile = Profile {
            metadata: Metadata {
                name: String::from("Example Pack"),
                icon: None,
                path: PathBuf::from("/tmp/nunya/beeswax"),
                version: LoaderVersion {
                    id: String::from("1.18.2"),
                    loaders: Vec::new(),
                },
                format_version: CURRENT_FORMAT_VERSION,
            },
            java: JavaSettings {
                install: PathBuf::from("/usr/bin/java"),
                extra_arguments: Vec::new(),
            },
            memory: MemorySettings {
                minimum: None,
                maximum: String::from("8192k"),
            },
            resolution: Some(WindowSize(1920, 1080)),
            hooks: ProfileHooks {
                pre_launch: None,
                wrapper: None,
                post_exit: None,
            },
        };
        let json = serde_json::json!({
            "metadata": {
                "name": "Example Pack",
                "path": "/tmp/nunya/beeswax",
                "version": {
                    "id": "1.18.2",
                    "loaders": [],
                },
                "format_version": 1u32,
            },
            "java": {
              "install": "/usr/bin/java",
            },
            "memory": {
              "maximum": "8192k",
            },
            "resolution": (1920u16, 1080u16),
            "hooks": {},
        });

        assert_eq!(serde_json::to_value(profile.clone())?, json.clone());
        assert_str_eq!(
            format!("{:?}", serde_json::from_value::<Profile>(json)?),
            format!("{:?}", profile),
        );
        Ok(())
    }
}
