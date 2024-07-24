//! Theseus directory information
use crate::state::{JavaVersion, Settings};
use crate::util::fetch::IoSemaphore;
use std::path::{Path, PathBuf};
use tokio::fs;

pub const CACHES_FOLDER_NAME: &str = "caches";
pub const LAUNCHER_LOGS_FOLDER_NAME: &str = "launcher_logs";
pub const PROFILES_FOLDER_NAME: &str = "profiles";
pub const METADATA_FOLDER_NAME: &str = "meta";

#[derive(Debug)]
pub struct DirectoryInfo {
    pub settings_dir: PathBuf, // Base settings directory- settings.json and icon cache.
    pub config_dir: PathBuf, // Base config directory- instances, minecraft downloads, etc. Changeable as a setting.
}

impl DirectoryInfo {
    // Get the settings directory
    // init() is not needed for this function
    pub fn get_initial_settings_dir() -> Option<PathBuf> {
        Self::env_path("THESEUS_CONFIG_DIR")
            .or_else(|| Some(dirs::data_dir()?.join("ModrinthApp")))
    }

    /// Get all paths needed for Theseus to operate properly
    #[tracing::instrument]
    pub async fn init(config_dir: Option<String>) -> crate::Result<Self> {
        let settings_dir = Self::get_initial_settings_dir().ok_or(
            crate::ErrorKind::FSError(
                "Could not find valid settings dir".to_string(),
            ),
        )?;

        fs::create_dir_all(&settings_dir).await.map_err(|err| {
            crate::ErrorKind::FSError(format!(
                "Error creating Theseus config directory: {err}"
            ))
        })?;

        let config_dir = config_dir
            .map(PathBuf::from)
            .unwrap_or_else(|| settings_dir.clone());

        Ok(Self {
            settings_dir,
            config_dir,
        })
    }

    /// Get the Minecraft instance metadata directory
    #[inline]
    pub fn metadata_dir(&self) -> PathBuf {
        self.config_dir.join(METADATA_FOLDER_NAME)
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
        self.config_dir.join(PROFILES_FOLDER_NAME)
    }

    /// Gets the logs dir for a given profile
    #[inline]
    pub fn profile_logs_dir(&self, profile_path: &str) -> PathBuf {
        self.profiles_dir().join(profile_path).join("logs")
    }

    /// Gets the crash reports dir for a given profile
    #[inline]
    pub fn crash_reports_dir(&self, profile_path: &str) -> PathBuf {
        self.profiles_dir().join(profile_path).join("crash-reports")
    }

    #[inline]
    pub fn launcher_logs_dir() -> Option<PathBuf> {
        Self::get_initial_settings_dir()
            .map(|d| d.join(LAUNCHER_LOGS_FOLDER_NAME))
    }

    /// Get the cache directory for Theseus
    #[inline]
    pub fn caches_dir(&self) -> PathBuf {
        self.settings_dir.join(CACHES_FOLDER_NAME)
    }

    /// Get path from environment variable
    #[inline]
    fn env_path(name: &str) -> Option<PathBuf> {
        std::env::var_os(name).map(PathBuf::from)
    }

    pub async fn move_launcher_directory<'a, E>(
        settings: &mut Settings,
        exec: E,
        io_semaphore: &IoSemaphore,
    ) -> crate::Result<()>
    where
        E: sqlx::Executor<'a, Database = sqlx::Sqlite> + Copy,
    {
        if let Some(ref prev_custom_dir) = settings.prev_custom_dir {
            let prev_dir = PathBuf::from(prev_custom_dir);
            let app_dir = DirectoryInfo::get_initial_settings_dir().ok_or(
                crate::ErrorKind::FSError(
                    "Could not find valid config dir".to_string(),
                ),
            )?;

            let move_dir = settings
                .custom_dir
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_else(|| app_dir.clone());

            async fn is_dir_writeable(
                new_config_dir: &Path,
            ) -> crate::Result<bool> {
                let temp_path = new_config_dir.join(".tmp");
                match fs::write(temp_path.clone(), "test").await {
                    Ok(_) => {
                        fs::remove_file(temp_path).await?;
                        Ok(true)
                    }
                    Err(e) => {
                        tracing::error!(
                            "Error writing to new config dir: {}",
                            e
                        );
                        Ok(false)
                    }
                }
            }

            async fn move_directory(
                source: &Path,
                destination: &Path,
                io_semaphore: &IoSemaphore,
            ) -> crate::Result<()> {
                if !source.exists() {
                    crate::util::io::create_dir_all(source).await?;
                }

                if !destination.exists() {
                    crate::util::io::create_dir_all(destination).await?;
                }

                for entry_path in
                    crate::pack::import::get_all_subfiles(source).await?
                {
                    let relative_path = entry_path.strip_prefix(source)?;
                    let new_path = destination.join(relative_path);

                    crate::util::fetch::copy(
                        &entry_path,
                        &new_path,
                        io_semaphore,
                    )
                    .await?;
                }

                Ok(())
            }

            let new_dir = move_dir.to_string_lossy().to_string();

            if prev_dir != move_dir {
                if !is_dir_writeable(&move_dir).await? {
                    settings.custom_dir = Some(prev_custom_dir.clone());

                    return Ok(());
                }

                move_directory(
                    &prev_dir.join(CACHES_FOLDER_NAME),
                    &app_dir.join(CACHES_FOLDER_NAME),
                    io_semaphore,
                )
                .await?;
                move_directory(
                    &prev_dir.join(LAUNCHER_LOGS_FOLDER_NAME),
                    &app_dir.join(LAUNCHER_LOGS_FOLDER_NAME),
                    io_semaphore,
                )
                .await?;

                move_directory(
                    &prev_dir.join(PROFILES_FOLDER_NAME),
                    &move_dir.join(PROFILES_FOLDER_NAME),
                    io_semaphore,
                )
                .await?;
                move_directory(
                    &prev_dir.join(METADATA_FOLDER_NAME),
                    &move_dir.join(METADATA_FOLDER_NAME),
                    io_semaphore,
                )
                .await?;

                let java_versions = JavaVersion::get_all(exec).await?;
                for (_, mut java_version) in java_versions {
                    java_version.path = java_version.path.replace(
                        prev_custom_dir,
                        new_dir.trim_end_matches('/').trim_end_matches('\\'),
                    );
                    java_version.upsert(exec).await?;
                }
            }

            settings.custom_dir = Some(new_dir.clone());
            settings.prev_custom_dir = Some(new_dir);

            settings.update(exec).await?;
        }

        Ok(())
    }
}
