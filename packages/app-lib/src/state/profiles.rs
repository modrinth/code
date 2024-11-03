use super::settings::{Hooks, MemorySettings, WindowSize};
use crate::state::{cache_file_hash, CacheBehaviour, CachedEntry};
use crate::util;
use crate::util::fetch::{write_cached_icon, FetchSemaphore, IoSemaphore};
use crate::util::io::{self};
use chrono::{DateTime, TimeZone, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::path::{Path, PathBuf};

// Represent a Minecraft instance.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Profile {
    pub path: String,
    pub install_stage: ProfileInstallStage,

    pub name: String,
    pub icon_path: Option<String>,

    pub game_version: String,
    pub loader: ModLoader,
    pub loader_version: Option<String>,

    pub groups: Vec<String>,

    pub linked_data: Option<LinkedData>,

    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub last_played: Option<DateTime<Utc>>,

    pub submitted_time_played: u64,
    pub recent_time_played: u64,

    pub java_path: Option<String>,
    pub extra_launch_args: Option<Vec<String>>,
    pub custom_env_vars: Option<Vec<(String, String)>>,

    pub memory: Option<MemorySettings>,
    pub force_fullscreen: Option<bool>,
    pub game_resolution: Option<WindowSize>,
    pub hooks: Hooks,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProfileInstallStage {
    /// Profile is installed
    Installed,
    /// Profile's minecraft game is still installing
    Installing,
    /// Profile created for pack, but the pack hasn't been fully installed yet
    PackInstalling,
    /// Profile is not installed
    NotInstalled,
}

impl ProfileInstallStage {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Self::Installed => "installed",
            Self::Installing => "installing",
            Self::PackInstalling => "pack_installing",
            Self::NotInstalled => "not_installed",
        }
    }

    pub fn from_str(val: &str) -> Self {
        match val {
            "installed" => Self::Installed,
            "installing" => Self::Installing,
            "pack_installing" => Self::PackInstalling,
            "not_installed" => Self::NotInstalled,
            _ => Self::NotInstalled,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LinkedData {
    pub project_id: String,
    pub version_id: String,

    pub locked: bool,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ModLoader {
    Vanilla,
    Forge,
    Fabric,
    Quilt,
    NeoForge,
}

impl ModLoader {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Self::Vanilla => "vanilla",
            Self::Forge => "forge",
            Self::Fabric => "fabric",
            Self::Quilt => "quilt",
            Self::NeoForge => "neoforge",
        }
    }

    pub fn as_meta_str(&self) -> &'static str {
        match *self {
            Self::Vanilla => "vanilla",
            Self::Forge => "forge",
            Self::Fabric => "fabric",
            Self::Quilt => "quilt",
            Self::NeoForge => "neo",
        }
    }

    pub fn from_string(val: &str) -> Self {
        match val {
            "vanilla" => Self::Vanilla,
            "forge" => Self::Forge,
            "fabric" => Self::Fabric,
            "quilt" => Self::Quilt,
            "neoforge" => Self::NeoForge,
            _ => Self::Vanilla,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProfileFile {
    pub hash: String,
    pub file_name: String,
    pub size: u64,
    pub metadata: Option<FileMetadata>,
    pub update_version_id: Option<String>,
    pub project_type: ProjectType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileMetadata {
    pub project_id: String,
    pub version_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
    Mod,
    DataPack,
    ResourcePack,
    ShaderPack,
}

impl ProjectType {
    pub fn get_from_loaders(loaders: Vec<String>) -> Option<Self> {
        if loaders
            .iter()
            .any(|x| ["fabric", "forge", "quilt", "neoforge"].contains(&&**x))
        {
            Some(ProjectType::Mod)
        } else if loaders.iter().any(|x| x == "datapack") {
            Some(ProjectType::DataPack)
        } else if loaders.iter().any(|x| ["iris", "optifine"].contains(&&**x)) {
            Some(ProjectType::ShaderPack)
        } else if loaders
            .iter()
            .any(|x| ["vanilla", "canvas", "minecraft"].contains(&&**x))
        {
            Some(ProjectType::ResourcePack)
        } else {
            None
        }
    }

    pub fn get_from_parent_folder(path: PathBuf) -> Option<Self> {
        // Get parent folder
        let path = path.parent()?.file_name()?;
        match path.to_str()? {
            "mods" => Some(ProjectType::Mod),
            "datapacks" => Some(ProjectType::DataPack),
            "resourcepacks" => Some(ProjectType::ResourcePack),
            "shaderpacks" => Some(ProjectType::ShaderPack),
            _ => None,
        }
    }

    pub fn get_name(&self) -> &'static str {
        match self {
            ProjectType::Mod => "mod",
            ProjectType::DataPack => "datapack",
            ProjectType::ResourcePack => "resourcepack",
            ProjectType::ShaderPack => "shaderpack",
        }
    }

    pub fn get_folder(&self) -> &'static str {
        match self {
            ProjectType::Mod => "mods",
            ProjectType::DataPack => "datapacks",
            ProjectType::ResourcePack => "resourcepacks",
            ProjectType::ShaderPack => "shaderpacks",
        }
    }

    pub fn iterator() -> impl Iterator<Item = ProjectType> {
        [
            ProjectType::Mod,
            ProjectType::DataPack,
            ProjectType::ResourcePack,
            ProjectType::ShaderPack,
        ]
        .iter()
        .copied()
    }
}

struct ProfileQueryResult {
    path: String,
    install_stage: String,
    name: String,
    icon_path: Option<String>,
    game_version: String,
    mod_loader: String,
    mod_loader_version: Option<String>,
    groups: serde_json::Value,
    linked_project_id: Option<String>,
    linked_version_id: Option<String>,
    locked: Option<i64>,
    created: i64,
    modified: i64,
    last_played: Option<i64>,
    submitted_time_played: i64,
    recent_time_played: i64,
    override_java_path: Option<String>,
    override_extra_launch_args: serde_json::Value,
    override_custom_env_vars: serde_json::Value,
    override_mc_memory_max: Option<i64>,
    override_mc_force_fullscreen: Option<i64>,
    override_mc_game_resolution_x: Option<i64>,
    override_mc_game_resolution_y: Option<i64>,
    override_hook_pre_launch: Option<String>,
    override_hook_wrapper: Option<String>,
    override_hook_post_exit: Option<String>,
}

impl TryFrom<ProfileQueryResult> for Profile {
    type Error = crate::Error;

    fn try_from(x: ProfileQueryResult) -> Result<Self, Self::Error> {
        Ok(Profile {
            path: x.path,
            install_stage: ProfileInstallStage::from_str(&x.install_stage),
            name: x.name,
            icon_path: x.icon_path,
            game_version: x.game_version,
            loader: ModLoader::from_string(&x.mod_loader),
            loader_version: x.mod_loader_version,
            groups: serde_json::from_value(x.groups).unwrap_or_default(),
            linked_data: if let Some(project_id) = x.linked_project_id {
                if let Some(version_id) = x.linked_version_id {
                    x.locked.map(|locked| LinkedData {
                        project_id,
                        version_id,
                        locked: locked == 1,
                    })
                } else {
                    None
                }
            } else {
                None
            },
            created: Utc
                .timestamp_opt(x.created, 0)
                .single()
                .unwrap_or_else(Utc::now),
            modified: Utc
                .timestamp_opt(x.modified, 0)
                .single()
                .unwrap_or_else(Utc::now),
            last_played: x
                .last_played
                .and_then(|x| Utc.timestamp_opt(x, 0).single()),
            submitted_time_played: x.submitted_time_played as u64,
            recent_time_played: x.recent_time_played as u64,
            java_path: x.override_java_path,
            extra_launch_args: serde_json::from_value(
                x.override_extra_launch_args,
            )
            .ok(),
            custom_env_vars: serde_json::from_value(x.override_custom_env_vars)
                .ok(),
            memory: x
                .override_mc_memory_max
                .map(|x| MemorySettings { maximum: x as u32 }),
            force_fullscreen: x.override_mc_force_fullscreen.map(|x| x == 1),
            game_resolution: if let Some(x_res) =
                x.override_mc_game_resolution_x
            {
                x.override_mc_game_resolution_y
                    .map(|y_res| WindowSize(x_res as u16, y_res as u16))
            } else {
                None
            },
            hooks: Hooks {
                pre_launch: x.override_hook_pre_launch,
                wrapper: x.override_hook_wrapper,
                post_exit: x.override_hook_post_exit,
            },
        })
    }
}

macro_rules! select_profiles_with_predicate {
    ($predicate:tt, $param:ident) => {
        sqlx::query_as!(
            ProfileQueryResult,
            r#"
            SELECT
                path, install_stage, name, icon_path,
                game_version, mod_loader, mod_loader_version,
                json(groups) as "groups!: serde_json::Value",
                linked_project_id, linked_version_id, locked,
                created, modified, last_played,
                submitted_time_played, recent_time_played,
                override_java_path,
                json(override_extra_launch_args) as "override_extra_launch_args!: serde_json::Value", json(override_custom_env_vars) as "override_custom_env_vars!: serde_json::Value",
                override_mc_memory_max, override_mc_force_fullscreen, override_mc_game_resolution_x, override_mc_game_resolution_y,
                override_hook_pre_launch, override_hook_wrapper, override_hook_post_exit
            FROM profiles
            "#
                + $predicate,
            $param
        )
    };
}

impl Profile {
    pub async fn get(
        path: &str,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<Option<Self>> {
        Ok(Self::get_many(&[path], exec).await?.into_iter().next())
    }

    pub async fn get_many(
        paths: &[&str],
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<Vec<Self>> {
        let ids = serde_json::to_string(&paths)?;
        let results = select_profiles_with_predicate!(
            "WHERE path IN (SELECT value FROM json_each($1))",
            ids
        )
        .fetch_all(exec)
        .await?;

        results
            .into_iter()
            .map(|r| r.try_into())
            .collect::<crate::Result<Vec<_>>>()
    }

    pub async fn get_all(
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<Vec<Self>> {
        let true_val = 1;
        let results = select_profiles_with_predicate!("WHERE 1=$1", true_val)
            .fetch_all(exec)
            .await?;

        results
            .into_iter()
            .map(|r| r.try_into())
            .collect::<crate::Result<Vec<_>>>()
    }

    pub async fn upsert(
        &self,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<()> {
        let install_stage = self.install_stage.as_str();
        let mod_loader = self.loader.as_str();

        let groups = serde_json::to_string(&self.groups)?;

        let linked_data_project_id =
            self.linked_data.as_ref().map(|x| x.project_id.clone());
        let linked_data_version_id =
            self.linked_data.as_ref().map(|x| x.version_id.clone());
        let linked_data_locked = self.linked_data.as_ref().map(|x| x.locked);

        let created = self.created.timestamp();
        let modified = self.modified.timestamp();
        let last_played = self.last_played.map(|x| x.timestamp());

        let submitted_time_played = self.submitted_time_played as i64;
        let recent_time_played = self.recent_time_played as i64;

        let memory_max = self.memory.map(|x| x.maximum);

        let game_resolution_x = self.game_resolution.map(|x| x.0);
        let game_resolution_y = self.game_resolution.map(|x| x.1);

        let extra_launch_args = serde_json::to_string(&self.extra_launch_args)?;
        let custom_env_vars = serde_json::to_string(&self.custom_env_vars)?;

        sqlx::query!(
            "
            INSERT INTO profiles (
                path, install_stage, name, icon_path,
                game_version, mod_loader, mod_loader_version,
                groups,
                linked_project_id, linked_version_id, locked,
                created, modified, last_played,
                submitted_time_played, recent_time_played,
                override_java_path, override_extra_launch_args, override_custom_env_vars,
                override_mc_memory_max, override_mc_force_fullscreen, override_mc_game_resolution_x, override_mc_game_resolution_y,
                override_hook_pre_launch, override_hook_wrapper, override_hook_post_exit
            )
            VALUES (
                $1, $2, $3, $4,
                $5, $6, $7,
                jsonb($8),
                $9, $10, $11,
                $12, $13, $14,
                $15, $16,
                $17, jsonb($18), jsonb($19),
                $20, $21, $22, $23,
                $24, $25, $26
            )
            ON CONFLICT (path) DO UPDATE SET
                install_stage = $2,
                name = $3,
                icon_path = $4,

                game_version = $5,
                mod_loader = $6,
                mod_loader_version = $7,

                groups = jsonb($8),

                linked_project_id = $9,
                linked_version_id = $10,
                locked = $11,

                created = $12,
                modified = $13,
                last_played = $14,

                submitted_time_played = $15,
                recent_time_played = $16,

                override_java_path = $17,
                override_extra_launch_args = jsonb($18),
                override_custom_env_vars = jsonb($19),
                override_mc_memory_max = $20,
                override_mc_force_fullscreen = $21,
                override_mc_game_resolution_x = $22,
                override_mc_game_resolution_y = $23,

                override_hook_pre_launch = $24,
                override_hook_wrapper = $25,
                override_hook_post_exit = $26
            ",
            self.path,
            install_stage,
            self.name,
            self.icon_path,
            self.game_version,
            mod_loader,
            self.loader_version,
            groups,
            linked_data_project_id,
            linked_data_version_id,
            linked_data_locked,
            created,
            modified,
            last_played,
            submitted_time_played,
            recent_time_played,
            self.java_path,
            extra_launch_args,
            custom_env_vars,
            memory_max,
            self.force_fullscreen,
            game_resolution_x,
            game_resolution_y,
            self.hooks.pre_launch,
            self.hooks.wrapper,
            self.hooks.post_exit,
        )
            .execute(exec)
            .await?;

        Ok(())
    }

    pub async fn remove(
        profile_path: &str,
        pool: &SqlitePool,
    ) -> crate::Result<()> {
        sqlx::query!(
            "
            DELETE FROM profiles
            WHERE path = $1
            ",
            profile_path
        )
        .execute(pool)
        .await?;

        if let Ok(path) = crate::api::profile::get_full_path(profile_path).await
        {
            io::remove_dir_all(&path).await?;
        }

        Ok(())
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
        self.icon_path = Some(file.to_string_lossy().to_string());
        self.modified = Utc::now();
        Ok(())
    }

    pub(crate) async fn refresh_all() -> crate::Result<()> {
        let state = crate::State::get().await?;
        let all = Self::get_all(&state.pool).await?;

        let mut keys = vec![];

        for profile in &all {
            let path =
                crate::api::profile::get_full_path(&profile.path).await?;

            for project_type in ProjectType::iterator() {
                let folder = project_type.get_folder();
                let path = path.join(folder);

                if path.exists() {
                    for subdirectory in std::fs::read_dir(&path)
                        .map_err(|e| io::IOError::with_path(e, &path))?
                    {
                        let subdirectory =
                            subdirectory.map_err(io::IOError::from)?.path();
                        if subdirectory.is_file() {
                            if let Some(file_name) = subdirectory
                                .file_name()
                                .and_then(|x| x.to_str())
                            {
                                let file_size = subdirectory
                                    .metadata()
                                    .map_err(io::IOError::from)?
                                    .len();

                                keys.push(format!(
                                    "{file_size}-{}/{folder}/{file_name}",
                                    profile.path
                                ));
                            }
                        }
                    }
                }
            }
        }

        let file_hashes = CachedEntry::get_file_hash_many(
            &keys.iter().map(|s| &**s).collect::<Vec<_>>(),
            None,
            &state.pool,
            &state.fetch_semaphore,
        )
        .await?;

        let file_updates = file_hashes
            .iter()
            .filter_map(|x| {
                all.iter().find(|prof| x.path.contains(&prof.path)).map(
                    |profile| {
                        format!(
                            "{}-{}-{}",
                            x.hash,
                            profile.loader.as_str(),
                            profile.game_version
                        )
                    },
                )
            })
            .collect::<Vec<_>>();

        let file_hashes_ref =
            file_hashes.iter().map(|x| &*x.hash).collect::<Vec<_>>();
        let file_updates_ref =
            file_updates.iter().map(|x| &**x).collect::<Vec<_>>();
        tokio::try_join!(
            CachedEntry::get_file_many(
                &file_hashes_ref,
                Some(CacheBehaviour::MustRevalidate),
                &state.pool,
                &state.fetch_semaphore,
            ),
            CachedEntry::get_file_update_many(
                &file_updates_ref,
                Some(CacheBehaviour::MustRevalidate),
                &state.pool,
                &state.fetch_semaphore,
            )
        )?;

        Ok(())
    }

    pub async fn get_projects(
        &self,
        cache_behaviour: Option<CacheBehaviour>,
        pool: &SqlitePool,
        fetch_semaphore: &FetchSemaphore,
    ) -> crate::Result<DashMap<String, ProfileFile>> {
        let path = crate::api::profile::get_full_path(&self.path).await?;

        struct InitialScanFile {
            path: String,
            file_name: String,
            project_type: ProjectType,
            size: u64,
            cache_key: String,
        }

        let mut keys = vec![];

        for project_type in ProjectType::iterator() {
            let folder = project_type.get_folder();
            let path = path.join(folder);

            if path.exists() {
                for subdirectory in std::fs::read_dir(&path)
                    .map_err(|e| io::IOError::with_path(e, &path))?
                {
                    let subdirectory =
                        subdirectory.map_err(io::IOError::from)?.path();
                    if subdirectory.is_file() {
                        if let Some(file_name) =
                            subdirectory.file_name().and_then(|x| x.to_str())
                        {
                            let file_size = subdirectory
                                .metadata()
                                .map_err(io::IOError::from)?
                                .len();

                            keys.push(InitialScanFile {
                                path: format!(
                                    "{}/{folder}/{}",
                                    self.path,
                                    file_name.replace(".disabled", "")
                                ),
                                file_name: file_name.to_string(),
                                project_type,
                                size: file_size,
                                cache_key: format!(
                                    "{file_size}-{}/{folder}/{file_name}",
                                    self.path
                                ),
                            });
                        }
                    }
                }
            }
        }

        let file_hashes = CachedEntry::get_file_hash_many(
            &keys.iter().map(|s| &*s.cache_key).collect::<Vec<_>>(),
            None,
            pool,
            fetch_semaphore,
        )
        .await?;

        let file_updates = file_hashes
            .iter()
            .map(|x| {
                format!(
                    "{}-{}-{}",
                    x.hash,
                    self.loader.as_str(),
                    self.game_version
                )
            })
            .collect::<Vec<_>>();

        let file_hashes_ref =
            file_hashes.iter().map(|x| &*x.hash).collect::<Vec<_>>();
        let file_updates_ref =
            file_updates.iter().map(|x| &**x).collect::<Vec<_>>();
        let (mut file_info, file_updates) = tokio::try_join!(
            CachedEntry::get_file_many(
                &file_hashes_ref,
                cache_behaviour,
                pool,
                fetch_semaphore,
            ),
            CachedEntry::get_file_update_many(
                &file_updates_ref,
                cache_behaviour,
                pool,
                fetch_semaphore,
            )
        )?;

        let files = DashMap::new();

        for hash in file_hashes {
            let info_index = file_info.iter().position(|x| x.hash == hash.hash);
            let file = info_index.map(|x| file_info.remove(x));

            if let Some(initial_file_index) =
                keys.iter().position(|x| x.path == hash.path)
            {
                let initial_file = keys.remove(initial_file_index);

                let path = format!(
                    "{}/{}",
                    initial_file.project_type.get_folder(),
                    initial_file.file_name
                );

                let update_version_id = if let Some(update) = file_updates
                    .iter()
                    .find(|x| x.hash == hash.hash)
                    .map(|x| x.update_version_id.clone())
                {
                    if let Some(metadata) = &file {
                        if metadata.version_id != update {
                            Some(update)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                let file = ProfileFile {
                    update_version_id,
                    hash: hash.hash,
                    file_name: initial_file.file_name,
                    size: initial_file.size,
                    metadata: file.map(|x| FileMetadata {
                        project_id: x.project_id,
                        version_id: x.version_id,
                    }),
                    project_type: initial_file.project_type,
                };
                files.insert(path, file);
            }
        }

        Ok(files)
    }

    #[tracing::instrument(skip(pool))]
    pub async fn add_project_version(
        profile_path: &str,
        version_id: &str,
        pool: &SqlitePool,
        fetch_semaphore: &FetchSemaphore,
        io_semaphore: &IoSemaphore,
    ) -> crate::Result<String> {
        let version =
            CachedEntry::get_version(version_id, None, pool, fetch_semaphore)
                .await?
                .ok_or_else(|| {
                    crate::ErrorKind::InputError(format!(
                        "Unable to install version id {version_id}. Not found."
                    ))
                    .as_error()
                })?;

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

        let bytes = util::fetch::fetch(
            &file.url,
            file.hashes.get("sha1").map(|x| &**x),
            fetch_semaphore,
            pool,
        )
        .await?;

        let path = Self::add_project_bytes(
            profile_path,
            &file.filename,
            bytes,
            file.hashes.get("sha1").map(|x| &**x),
            ProjectType::get_from_loaders(version.loaders.clone()),
            io_semaphore,
            pool,
        )
        .await?;
        Ok(path)
    }

    #[tracing::instrument(skip(bytes))]

    pub async fn add_project_bytes(
        profile_path: &str,
        file_name: &str,
        bytes: bytes::Bytes,
        hash: Option<&str>,
        project_type: Option<ProjectType>,
        io_semaphore: &IoSemaphore,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<String> {
        let project_type = if let Some(project_type) = project_type {
            project_type
        } else {
            let cursor = std::io::Cursor::new(&*bytes);

            let mut archive = zip::ZipArchive::new(cursor).map_err(|_| {
                crate::ErrorKind::InputError(
                    "Unable to infer project type for input file".to_string(),
                )
            })?;

            if archive.by_name("fabric.mod.json").is_ok()
                || archive.by_name("quilt.mod.json").is_ok()
                || archive.by_name("META-INF/neoforge.mods.toml").is_ok()
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
            } else if archive.file_names().any(|x| x.starts_with("shaders/")) {
                ProjectType::ShaderPack
            } else {
                return Err(crate::ErrorKind::InputError(
                    "Unable to infer project type for input file".to_string(),
                )
                .into());
            }
        };

        let path = crate::api::profile::get_full_path(profile_path).await?;
        let project_path =
            format!("{}/{}", project_type.get_folder(), file_name);

        cache_file_hash(bytes.clone(), profile_path, &project_path, hash, exec)
            .await?;

        util::fetch::write(&path.join(&project_path), &bytes, io_semaphore)
            .await?;

        Ok(project_path)
    }

    /// Toggle a project's disabled state.
    #[tracing::instrument]
    pub async fn toggle_disable_project(
        profile_path: &str,
        project_path: &str,
    ) -> crate::Result<String> {
        let path = crate::api::profile::get_full_path(profile_path).await?;

        let new_path = if project_path.ends_with(".disabled") {
            project_path.replace(".disabled", "")
        } else {
            format!("{project_path}.disabled")
        };

        io::rename(&path.join(project_path), &path.join(&new_path)).await?;

        Ok(new_path)
    }

    #[tracing::instrument]
    pub async fn remove_project(
        profile_path: &str,
        project_path: &str,
    ) -> crate::Result<()> {
        if let Ok(path) = crate::api::profile::get_full_path(profile_path).await
        {
            io::remove_file(path.join(project_path)).await?;
        }

        Ok(())
    }
}
