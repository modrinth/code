use super::settings::{Hooks, MemorySettings, WindowSize};
use crate::config::MODRINTH_API_URL;
use crate::data::DirectoryInfo;
use crate::event::emit::{emit_profile, emit_warning};
use crate::event::ProfilePayloadType;
use crate::prelude::JavaVersion;
use crate::state::projects::Project;
use crate::state::{ModrinthVersion, ProjectMetadata, ProjectType};
use crate::util::fetch::{
    fetch, fetch_json, write, write_cached_icon, IoSemaphore,
};
use crate::util::io::{self, IOError};
use crate::State;
use chrono::{DateTime, Utc};
use daedalus::get_hash;
use daedalus::modded::LoaderVersion;
use futures::prelude::*;
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::Debouncer;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use uuid::Uuid;

const PROFILE_JSON_PATH: &str = "profile.json";

pub(crate) struct Profiles(pub HashMap<ProfilePathId, Profile>);

#[derive(
    Serialize, Deserialize, Clone, Copy, Debug, Default, Eq, PartialEq,
)]
#[serde(rename_all = "snake_case")]
pub enum ProfileInstallStage {
    /// Profile is installed
    Installed,
    /// Profile's minecraft game is still installing
    Installing,
    /// Profile created for pack, but the pack hasn't been fully installed yet
    PackInstalling,
    /// Profile is not installed
    #[default]
    NotInstalled,
}

/// newtype wrapper over a Profile path, to be usable as a clear identifier for the kind of path used
/// eg: for "a/b/c/profiles/My Mod", the ProfilePathId would be "My Mod" (a relative path)
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
#[serde(transparent)]
pub struct ProfilePathId(PathBuf);
impl ProfilePathId {
    // Create a new ProfilePathId from a full file path
    pub async fn from_fs_path(path: PathBuf) -> crate::Result<Self> {
        let path: PathBuf = io::canonicalize(path)?;
        let profiles_dir = io::canonicalize(
            State::get().await?.directories.profiles_dir().await,
        )?;
        path.strip_prefix(profiles_dir)
            .ok()
            .and_then(|p| p.file_name())
            .ok_or_else(|| {
                crate::ErrorKind::FSError(format!(
                    "Path {path:?} does not correspond to a profile",
                    path = path
                ))
            })?;
        Ok(Self(path))
    }

    // Create a new ProfilePathId from a relative path
    pub fn new(path: &Path) -> Self {
        ProfilePathId(PathBuf::from(path))
    }

    pub async fn get_full_path(&self) -> crate::Result<PathBuf> {
        let state = State::get().await?;
        let profiles_dir = state.directories.profiles_dir().await;
        Ok(profiles_dir.join(&self.0))
    }

    pub fn check_valid_utf(&self) -> crate::Result<&Self> {
        self.0
            .to_str()
            .ok_or(crate::ErrorKind::UTFError(self.0.clone()).as_error())?;
        Ok(self)
    }
}
impl std::fmt::Display for ProfilePathId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.display().fmt(f)
    }
}

/// newtype wrapper over a Profile path, to be usable as a clear identifier for the kind of path used
/// eg: for "a/b/c/profiles/My Mod/mods/myproj", the ProjectPathId would be "mods/myproj"
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
#[serde(transparent)]
pub struct ProjectPathId(pub PathBuf);
impl ProjectPathId {
    // Create a new ProjectPathId from a full file path
    pub async fn from_fs_path(path: PathBuf) -> crate::Result<Self> {
        let path: PathBuf = io::canonicalize(path)?;
        let profiles_dir: PathBuf = io::canonicalize(
            State::get().await?.directories.profiles_dir().await,
        )?;
        path.strip_prefix(profiles_dir)
            .ok()
            .map(|p| p.components().skip(1).collect::<PathBuf>())
            .ok_or_else(|| {
                crate::ErrorKind::FSError(format!(
                    "Path {path:?} does not correspond to a profile",
                    path = path
                ))
            })?;
        Ok(Self(path))
    }

    pub async fn get_full_path(
        &self,
        profile: ProfilePathId,
    ) -> crate::Result<PathBuf> {
        let _state = State::get().await?;
        let profile_dir = profile.get_full_path().await?;
        Ok(profile_dir.join(&self.0))
    }

    // Create a new ProjectPathId from a relative path
    pub fn new(path: &Path) -> Self {
        ProjectPathId(PathBuf::from(path))
    }
}

// Represent a Minecraft instance.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Profile {
    pub uuid: Uuid, // todo: will be used in restructure to refer to profiles
    #[serde(default)]
    pub install_stage: ProfileInstallStage,
    #[serde(default)]
    pub path: PathBuf, // Relative path to the profile, to be used in ProfilePathId
    pub metadata: ProfileMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub java: Option<JavaSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<MemorySettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<WindowSize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks: Option<Hooks>,
    pub projects: HashMap<ProjectPathId, Project>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProfileMetadata {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub groups: Vec<String>,

    pub game_version: String,
    #[serde(default)]
    pub loader: ModLoader,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loader_version: Option<LoaderVersion>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_data: Option<LinkedData>,

    #[serde(default)]
    pub date_created: DateTime<Utc>,
    #[serde(default)]
    pub date_modified: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_played: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LinkedData {
    pub project_id: Option<String>,
    pub version_id: Option<String>,
}

#[derive(
    Debug, Eq, PartialEq, Clone, Copy, Deserialize, Serialize, Default,
)]
#[serde(rename_all = "lowercase")]
pub enum ModLoader {
    #[default]
    Vanilla,
    Forge,
    Fabric,
    Quilt,
}

impl std::fmt::Display for ModLoader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match *self {
            Self::Vanilla => "Vanilla",
            Self::Forge => "Forge",
            Self::Fabric => "Fabric",
            Self::Quilt => "Quilt",
        })
    }
}

impl ModLoader {
    pub(crate) fn as_api_str(&self) -> &'static str {
        match *self {
            Self::Vanilla => "vanilla",
            Self::Forge => "forge",
            Self::Fabric => "fabric",
            Self::Quilt => "quilt",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JavaSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_version: Option<JavaVersion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_arguments: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_env_args: Option<Vec<(String, String)>>,
}

impl Profile {
    #[tracing::instrument]
    pub async fn new(
        uuid: Uuid,
        name: String,
        version: String,
    ) -> crate::Result<Self> {
        if name.trim().is_empty() {
            return Err(crate::ErrorKind::InputError(String::from(
                "Empty name for instance!",
            ))
            .into());
        }

        Ok(Self {
            uuid,
            install_stage: ProfileInstallStage::NotInstalled,
            path: PathBuf::new().join(&name),
            metadata: ProfileMetadata {
                name,
                icon: None,
                icon_url: None,
                groups: vec![],
                game_version: version,
                loader: ModLoader::Vanilla,
                loader_version: None,
                linked_data: None,
                date_created: Utc::now(),
                date_modified: Utc::now(),
                last_played: None,
            },
            projects: HashMap::new(),
            java: None,
            memory: None,
            resolution: None,
            hooks: None,
        })
    }

    // Gets the ProfilePathId for this profile
    #[inline]
    pub fn profile_id(&self) -> ProfilePathId {
        ProfilePathId::new(&self.path)
    }

    #[tracing::instrument(skip(self, semaphore, icon))]
    pub async fn set_icon<'a>(
        &'a mut self,
        cache_dir: &Path,
        semaphore: &IoSemaphore,
        icon: bytes::Bytes,
        file_name: &str,
    ) -> crate::Result<()> {
        let file =
            write_cached_icon(file_name, cache_dir, icon, semaphore).await?;
        self.metadata.icon = Some(file);
        self.metadata.date_modified = Utc::now();
        Ok(())
    }

    pub fn crash_task(path: ProfilePathId) {
        tokio::task::spawn(async move {
            let res = async {
                let profile = crate::api::profile::get(&path, None).await?;

                if let Some(profile) = profile {
                    emit_warning(&format!("Profile {} has crashed! Visit the logs page to see a crash report.", profile.metadata.name)).await?;
                }

                Ok::<(), crate::Error>(())
            }
                .await;

            match res {
                Ok(()) => {}
                Err(err) => {
                    tracing::warn!(
                        "Unable to send crash report to frontend: {err}"
                    )
                }
            };
        });
    }

    pub fn sync_projects_task(profile_path_id: ProfilePathId) {
        tokio::task::spawn(async move {
            let span =
                tracing::span!(tracing::Level::INFO, "sync_projects_task");
            tracing::debug!(
                parent: &span,
                "Syncing projects for profile {}",
                profile_path_id
            );
            let res = async {
                let _span = span.enter();
                let state = State::get().await?;
                let profile = crate::api::profile::get(&profile_path_id, None).await?;

                if let Some(profile) = profile {
                    let paths = profile.get_profile_full_project_paths().await?;

                    let caches_dir = state.directories.caches_dir();
                    let projects = crate::state::infer_data_from_files(
                        profile.clone(),
                        paths,
                        caches_dir,
                        &state.io_semaphore,
                        &state.fetch_semaphore,
                    )
                    .await?;

                    let mut new_profiles = state.profiles.write().await;
                    if let Some(profile) = new_profiles.0.get_mut(&profile_path_id) {
                        profile.projects = projects;
                    }
                    emit_profile(
                        profile.uuid,
                        profile.get_profile_full_path().await?,
                        &profile.metadata.name,
                        ProfilePayloadType::Synced,
                    )
                    .await?;
                } else {
                    tracing::warn!(
                        "Unable to fetch single profile projects: path {profile_path_id} invalid",
                    );
                }
                Ok::<(), crate::Error>(())
            }.await;
            match res {
                Ok(()) => {}
                Err(err) => {
                    tracing::warn!(
                        "Unable to fetch single profile projects: {err}"
                    )
                }
            };
        });
    }

    // Get full path to profile
    pub async fn get_profile_full_path(&self) -> crate::Result<PathBuf> {
        let state = State::get().await?;
        let profiles_dir = state.directories.profiles_dir().await;
        Ok(profiles_dir.join(&self.path))
    }

    /// Gets paths to projects as their full paths, not just their relative paths
    pub async fn get_profile_full_project_paths(
        &self,
    ) -> crate::Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let profile_path = self.get_profile_full_path().await?;
        let mut read_paths = |path: &str| {
            let new_path = profile_path.join(path);
            if new_path.exists() {
                let path = self.path.join(path);
                for path in std::fs::read_dir(&path)
                    .map_err(|e| IOError::with_path(e, &path))?
                {
                    let path = path.map_err(IOError::from)?.path();
                    if path.is_file() {
                        files.push(path);
                    }
                }
            }
            Ok::<(), crate::Error>(())
        };

        read_paths(ProjectType::Mod.get_folder())?;
        read_paths(ProjectType::ShaderPack.get_folder())?;
        read_paths(ProjectType::ResourcePack.get_folder())?;
        read_paths(ProjectType::DataPack.get_folder())?;

        Ok(files)
    }

    #[tracing::instrument(skip(watcher))]
    #[theseus_macros::debug_pin]
    pub async fn watch_fs(
        profile_path: &Path,
        watcher: &mut Debouncer<RecommendedWatcher>,
    ) -> crate::Result<()> {
        async fn watch_path(
            profile_path: &Path,
            watcher: &mut Debouncer<RecommendedWatcher>,
            path: &str,
        ) -> crate::Result<()> {
            let path = profile_path.join(path);

            io::create_dir_all(&path).await?;

            watcher
                .watcher()
                .watch(&profile_path.join(path), RecursiveMode::Recursive)?;

            Ok(())
        }

        watch_path(profile_path, watcher, ProjectType::Mod.get_folder())
            .await?;
        watch_path(profile_path, watcher, ProjectType::ShaderPack.get_folder())
            .await?;
        watch_path(
            profile_path,
            watcher,
            ProjectType::ResourcePack.get_folder(),
        )
        .await?;
        watch_path(profile_path, watcher, ProjectType::DataPack.get_folder())
            .await?;
        watch_path(profile_path, watcher, "crash-reports").await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    #[theseus_macros::debug_pin]
    pub async fn add_project_version(
        &self,
        version_id: String,
    ) -> crate::Result<(ProjectPathId, ModrinthVersion)> {
        let state = State::get().await?;

        let version = fetch_json::<ModrinthVersion>(
            Method::GET,
            &format!("{MODRINTH_API_URL}version/{version_id}"),
            None,
            None,
            &state.fetch_semaphore,
        )
        .await?;

        let file = if let Some(file) = version.files.iter().find(|x| x.primary)
        {
            file
        } else if let Some(file) = version.files.first() {
            file
        } else {
            return Err(crate::ErrorKind::InputError(
                "No files for input version present!".to_string(),
            )
            .into());
        };

        let bytes = fetch(
            &file.url,
            file.hashes.get("sha1").map(|x| &**x),
            &state.fetch_semaphore,
        )
        .await?;

        let path = self
            .add_project_bytes(
                &file.filename,
                bytes,
                ProjectType::get_from_loaders(version.loaders.clone()),
            )
            .await?;

        Ok((path, version))
    }

    #[tracing::instrument(skip(self, bytes))]
    #[theseus_macros::debug_pin]
    pub async fn add_project_bytes(
        &self,
        file_name: &str,
        bytes: bytes::Bytes,
        project_type: Option<ProjectType>,
    ) -> crate::Result<ProjectPathId> {
        let project_type = if let Some(project_type) = project_type {
            project_type
        } else {
            let cursor = Cursor::new(&*bytes);

            let mut archive = zip::ZipArchive::new(cursor).map_err(|_| {
                crate::ErrorKind::InputError(
                    "Unable to infer project type for input file".to_string(),
                )
            })?;
            if archive.by_name("fabric.mod.json").is_ok()
                || archive.by_name("quilt.mod.json").is_ok()
                || archive.by_name("META-INF/mods.toml").is_ok()
                || archive.by_name("mcmod.info").is_ok()
            {
                ProjectType::Mod
            } else if archive.by_name("pack.mcmeta").is_ok() {
                if archive.file_names().any(|x| x.starts_with("data/")) {
                    ProjectType::DataPack
                } else {
                    ProjectType::ResourcePack
                }
            } else {
                return Err(crate::ErrorKind::InputError(
                    "Unable to infer project type for input file".to_string(),
                )
                .into());
            }
        };

        let state = State::get().await?;
        let relative_name = PathBuf::new()
            .join(project_type.get_folder())
            .join(file_name);
        let file_path = self
            .get_profile_full_path()
            .await?
            .join(relative_name.clone());
        let project_path_id = ProjectPathId::new(&relative_name);
        write(&file_path, &bytes, &state.io_semaphore).await?;

        let hash = get_hash(bytes).await?;
        {
            let mut profiles = state.profiles.write().await;

            if let Some(profile) = profiles.0.get_mut(&self.profile_id()) {
                profile.projects.insert(
                    project_path_id.clone(),
                    Project {
                        sha512: hash,
                        disabled: false,
                        metadata: ProjectMetadata::Unknown,
                        file_name: file_name.to_string(),
                    },
                );
                profile.metadata.date_modified = Utc::now();
            }
        }

        Ok(project_path_id)
    }

    /// Toggle a project's disabled state.
    /// 'path' should be relative to the profile's path.
    #[tracing::instrument(skip(self))]
    #[theseus_macros::debug_pin]
    pub async fn toggle_disable_project(
        &self,
        relative_path: &ProjectPathId,
    ) -> crate::Result<ProjectPathId> {
        let state = State::get().await?;
        if let Some(mut project) = {
            let mut profiles: tokio::sync::RwLockWriteGuard<'_, Profiles> =
                state.profiles.write().await;

            if let Some(profile) = profiles.0.get_mut(&self.profile_id()) {
                profile.projects.remove(relative_path)
            } else {
                None
            }
        } {
            // Get relative path from former ProjectPathId
            let relative_path = relative_path.0.to_path_buf();
            let mut new_path = relative_path.clone();

            if relative_path
                .extension()
                .map_or(false, |ext| ext == "disabled")
            {
                project.disabled = false;
                new_path.set_file_name(
                    relative_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .replace(".disabled", ""),
                );
            } else {
                new_path.set_file_name(format!(
                    "{}.disabled",
                    relative_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                ));
                project.disabled = true;
            }

            let true_path =
                self.get_profile_full_path().await?.join(&relative_path);
            let true_new_path =
                self.get_profile_full_path().await?.join(&new_path);
            io::rename(&true_path, &true_new_path).await?;

            let new_project_path_id = ProjectPathId::new(&new_path);

            let mut profiles = state.profiles.write().await;
            if let Some(profile) = profiles.0.get_mut(&self.profile_id()) {
                profile
                    .projects
                    .insert(new_project_path_id.clone(), project);
                profile.metadata.date_modified = Utc::now();
            }

            Ok(new_project_path_id)
        } else {
            Err(crate::ErrorKind::InputError(format!(
                "Project path does not exist: {:?}",
                relative_path
            ))
            .into())
        }
    }

    pub async fn remove_project(
        &self,
        relative_path: &ProjectPathId,
        dont_remove_arr: Option<bool>,
    ) -> crate::Result<()> {
        let state = State::get().await?;
        if self.projects.contains_key(relative_path) {
            io::remove_file(
                self.get_profile_full_path()
                    .await?
                    .join(relative_path.0.clone()),
            )
            .await?;
            if !dont_remove_arr.unwrap_or(false) {
                let mut profiles = state.profiles.write().await;

                if let Some(profile) = profiles.0.get_mut(&self.profile_id()) {
                    profile.projects.remove(relative_path);
                    profile.metadata.date_modified = Utc::now();
                }
            }
        } else {
            return Err(crate::ErrorKind::InputError(format!(
                "Project path does not exist: {:?}",
                relative_path
            ))
            .into());
        }

        Ok(())
    }
}

impl Profiles {
    #[tracing::instrument(skip(file_watcher))]
    #[theseus_macros::debug_pin]
    pub async fn init(
        dirs: &DirectoryInfo,
        file_watcher: &mut Debouncer<RecommendedWatcher>,
    ) -> crate::Result<Self> {
        let mut profiles = HashMap::new();
        let profiles_dir = dirs.profiles_dir().await;
        io::create_dir_all(&&profiles_dir).await?;

        file_watcher
            .watcher()
            .watch(&profiles_dir, RecursiveMode::NonRecursive)?;

        let mut entries = io::read_dir(&dirs.profiles_dir().await).await?;
        while let Some(entry) =
            entries.next_entry().await.map_err(IOError::from)?
        {
            let path = entry.path();
            if path.is_dir() {
                let prof = match Self::read_profile_from_dir(&path, dirs).await
                {
                    Ok(prof) => Some(prof),
                    Err(err) => {
                        tracing::warn!(
                            "Error loading profile: {err}. Skipping..."
                        );
                        None
                    }
                };
                if let Some(profile) = prof {
                    let path = io::canonicalize(path)?;
                    Profile::watch_fs(&path, file_watcher).await?;
                    profiles.insert(profile.profile_id(), profile);
                }
            }
        }

        Ok(Self(profiles))
    }

    #[tracing::instrument]
    #[theseus_macros::debug_pin]
    pub async fn update_projects() {
        let res = async {
            let state = State::get().await?;

            // profile, child paths
            let mut files: Vec<(Profile, Vec<PathBuf>)> = Vec::new();
            {
                let profiles = state.profiles.read().await;
                for (_profile_path, profile) in profiles.0.iter() {
                    let paths =
                        profile.get_profile_full_project_paths().await?;

                    files.push((profile.clone(), paths));
                }
            }

            let caches_dir = state.directories.caches_dir();
            future::try_join_all(files.into_iter().map(
                |(profile, files)| async {
                    let profile_name = profile.profile_id();
                    let inferred = super::projects::infer_data_from_files(
                        profile,
                        files,
                        caches_dir.clone(),
                        &state.io_semaphore,
                        &state.fetch_semaphore,
                    )
                    .await?;

                    let mut new_profiles = state.profiles.write().await;
                    if let Some(profile) = new_profiles.0.get_mut(&profile_name)
                    {
                        profile.projects = inferred;
                    }
                    drop(new_profiles);

                    Ok::<(), crate::Error>(())
                },
            ))
            .await?;

            {
                let profiles = state.profiles.read().await;
                profiles.sync().await?;
            }

            Ok::<(), crate::Error>(())
        }
        .await;

        match res {
            Ok(()) => {}
            Err(err) => {
                tracing::warn!("Unable to fetch profile projects: {err}")
            }
        };
    }

    #[tracing::instrument(skip(self, profile))]
    #[theseus_macros::debug_pin]
    pub async fn insert(&mut self, profile: Profile) -> crate::Result<&Self> {
        emit_profile(
            profile.uuid,
            profile.get_profile_full_path().await?,
            &profile.metadata.name,
            ProfilePayloadType::Added,
        )
        .await?;

        let state = State::get().await?;
        let mut file_watcher = state.file_watcher.write().await;
        Profile::watch_fs(
            &profile.get_profile_full_path().await?,
            &mut file_watcher,
        )
        .await?;

        let profile_name = profile.profile_id();
        profile_name.check_valid_utf()?;
        self.0.insert(profile_name, profile);
        Ok(self)
    }

    #[tracing::instrument(skip(self))]
    pub async fn remove(
        &mut self,
        profile_path: &ProfilePathId,
    ) -> crate::Result<Option<Profile>> {
        let profile = self.0.remove(profile_path);

        let path = profile_path.get_full_path().await?;
        if path.exists() {
            io::remove_dir_all(&path).await?;
        }

        Ok(profile)
    }

    #[tracing::instrument(skip_all)]
    pub async fn sync(&self) -> crate::Result<&Self> {
        let _state = State::get().await?;
        stream::iter(self.0.iter())
            .map(Ok::<_, crate::Error>)
            .try_for_each_concurrent(None, |(_, profile)| async move {
                let json = serde_json::to_vec(&profile)?;

                let json_path = profile
                    .get_profile_full_path()
                    .await?
                    .join(PROFILE_JSON_PATH);

                io::write(&json_path, &json).await?;
                Ok::<_, crate::Error>(())
            })
            .await?;

        Ok(self)
    }

    async fn read_profile_from_dir(
        path: &Path,
        dirs: &DirectoryInfo,
    ) -> crate::Result<Profile> {
        let json = io::read(&path.join(PROFILE_JSON_PATH)).await?;
        let mut profile = serde_json::from_slice::<Profile>(&json)?;

        // Get name from stripped path
        profile.path =
            PathBuf::from(path.strip_prefix(dirs.profiles_dir().await)?);

        Ok(profile)
    }

    pub fn sync_available_profiles_task(profile_path_id: ProfilePathId) {
        tokio::task::spawn(async move {
            let span = tracing::span!(
                tracing::Level::INFO,
                "sync_available_profiles_task"
            );
            let res = async {
                let _span = span.enter();
                let state = State::get().await?;
                let dirs = &state.directories;
                let mut profiles = state.profiles.write().await;

                if let Some(profile) = profiles.0.get_mut(&profile_path_id) {
                    if !profile.get_profile_full_path().await?.exists() {
                        // if path exists in the state but no longer in the filesystem, remove it from the state list
                        emit_profile(
                            profile.uuid,
                            profile.get_profile_full_path().await?,
                            &profile.metadata.name,
                            ProfilePayloadType::Removed,
                        )
                        .await?;
                        tracing::debug!("Removed!");
                        profiles.0.remove(&profile_path_id);
                    }
                } else if profile_path_id.get_full_path().await?.exists() {
                    // if it exists in the filesystem but no longer in the state, add it to the state list
                    profiles
                        .insert(
                            Self::read_profile_from_dir(
                                &profile_path_id.get_full_path().await?,
                                dirs,
                            )
                            .await?,
                        )
                        .await?;
                    Profile::sync_projects_task(profile_path_id);
                }
                Ok::<(), crate::Error>(())
            }
            .await;

            match res {
                Ok(()) => {}
                Err(err) => {
                    tracing::warn!("Unable to fetch all profiles: {err}")
                }
            };
        });
    }
}
