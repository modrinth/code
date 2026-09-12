//! Theseus directory information
use crate::LoadingBarType;
use crate::event::emit::{emit_loading, init_loading};
use crate::state::LAUNCHER_STATE;
use crate::state::Settings;
use crate::util::fetch::IoSemaphore;
use std::path::PathBuf;
use tokio::fs;

pub const CACHES_FOLDER_NAME: &str = "caches";
pub const LAUNCHER_LOGS_FOLDER_NAME: &str = "launcher_logs";
pub const INSTANCES_FOLDER_NAME: &str = "profiles";
pub const METADATA_FOLDER_NAME: &str = "meta";
pub const SYNCED_OPTIONS_FOLDER_NAME: &str = "synced-options";
pub const STORE_FOLDER_NAME: &str = "store";

#[derive(Clone, Debug)]
pub struct DirectoryInfo {
    pub settings_dir: PathBuf, // Base settings directory- app database
    pub config_dir: PathBuf, // Base config directory- instances, minecraft downloads, etc. Changeable as a setting.
    pub app_identifier: String,
}

impl DirectoryInfo {
    pub fn global_handle_if_ready() -> Option<&'static Self> {
        LAUNCHER_STATE.get().map(|x| &x.directories)
    }

    pub fn get_initial_settings_dir(&self) -> Option<PathBuf> {
        Self::initial_settings_dir_path(&self.app_identifier)
    }

    // Get the settings directory
    // init() is not needed for this function
    pub fn initial_settings_dir_path(app_identifier: &str) -> Option<PathBuf> {
        Self::env_path("THESEUS_CONFIG_DIR")
            .or_else(|| Some(dirs::data_dir()?.join(app_identifier)))
    }

    /// Get all paths needed for Theseus to operate properly
    #[tracing::instrument]
    pub async fn init(
        config_dir: Option<String>,
        app_identifier: &str,
    ) -> crate::Result<Self> {
        let settings_dir = Self::initial_settings_dir_path(app_identifier)
            .ok_or(crate::ErrorKind::FSError(
                "Could not find valid settings dir".to_string(),
            ))?;

        fs::create_dir_all(&settings_dir).await.map_err(|err| {
            crate::ErrorKind::FSError(format!(
                "Error creating Theseus config directory: {err}"
            ))
        })?;

        let config_dir =
            config_dir.map_or_else(|| settings_dir.clone(), PathBuf::from);

        Ok(Self {
            settings_dir,
            config_dir,
            app_identifier: app_identifier.to_owned(),
        })
    }

    pub fn store_dir(&self) -> PathBuf {
        self.config_dir.join(STORE_FOLDER_NAME)
    }

    pub fn content_store_dir(&self) -> PathBuf {
        self.store_dir().join("content")
    }

    pub fn store_staging_dir(&self) -> PathBuf {
        self.store_dir().join("staging")
    }

    /// Get the Minecraft instance metadata directory
    #[inline]
    pub fn metadata_dir(&self) -> PathBuf {
        self.config_dir.join(METADATA_FOLDER_NAME)
    }

    pub fn install_backups_dir(&self) -> PathBuf {
        self.metadata_dir().join("install_job_backups")
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

    /// Get the Minecraft log config's directory
    #[inline]
    pub fn log_configs_dir(&self) -> PathBuf {
        self.metadata_dir().join("log_configs")
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

    /// Get the instances directory
    #[inline]
    pub fn instances_dir(&self) -> PathBuf {
        self.config_dir.join(INSTANCES_FOLDER_NAME)
    }

    #[inline]
    pub fn synced_options_dir(&self) -> PathBuf {
        self.config_dir.join(SYNCED_OPTIONS_FOLDER_NAME)
    }

    /// Gets the logs dir for a given instance path
    #[inline]
    pub fn instance_logs_dir(&self, instance_path: &str) -> PathBuf {
        self.instances_dir().join(instance_path).join("logs")
    }

    /// Gets the crash reports dir for a given instance path
    #[inline]
    pub fn crash_reports_dir(&self, instance_path: &str) -> PathBuf {
        self.instances_dir()
            .join(instance_path)
            .join("crash-reports")
    }

    #[inline]
    pub fn launcher_logs_dir(&self) -> Option<PathBuf> {
        self.get_initial_settings_dir()
            .map(|d| d.join(LAUNCHER_LOGS_FOLDER_NAME))
    }

    #[inline]
    pub fn launcher_logs_dir_path(app_identifier: &str) -> Option<PathBuf> {
        Self::initial_settings_dir_path(app_identifier)
            .map(|d| d.join(LAUNCHER_LOGS_FOLDER_NAME))
    }

    /// Get the cache directory for Theseus
    #[inline]
    pub fn caches_dir(&self) -> PathBuf {
        self.config_dir.join(CACHES_FOLDER_NAME)
    }

    /// Get path from environment variable
    #[inline]
    fn env_path(name: &str) -> Option<PathBuf> {
        std::env::var_os(name).map(PathBuf::from)
    }

    #[tracing::instrument(skip(settings, pool, _io_semaphore))]
    pub async fn move_launcher_directory(
        settings: &mut Settings,
        pool: &sqlx::SqlitePool,
        _io_semaphore: &IoSemaphore,
        app_identifier: &str,
    ) -> crate::Result<()> {
        let initial = DirectoryInfo::initial_settings_dir_path(app_identifier)
            .ok_or_else(|| {
                crate::ErrorKind::FSError(
                    "Could not find the app directory".to_string(),
                )
            })?;
        let destination = settings
            .custom_dir
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| initial.clone());
        let previous = settings.prev_custom_dir.as_ref().map(PathBuf::from);
        let settings_root = fs::canonicalize(&initial).await?;
        let mut locked_roots = std::collections::HashSet::from([settings_root]);
        let mut move_locks = Vec::new();
        for root in previous.iter().chain(std::iter::once(&destination)) {
            fs::create_dir_all(root).await?;
            let root = fs::canonicalize(root).await?;
            if locked_roots.insert(root.clone()) {
                move_locks.push(
                    super::content_store::ContentStore::lock_process(&root)
                        .await?,
                );
            }
        }
        if let Some(previous) = &previous
            && previous != &destination
        {
            let loading = init_loading(
                LoadingBarType::DirectoryMove {
                    old: previous.to_string_lossy().into_owned(),
                    new: destination.to_string_lossy().into_owned(),
                },
                100.0,
                "Moving launcher directory",
            )
            .await?;
            super::content_store::migration::move_app_directory(
                previous,
                &destination,
                pool,
            )
            .await?;
            emit_loading(&loading, 100.0, None)?;
        }
        if previous
            .as_ref()
            .is_some_and(|previous| previous == &destination)
        {
            super::content_store::migration::resume_completed_move(
                &destination,
                pool,
            )
            .await?;
        }
        settings.custom_dir = Some(destination.to_string_lossy().into_owned());
        settings.prev_custom_dir.clone_from(&settings.custom_dir);
        settings.update(pool).await?;
        if let Some(previous) = &previous
            && previous != &destination
        {
            super::content_store::migration::finish_app_directory_move(
                previous,
                &destination,
                pool,
            )
            .await?;
        }
        drop(move_locks);
        Ok(())
    }
}
