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
    #[tracing::instrument]
    pub async fn init() -> crate::Result<Self> {
        // Working directory
        let working_dir = std::env::current_dir().map_err(|err| {
            crate::ErrorKind::FSError(format!(
                "Could not open working directory: {err}"
            ))
        })?;

        // Config directory
        let config_dir = Self::env_path("THESEUS_CONFIG_DIR")
            .or_else(|| Some(dirs::config_dir()?.join("com.modrinth.theseus")))
            .ok_or(crate::ErrorKind::FSError(
                "Could not find valid config dir".to_string(),
            ))?;

        fs::create_dir_all(&config_dir).await.map_err(|err| {
            crate::ErrorKind::FSError(format!(
                "Error creating Theseus config directory: {err}"
            ))
        })?;

        Ok(Self {
            config_dir,
            working_dir,
        })
    }

    /// Get the Minecraft instance metadata directory
    #[inline]
    pub fn metadata_dir(&self) -> PathBuf {
        self.config_dir.join("meta")
    }

    /// Get the Minecraft java versions metadata directory
    #[inline]
    pub fn java_versions_dir(&self) -> PathBuf {
        self.metadata_dir().join("java_versions")
    }

    /// Get the Minecraft versions metadata directory
    #[inline]
    pub fn versions_dir(&self) -> PathBuf {
        self.metadata_dir().join("versions")
    }

    /// Get the metadata directory for a given version
    #[inline]
    pub fn version_dir(&self, version: &str) -> PathBuf {
        self.versions_dir().join(version)
    }

    /// Get the Minecraft libraries metadata directory
    #[inline]
    pub fn libraries_dir(&self) -> PathBuf {
        self.metadata_dir().join("libraries")
    }

    /// Get the Minecraft assets metadata directory
    #[inline]
    pub fn assets_dir(&self) -> PathBuf {
        self.metadata_dir().join("assets")
    }

    /// Get the assets index directory
    #[inline]
    pub fn assets_index_dir(&self) -> PathBuf {
        self.assets_dir().join("indexes")
    }

    /// Get the assets objects directory
    #[inline]
    pub fn objects_dir(&self) -> PathBuf {
        self.assets_dir().join("objects")
    }

    /// Get the directory for a specific object
    #[inline]
    pub fn object_dir(&self, hash: &str) -> PathBuf {
        self.objects_dir().join(&hash[..2]).join(hash)
    }

    /// Get the Minecraft legacy assets metadata directory
    #[inline]
    pub fn legacy_assets_dir(&self) -> PathBuf {
        self.metadata_dir().join("resources")
    }

    /// Get the Minecraft legacy assets metadata directory
    #[inline]
    pub fn natives_dir(&self) -> PathBuf {
        self.metadata_dir().join("natives")
    }

    /// Get the natives directory for a version of Minecraft
    #[inline]
    pub fn version_natives_dir(&self, version: &str) -> PathBuf {
        self.natives_dir().join(version)
    }

    /// Get the directory containing instance icons
    #[inline]
    pub fn icon_dir(&self) -> PathBuf {
        self.config_dir.join("icons")
    }

    /// Get the profiles directory for created profiles
    #[inline]
    pub fn profiles_dir(&self) -> PathBuf {
        self.config_dir.join("profiles")
    }

    /// Gets the logs dir for a given profile
    #[inline]
    pub fn profile_logs_dir(&self, profile: uuid::Uuid) -> PathBuf {
        self.profiles_dir()
            .join(profile.to_string())
            .join("modrinth_logs")
    }

    /// Get the file containing the global database
    #[inline]
    pub fn database_file(&self) -> PathBuf {
        self.config_dir.join("data.bin")
    }

    /// Get the settings file for Theseus
    #[inline]
    pub fn settings_file(&self) -> PathBuf {
        self.config_dir.join("settings.json")
    }

    /// Get the cache directory for Theseus
    #[inline]
    pub fn caches_dir(&self) -> PathBuf {
        self.config_dir.join("caches")
    }

    #[inline]
    pub fn caches_meta_dir(&self) -> PathBuf {
        self.config_dir.join("caches").join("metadata")
    }

    /// Get path from environment variable
    #[inline]
    fn env_path(name: &str) -> Option<PathBuf> {
        std::env::var_os(name).map(PathBuf::from)
    }
}
