//! Theseus directory information
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug)]
pub struct DirectoryInfo {
    pub config_dir: PathBuf,
    pub working_dir: PathBuf,
}

impl DirectoryInfo {
    /// Get all paths needed for Theseus to operate properly
    pub async fn init() -> crate::Result<Self> {
        // Working directory
        let working_dir = std::env::current_dir().map_err(|err| {
            crate::Error::FSError(format!(
                "Could not open working directory: {err}"
            ))
        })?;

        // Config directory
        let config_dir = Self::env_path("THESEUS_CONFIG_DIR")
            .or_else(|| Some(dirs::config_dir()?.join("theseus")))
            .ok_or(crate::Error::FSError(
                "Could not find valid config dir".to_string(),
            ))?;

        fs::create_dir_all(&config_dir).await.map_err(|err| {
            crate::Error::FSError(format!(
                "Error creating Theseus config directory: {err}"
            ))
        })?;

        Ok(Self {
            config_dir,
            working_dir,
        })
    }

    /// Get the Minecraft instance metadata directory
    pub fn metadata_dir(&self) -> PathBuf {
        self.config_dir.join("meta")
    }

    /// Get the Minecraft versions metadata directory
    pub fn versions_dir(&self) -> PathBuf {
        self.metadata_dir().join("versions")
    }

    /// Get the metadata directory for a given version
    pub fn version_dir(&self, version: &str) -> PathBuf {
        self.versions_dir().join(version)
    }

    /// Get the Minecraft libraries metadata directory
    pub fn libraries_dir(&self) -> PathBuf {
        self.metadata_dir().join("libraries")
    }

    /// Get the Minecraft assets metadata directory
    pub fn assets_dir(&self) -> PathBuf {
        self.metadata_dir().join("assets")
    }

    /// Get the assets index directory
    pub fn assets_index_dir(&self) -> PathBuf {
        self.assets_dir().join("indexes")
    }

    /// Get the assets objects directory
    pub fn objects_dir(&self) -> PathBuf {
        self.assets_dir().join("objects")
    }

    /// Get the directory for a specific object
    pub fn object_dir(&self, hash: &str) -> PathBuf {
        self.objects_dir().join(&hash[..2]).join(hash)
    }

    /// Get the Minecraft legacy assets metadata directory
    pub fn legacy_assets_dir(&self) -> PathBuf {
        self.metadata_dir().join("resources")
    }

    /// Get the Minecraft legacy assets metadata directory
    pub fn natives_dir(&self) -> PathBuf {
        self.metadata_dir().join("natives")
    }

    /// Get the natives directory for a version of Minecraft
    pub fn version_natives_dir(&self, version: &str) -> PathBuf {
        self.natives_dir().join(version)
    }

    /// Get the directory containing instance icons
    pub fn icon_dir(&self) -> PathBuf {
        self.config_dir.join("icons")
    }

    /// Get the file containing the global database
    pub fn database_file(&self) -> PathBuf {
        self.config_dir.join("data.bin")
    }

    /// Get the settings file for Theseus
    pub fn settings_file(&self) -> PathBuf {
        self.config_dir.join("settings.json")
    }

    /// Get path from environment variable
    fn env_path(name: &str) -> Option<PathBuf> {
        std::env::var_os(name).map(PathBuf::from)
    }
}
