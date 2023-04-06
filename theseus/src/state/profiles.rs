use super::settings::{Hooks, MemorySettings, WindowSize};
use crate::config::BINCODE_CONFIG;
use crate::data::DirectoryInfo;
use crate::state::projects::Project;
use daedalus::modded::LoaderVersion;
use dunce::canonicalize;
use futures::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tokio::fs;
use tokio::sync::Semaphore;

const PROFILE_JSON_PATH: &str = "profile.json";
const PROFILE_SUBTREE: &[u8] = b"profiles";

pub(crate) struct Profiles(pub HashMap<PathBuf, Profile>);

// TODO: possibly add defaults to some of these values
pub const CURRENT_FORMAT_VERSION: u32 = 1;
pub const SUPPORTED_ICON_FORMATS: &[&str] = &[
    "bmp", "gif", "jpeg", "jpg", "jpe", "png", "svg", "svgz", "webp", "rgb",
    "mp4",
];

// Represent a Minecraft instance.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Profile {
    #[serde(skip)]
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
        icon: &'a Path,
    ) -> crate::Result<&'a mut Self> {
        let ext = icon
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("");

        if SUPPORTED_ICON_FORMATS.contains(&ext) {
            let file_name = format!("icon.{ext}");
            fs::copy(icon, &self.path.join(&file_name)).await?;
            self.metadata.icon =
                Some(Path::new(&format!("./{file_name}")).to_owned());

            Ok(self)
        } else {
            Err(crate::ErrorKind::InputError(format!(
                "Unsupported image type: {ext}"
            ))
            .into())
        }
    }

    pub fn get_profile_project_paths(&self) -> crate::Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let mut read_paths = |path: &str| {
            let new_path = self.path.join(path);
            if new_path.exists() {
                for path in std::fs::read_dir(self.path.join(path))? {
                    files.push(path?.path());
                }
            }
            Ok::<(), crate::Error>(())
        };

        read_paths("mods")?;
        read_paths("shaders")?;
        read_paths("resourcepacks")?;
        read_paths("datapacks")?;

        Ok(files)
    }
}

impl Profiles {
    #[tracing::instrument(skip(db))]
    pub async fn init(
        db: &sled::Db,
        dirs: &DirectoryInfo,
        io_sempahore: &Semaphore,
    ) -> crate::Result<Self> {
        let profile_db = db.get(PROFILE_SUBTREE)?.map_or(
            Ok(Default::default()),
            |bytes| {
                bincode::decode_from_slice::<Box<[PathBuf]>, _>(
                    &bytes,
                    *BINCODE_CONFIG,
                )
                .map(|it| it.0)
            },
        )?;

        let mut profiles = stream::iter(profile_db.iter())
            .then(|it| async move {
                let path = PathBuf::from(it);
                let prof = match Self::read_profile_from_dir(&path).await {
                    Ok(prof) => Some(prof),
                    Err(err) => {
                        log::warn!("Error loading profile: {err}. Skipping...");
                        None
                    }
                };
                (path, prof)
            })
            .filter_map(|(key, opt_value)| async move {
                opt_value.map(|value| (key, value))
            })
            .collect::<HashMap<PathBuf, Profile>>()
            .await;

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
    pub async fn sync<'a>(
        &'a self,
        batch: &'a mut sled::Batch,
    ) -> crate::Result<&Self> {
        stream::iter(self.0.iter())
            .map(Ok::<_, crate::Error>)
            .try_for_each_concurrent(None, |(path, profile)| async move {
                let json = serde_json::to_vec_pretty(&profile)?;

                let json_path = Path::new(&path.to_string_lossy().to_string())
                    .join(PROFILE_JSON_PATH);

                fs::write(json_path, json).await?;
                Ok::<_, crate::Error>(())
            })
            .await?;

        batch.insert(
            PROFILE_SUBTREE,
            bincode::encode_to_vec(
                self.0.keys().collect::<Box<[_]>>(),
                *BINCODE_CONFIG,
            )?,
        );
        Ok(self)
    }

    async fn read_profile_from_dir(path: &Path) -> crate::Result<Profile> {
        let json = fs::read(path.join(PROFILE_JSON_PATH)).await?;
        let mut profile = serde_json::from_slice::<Profile>(&json)?;
        profile.path = PathBuf::from(path);
        Ok(profile)
    }
}
