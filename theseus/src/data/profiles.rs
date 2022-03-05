use super::DataError;
use daedalus::{minecraft::JavaVersion, modded::Version};
use futures::TryFutureExt;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

// TODO: possibly add defaults to some of these values
pub const CURRENT_FORMAT_VERSION: u32 = 1;
pub const SUPPORTED_ICON_FORMATS: &[&'static str] = &[
    "bmp", "gif", "jpeg", "jpg", "jpe", "png", "svg", "svgz", "webp", "rgb", "mp4",
];

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Profile {
    pub metadata: Metadata,
    pub java: JavaSettings,
    pub memory: MemorySettings,
    pub resolution: WindowSize,
    pub hooks: ProfileHooks,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Metadata {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<PathBuf>,
    pub path: PathBuf,
    pub version: Version,
    pub format_version: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JavaSettings {
    pub install: PathBuf,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub extra_arguments: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct MemorySettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<u32>,
    pub maximum: u32,
}

impl Default for MemorySettings {
    fn default() -> Self {
        Self {
            minimum: None,
            maximum: 2048,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct WindowSize(pub u16, pub u16);

impl Default for WindowSize {
    fn default() -> Self {
        Self(854, 480)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProfileHooks {
    #[serde(skip_serializing_if = "HashSet::is_empty", default)]
    pub pre_launch: HashSet<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrapper: Option<String>,
    #[serde(skip_serializing_if = "HashSet::is_empty", default)]
    pub post_exit: HashSet<String>,
}

impl Default for ProfileHooks {
    fn default() -> Self {
        Self {
            pre_launch: HashSet::<String>::new(),
            wrapper: None,
            post_exit: HashSet::<String>::new(),
        }
    }
}

impl Profile {
    pub async fn new(name: String, version: Version, path: PathBuf) -> Result<Self, DataError> {
        let version_id = version.id.clone();
        let (settings, version_info) = futures::try_join! {
            super::Settings::get(),
            super::Metadata::get()
                .and_then(|manifest| async move {
                    let manifest = manifest
                        .minecraft
                        .versions
                        .iter()
                        .find(|it| it.id == version_id)
                        .unwrap();
                    Ok(daedalus::minecraft::fetch_version_info(manifest).await?)
                }),
        }?;

        let java_install = match version_info.java_version {
            Some(JavaVersion { major_version, .. }) if major_version >= 16 => {
                settings.java_17_path.as_ref()
            }
            _ => settings.java_8_path.as_ref(),
        }
        .filter(|it| it.exists())
        .ok_or(DataError::JavaError)?;

        Ok(Self {
            metadata: Metadata {
                name,
                icon: None,
                path,
                version,
                format_version: CURRENT_FORMAT_VERSION,
            },
            java: JavaSettings {
                install: java_install.clone(),
                extra_arguments: settings.custom_java_args.clone(),
            },
            memory: settings.memory.clone(),
            resolution: settings.game_resolution.clone(),
            hooks: settings.hooks.clone(),
        })
    }

    pub async fn run(&self) {
        todo!()
    }

    pub async fn halt(&self) {
        todo!()
    }

    // TODO: deduplicate these builder methods
    // They are flat like this in order to allow builder-style usage
    pub fn with_name(&mut self, name: String) -> &mut Self {
        self.metadata.name = name;
        self
    }

    pub async fn with_icon(&mut self, icon: &Path) -> Result<&mut Self, DataError> {
        let ext = icon
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("");
        if !SUPPORTED_ICON_FORMATS.contains(&ext) {
            Err(DataError::FormatError(format!(
                "Unsupported image type: {ext}"
            )))
        } else {
            let new_path = self.metadata.path.join(format!("icon.{ext}"));
            tokio::fs::copy(icon, &new_path).await?;

            self.metadata.icon = Some(new_path);
            Ok(self)
        }
    }

    pub fn with_version(&mut self, version: Version) -> &mut Self {
        self.metadata.version = version;
        self
    }

    pub fn with_java_install(&mut self, path: PathBuf) -> &mut Self {
        self.java.install = path;
        self
    }

    pub fn with_java_args(&mut self, args: Vec<String>) -> &mut Self {
        self.java.extra_arguments = args;
        self
    }

    pub fn with_minimum_memory(&mut self, memory: u32) -> &mut Self {
        self.memory.minimum = Some(memory);
        self
    }

    pub fn with_maximum_memory(&mut self, memory: u32) -> &mut Self {
        self.memory.maximum = memory;
        self
    }

    pub fn with_resolution(&mut self, resolution: WindowSize) -> &mut Self {
        self.resolution = resolution;
        self
    }

    pub fn with_pre_launch(&mut self, hook: String) -> &mut Self {
        self.hooks.pre_launch.insert(hook);
        self
    }

    pub fn without_pre_launch(&mut self, hook: &str) -> &mut Self {
        self.hooks.pre_launch.remove(hook);
        self
    }

    pub fn with_post_exit(&mut self, hook: String) -> &mut Self {
        self.hooks.post_exit.insert(hook);
        self
    }

    pub fn without_post_exit(&mut self, hook: &str) -> &mut Self {
        self.hooks.post_exit.remove(hook);
        self
    }

    pub fn with_wrapper(&mut self, wrapper: Option<String>) -> &mut Self {
        self.hooks.wrapper = wrapper;
        self
    }
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
                version: Version {
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
                maximum: 8192,
            },
            resolution: WindowSize(1920, 1080),
            hooks: ProfileHooks {
                pre_launch: HashSet::new(),
                wrapper: None,
                post_exit: HashSet::new(),
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
              "maximum": 8192u32,
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
