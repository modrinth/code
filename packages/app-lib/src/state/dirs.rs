//! Theseus directory information
use crate::event::emit::{emit_loading, init_loading};
use crate::state::{JavaVersion, Profile, Settings};
use crate::util::fetch::IoSemaphore;
use crate::LoadingBarType;
use dashmap::DashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;

pub const CACHES_FOLDER_NAME: &str = "caches";
pub const LAUNCHER_LOGS_FOLDER_NAME: &str = "launcher_logs";
pub const PROFILES_FOLDER_NAME: &str = "profiles";
pub const METADATA_FOLDER_NAME: &str = "meta";

#[derive(Debug)]
pub struct DirectoryInfo {
    pub settings_dir: PathBuf, // Base settings directory- app database
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
        self.config_dir.join(CACHES_FOLDER_NAME)
    }

    /// Get path from environment variable
    #[inline]
    fn env_path(name: &str) -> Option<PathBuf> {
        std::env::var_os(name).map(PathBuf::from)
    }

    #[tracing::instrument(skip(settings, exec, io_semaphore))]
    pub async fn move_launcher_directory<'a, E>(
        settings: &mut Settings,
        exec: E,
        io_semaphore: &IoSemaphore,
    ) -> crate::Result<()>
    where
        E: sqlx::Executor<'a, Database = sqlx::Sqlite> + Copy,
    {
        let app_dir = DirectoryInfo::get_initial_settings_dir().ok_or(
            crate::ErrorKind::FSError(
                "Could not find valid config dir".to_string(),
            ),
        )?;

        if let Some(ref prev_custom_dir) = settings.prev_custom_dir {
            let prev_dir = PathBuf::from(prev_custom_dir);

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

            fn is_same_disk(
                old_dir: &Path,
                new_dir: &Path,
            ) -> crate::Result<bool> {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::MetadataExt;
                    Ok(old_dir.metadata()?.dev() == new_dir.metadata()?.dev())
                }

                #[cfg(windows)]
                {
                    let old_dir = crate::util::io::canonicalize(old_dir)?;
                    let new_dir = crate::util::io::canonicalize(new_dir)?;

                    let old_component = old_dir.components().next();
                    let new_component = new_dir.components().next();

                    match (old_component, new_component) {
                        (
                            Some(std::path::Component::Prefix(old)),
                            Some(std::path::Component::Prefix(new)),
                        ) => Ok(old.as_os_str() == new.as_os_str()),
                        _ => Ok(false),
                    }
                }
            }

            fn get_disk_usage(path: &Path) -> crate::Result<Option<u64>> {
                let path = crate::util::io::canonicalize(path)?;

                let disks = sysinfo::Disks::new_with_refreshed_list();

                for disk in disks.iter() {
                    if path.starts_with(disk.mount_point()) {
                        return Ok(Some(disk.available_space()));
                    }
                }

                Ok(None)
            }

            let new_dir = move_dir.to_string_lossy().to_string();

            if prev_dir != move_dir {
                let loader_bar_id = init_loading(
                    LoadingBarType::DirectoryMove {
                        old: prev_dir.clone(),
                        new: move_dir.clone(),
                    },
                    100.0,
                    "Moving launcher directory",
                )
                .await?;

                if !is_dir_writeable(&move_dir).await? {
                    return Err(crate::ErrorKind::DirectoryMoveError(format!("Cannot move directory to {}: directory is not writeable", move_dir.display())).into());
                }

                const MOVE_DIRS: &[&str] = &[
                    CACHES_FOLDER_NAME,
                    PROFILES_FOLDER_NAME,
                    METADATA_FOLDER_NAME,
                ];

                struct MovePath {
                    old: PathBuf,
                    new: PathBuf,
                    size: u64,
                }

                async fn add_paths(
                    source: &Path,
                    destination: &Path,
                    paths: &mut Vec<MovePath>,
                    total_size: &mut u64,
                ) -> crate::Result<()> {
                    if !source.exists() {
                        crate::util::io::create_dir_all(source).await?;
                    }

                    if !destination.exists() {
                        crate::util::io::create_dir_all(destination).await?;
                    }

                    for entry_path in
                        crate::pack::import::get_all_subfiles(source, false)
                            .await?
                    {
                        let relative_path = entry_path.strip_prefix(source)?;
                        let new_path = destination.join(relative_path);
                        let path_size =
                            entry_path.metadata().map(|x| x.len()).unwrap_or(0);

                        *total_size += path_size;

                        paths.push(MovePath {
                            old: entry_path,
                            new: new_path,
                            size: path_size,
                        });
                    }

                    Ok(())
                }

                let mut paths: Vec<MovePath> = vec![];
                let mut total_size = 0;

                for dir in MOVE_DIRS {
                    add_paths(
                        &prev_dir.join(dir),
                        &move_dir.join(dir),
                        &mut paths,
                        &mut total_size,
                    )
                    .await?;
                    emit_loading(
                        &loader_bar_id,
                        10.0 / (MOVE_DIRS.len() as f64),
                        None,
                    )?;
                }

                let paths_len = paths.len();

                if is_same_disk(&prev_dir, &move_dir).unwrap_or(false) {
                    let success_idxs = Arc::new(DashSet::new());

                    let loader_bar_id = Arc::new(&loader_bar_id);
                    let res =
                        futures::future::try_join_all(paths.iter().enumerate().map(|(idx, x)| {
                            let loader_bar_id = loader_bar_id.clone();
                            let success_idxs = success_idxs.clone();

                            async move {
                                let _permit = io_semaphore.0.acquire().await?;

                                if let Some(parent) = x.new.parent() {
                                    crate::util::io::create_dir_all(parent).await.map_err(|e| {
                                        crate::Error::from(crate::ErrorKind::DirectoryMoveError(
                                            format!(
                                                "Failed to create directory {}: {}",
                                                parent.display(),
                                                e
                                            )
                                        ))
                                    })?;
                                }

                                crate::util::io::rename(
                                    &x.old,
                                    &x.new,
                                )
                                .await
                                    .map_err(|e| {
                                        crate::Error::from(crate::ErrorKind::DirectoryMoveError(
                                            format!(
                                                "Failed to move directory from {} to {}: {}",
                                                x.old.display(),
                                                x.new.display(),
                                                e
                                            ),
                                        ))
                                    })?;

                                let _ = emit_loading(
                                    &loader_bar_id,
                                    90.0 / paths_len as f64,
                                    None,
                                );

                                success_idxs.insert(idx);

                                Ok::<(), crate::Error>(())
                            }
                        }))
                        .await;

                    if let Err(e) = res {
                        for idx in success_idxs.iter() {
                            let path = &paths[*idx.key()];

                            let res =
                                tokio::fs::rename(&path.new, &path.old).await;

                            if let Err(e) = res {
                                tracing::warn!(
                                    "Failed to rollback directory {}: {}",
                                    path.new.display(),
                                    e
                                );
                            }
                        }

                        return Err(e);
                    }
                } else {
                    if let Some(disk_usage) = get_disk_usage(&move_dir)? {
                        if total_size > disk_usage {
                            return Err(crate::ErrorKind::DirectoryMoveError(format!("Not enough space to move directory to {}: only {} bytes available", app_dir.display(), disk_usage)).into());
                        }
                    }

                    let loader_bar_id = Arc::new(&loader_bar_id);
                    futures::future::try_join_all(paths.iter().map(|x| {
                        let loader_bar_id = loader_bar_id.clone();

                        async move {
                            crate::util::fetch::copy(
                                &x.old,
                                &x.new,
                                io_semaphore,
                            )
                            .await.map_err(|e| { crate::Error::from(
                                crate::ErrorKind::DirectoryMoveError(format!("Failed to move directory from {} to {}: {}", x.old.display(), x.new.display(), e)))
                            })?;

                            let _ = emit_loading(
                                &loader_bar_id,
                                ((x.size as f64) / (total_size as f64)) * 60.0,
                                None,
                            );

                            Ok::<(), crate::Error>(())
                        }
                    }))
                    .await?;

                    futures::future::join_all(paths.iter().map(|x| {
                        let loader_bar_id = loader_bar_id.clone();

                        async move {
                            let res = async {
                                let _permit = io_semaphore.0.acquire().await?;
                                crate::util::io::remove_file(&x.old).await?;

                                emit_loading(
                                    &loader_bar_id,
                                    30.0 / paths_len as f64,
                                    None,
                                )?;

                                Ok::<(), crate::Error>(())
                            };

                            if let Err(e) = res.await {
                                tracing::warn!(
                                    "Failed to remove old file {}: {}",
                                    x.old.display(),
                                    e
                                );
                            }
                        }
                    }))
                    .await;
                }

                let java_versions = JavaVersion::get_all(exec).await?;
                for (_, mut java_version) in java_versions {
                    java_version.path = java_version.path.replace(
                        prev_custom_dir,
                        new_dir.trim_end_matches('/').trim_end_matches('\\'),
                    );
                    java_version.upsert(exec).await?
                }

                let profiles = Profile::get_all(exec).await?;

                for mut profile in profiles {
                    profile.icon_path = profile.icon_path.map(|x| {
                        x.replace(
                            prev_custom_dir,
                            new_dir
                                .trim_end_matches('/')
                                .trim_end_matches('\\'),
                        )
                    });
                    profile.java_path = profile.java_path.map(|x| {
                        x.replace(
                            prev_custom_dir,
                            new_dir
                                .trim_end_matches('/')
                                .trim_end_matches('\\'),
                        )
                    });
                    profile.upsert(exec).await?;
                }
            }

            settings.custom_dir = Some(new_dir);
        }

        settings.prev_custom_dir.clone_from(&settings.custom_dir);
        if settings.custom_dir.is_none() {
            settings.custom_dir = Some(app_dir.to_string_lossy().to_string());
        }

        settings.update(exec).await?;

        Ok(())
    }
}
