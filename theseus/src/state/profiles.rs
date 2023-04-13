use super::settings::{Hooks, MemorySettings, WindowSize};
use crate::config::MODRINTH_API_URL;
use crate::data::DirectoryInfo;
use crate::state::projects::Project;
use crate::state::{ModrinthVersion, ProjectType};
use crate::util::fetch::{fetch, fetch_json, write, write_cached_icon};
use crate::State;
use daedalus::modded::LoaderVersion;
use dunce::canonicalize;
use futures::prelude::*;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tokio::sync::Semaphore;
use tokio::{fs, sync::RwLock};

const PROFILE_JSON_PATH: &str = "profile.json";

pub(crate) struct Profiles(pub HashMap<PathBuf, Profile>);

// TODO: possibly add defaults to some of these values
pub const CURRENT_FORMAT_VERSION: u32 = 1;

// Represent a Minecraft instance.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Profile {
    pub path: PathBuf,
    pub metadata: ProfileMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub java: Option<JavaSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<MemorySettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<WindowSize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks: Option<Hooks>,
    pub projects: HashMap<PathBuf, Project>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProfileMetadata {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<PathBuf>,
    pub game_version: String,
    #[serde(default)]
    pub loader: ModLoader,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loader_version: Option<LoaderVersion>,
    pub format_version: u32,
    pub linked_project_id: Option<String>,
}

// TODO: Quilt?
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JavaSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jre_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_arguments: Option<Vec<String>>,
}

impl Profile {
    #[tracing::instrument]
    pub async fn new(
        name: String,
        version: String,
        path: PathBuf,
    ) -> crate::Result<Self> {
        if name.trim().is_empty() {
            return Err(crate::ErrorKind::InputError(String::from(
                "Empty name for instance!",
            ))
            .into());
        }

        Ok(Self {
            path: canonicalize(path)?,
            metadata: ProfileMetadata {
                name,
                icon: None,
                game_version: version,
                loader: ModLoader::Vanilla,
                loader_version: None,
                format_version: CURRENT_FORMAT_VERSION,
                linked_project_id: None,
            },
            projects: HashMap::new(),
            java: None,
            memory: None,
            resolution: None,
            hooks: None,
        })
    }

    #[tracing::instrument]
    pub async fn set_icon<'a>(
        &'a mut self,
        cache_dir: &Path,
        semaphore: &RwLock<Semaphore>,
        icon: bytes::Bytes,
        file_name: &str,
    ) -> crate::Result<&'a mut Self> {
        let file =
            write_cached_icon(file_name, cache_dir, icon, semaphore).await?;
        self.metadata.icon = Some(file);
        Ok(self)
    }

    pub async fn sync(&mut self) -> crate::Result<()> {
        let state = State::get().await?;

        let paths = self.get_profile_project_paths()?;
        let projects = crate::state::infer_data_from_files(
            paths,
            state.directories.caches_dir(),
            &state.io_semaphore,
        )
        .await?;

        self.projects = projects;

        Ok(())
    }

    pub fn get_profile_project_paths(&self) -> crate::Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let mut read_paths = |path: &str| {
            let new_path = self.path.join(path);
            if new_path.exists() {
                for path in std::fs::read_dir(self.path.join(path))? {
                    let path = path?.path();
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

    pub async fn add_project_version(
        &mut self,
        version_id: String,
    ) -> crate::Result<PathBuf> {
        let state = State::get().await?;

        let version = fetch_json::<ModrinthVersion>(
            Method::GET,
            &format!("{MODRINTH_API_URL}version/{version_id}"),
            None,
            None,
            &state.io_semaphore,
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
            &state.io_semaphore,
        )
        .await?;

        let path = self
            .add_project_bytes(
                &file.filename,
                bytes,
                ProjectType::get_from_loaders(version.loaders),
            )
            .await?;

        Ok(path)
    }

    pub async fn add_project_bytes(
        &mut self,
        file_name: &str,
        bytes: bytes::Bytes,
        project_type: Option<ProjectType>,
    ) -> crate::Result<PathBuf> {
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
        let path = self.path.join(project_type.get_folder()).join(file_name);
        write(&path, &bytes, &state.io_semaphore).await?;

        self.sync().await?;

        Ok(path)
    }

    pub async fn toggle_disable_project(
        &mut self,
        path: &Path,
    ) -> crate::Result<()> {
        if let Some(mut project) = self.projects.remove(path) {
            let path = path.to_path_buf();
            let mut new_path = path.clone();

            if path.extension().map_or(false, |ext| ext == "disabled") {
                project.disabled = false;
            } else {
                new_path.set_file_name(format!(
                    "{}.disabled",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ));
                project.disabled = true;
            }

            println!("{:?} -> {:?}", path, new_path);
            fs::rename(path, &new_path).await?;

            self.projects.insert(new_path, project);
        } else {
            return Err(crate::ErrorKind::InputError(format!(
                "Project path does not exist: {:?}",
                path
            ))
            .into());
        }

        Ok(())
    }

    pub async fn remove_project(&mut self, path: &Path) -> crate::Result<()> {
        if self.projects.contains_key(path) {
            fs::remove_file(path).await?;
            self.projects.remove(path);
        } else {
            return Err(crate::ErrorKind::InputError(format!(
                "Project path does not exist: {:?}",
                path
            ))
            .into());
        }

        Ok(())
    }
}

impl Profiles {
    #[tracing::instrument]
    pub async fn init(
        dirs: &DirectoryInfo,
        io_sempahore: &RwLock<Semaphore>,
    ) -> crate::Result<Self> {
        let mut profiles = HashMap::new();
        fs::create_dir_all(dirs.profiles_dir()).await?;
        let mut entries = fs::read_dir(dirs.profiles_dir()).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                let prof = match Self::read_profile_from_dir(&path).await {
                    Ok(prof) => Some(prof),
                    Err(err) => {
                        log::warn!("Error loading profile: {err}. Skipping...");
                        None
                    }
                };
                if let Some(profile) = prof {
                    let path = canonicalize(path)?;
                    profiles.insert(path, profile);
                }
            }
        }

        // project path, parent profile path
        let mut files: HashMap<PathBuf, PathBuf> = HashMap::new();
        {
            for (profile_path, profile) in profiles.iter() {
                let paths = profile.get_profile_project_paths()?;

                for path in paths {
                    files.insert(path, profile_path.clone());
                }
            }
        }

        let inferred = super::projects::infer_data_from_files(
            files.keys().cloned().collect(),
            dirs.caches_dir(),
            io_sempahore,
        )
        .await?;

        for (key, value) in inferred {
            if let Some(profile_path) = files.get(&key) {
                if let Some(profile) = profiles.get_mut(profile_path) {
                    profile.projects.insert(key, value);
                }
            }
        }

        Ok(Self(profiles))
    }

    #[tracing::instrument(skip(self))]
    pub fn insert(&mut self, profile: Profile) -> crate::Result<&Self> {
        self.0.insert(
            canonicalize(&profile.path)?
                .to_str()
                .ok_or(
                    crate::ErrorKind::UTFError(profile.path.clone()).as_error(),
                )?
                .into(),
            profile,
        );
        Ok(self)
    }

    #[tracing::instrument(skip(self))]
    pub async fn insert_from<'a>(
        &'a mut self,
        path: &'a Path,
    ) -> crate::Result<&Self> {
        self.insert(Self::read_profile_from_dir(&canonicalize(path)?).await?)
    }

    #[tracing::instrument(skip(self))]
    pub async fn remove(&mut self, path: &Path) -> crate::Result<&Self> {
        let path =
            PathBuf::from(&canonicalize(path)?.to_string_lossy().to_string());
        self.0.remove(&path);

        if path.exists() {
            fs::remove_dir_all(path).await?;
        }

        Ok(self)
    }

    #[tracing::instrument(skip_all)]
    pub async fn sync(&self) -> crate::Result<&Self> {
        stream::iter(self.0.iter())
            .map(Ok::<_, crate::Error>)
            .try_for_each_concurrent(None, |(path, profile)| async move {
                let json = serde_json::to_vec(&profile)?;

                let json_path = Path::new(&path.to_string_lossy().to_string())
                    .join(PROFILE_JSON_PATH);

                fs::write(json_path, json).await?;
                Ok::<_, crate::Error>(())
            })
            .await?;

        Ok(self)
    }

    async fn read_profile_from_dir(path: &Path) -> crate::Result<Profile> {
        let json = fs::read(path.join(PROFILE_JSON_PATH)).await?;
        let mut profile = serde_json::from_slice::<Profile>(&json)?;
        profile.path = PathBuf::from(path);
        Ok(profile)
    }
}
