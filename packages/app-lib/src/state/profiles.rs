use super::settings::{Hooks, MemorySettings, WindowSize};
use crate::profile::get_full_path;
use crate::state::server_join_log::JoinLogEntry;
use crate::state::{
    CacheBehaviour, CachedEntry, CachedFileHash, cache_file_hash,
};
use crate::util;
use crate::util::fetch::{FetchSemaphore, IoSemaphore, write_cached_icon};
use crate::util::io::{self};
use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use dashmap::DashMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::path::Path;
use std::sync::LazyLock;
use tokio::fs::DirEntry;
use tokio::io::{AsyncBufReadExt, AsyncRead};
use tokio::task::JoinSet;

// Represent a Minecraft instance.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Profile {
    pub path: String,
    pub install_stage: ProfileInstallStage,
    pub launcher_feature_version: LauncherFeatureVersion,

    pub name: String,
    pub icon_path: Option<String>,

    pub game_version: String,
    pub protocol_version: Option<i32>,
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
    MinecraftInstalling,
    /// Pack is installed, but Minecraft installation has not begun
    PackInstalled,
    /// Profile created for pack, but the pack hasn't been fully installed yet
    PackInstalling,
    /// Profile is not installed
    NotInstalled,
}

impl ProfileInstallStage {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Self::Installed => "installed",
            Self::MinecraftInstalling => "minecraft_installing",
            Self::PackInstalled => "pack_installed",
            Self::PackInstalling => "pack_installing",
            Self::NotInstalled => "not_installed",
        }
    }

    pub fn from_str(val: &str) -> Self {
        match val {
            "installed" => Self::Installed,
            "minecraft_installing" => Self::MinecraftInstalling,
            "installing" => Self::MinecraftInstalling, // Backwards compatibility
            "pack_installed" => Self::PackInstalled,
            "pack_installing" => Self::PackInstalling,
            "not_installed" => Self::NotInstalled,
            _ => Self::NotInstalled,
        }
    }
}

#[derive(
    Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd,
)]
#[serde(rename_all = "snake_case")]
pub enum LauncherFeatureVersion {
    None,
    MigratedServerLastPlayTime,
}

impl LauncherFeatureVersion {
    pub const MOST_RECENT: Self = Self::MigratedServerLastPlayTime;

    pub fn as_str(&self) -> &'static str {
        match *self {
            Self::None => "none",
            Self::MigratedServerLastPlayTime => {
                "migrated_server_last_play_time"
            }
        }
    }

    pub fn from_str(val: &str) -> Self {
        match val {
            "none" => Self::None,
            "migrated_server_last_play_time" => {
                Self::MigratedServerLastPlayTime
            }
            _ => Self::None,
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

#[derive(Serialize, Deserialize, Clone, Debug, Copy, PartialEq, Eq)]
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

    pub fn get_from_parent_folder(path: &Path) -> Option<Self> {
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
            ProjectType::ShaderPack => "shader",
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

    pub fn get_loaders(&self) -> &'static [&'static str] {
        match self {
            ProjectType::Mod => &["fabric", "forge", "quilt", "neoforge"],
            ProjectType::DataPack => &["datapack"],
            ProjectType::ResourcePack => &["vanilla", "canvas", "minecraft"],
            ProjectType::ShaderPack => &["iris", "optifine"],
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
    protocol_version: Option<i64>,
    launcher_feature_version: String,
}

impl TryFrom<ProfileQueryResult> for Profile {
    type Error = crate::Error;

    fn try_from(x: ProfileQueryResult) -> Result<Self, Self::Error> {
        Ok(Profile {
            path: x.path,
            install_stage: ProfileInstallStage::from_str(&x.install_stage),
            launcher_feature_version: LauncherFeatureVersion::from_str(
                &x.launcher_feature_version,
            ),
            name: x.name,
            icon_path: x.icon_path,
            game_version: x.game_version,
            protocol_version: x.protocol_version.map(|x| x as i32),
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
                path, install_stage, launcher_feature_version, name, icon_path,
                game_version, protocol_version, mod_loader, mod_loader_version,
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
        let launcher_feature_version = self.launcher_feature_version.as_str();

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
                override_hook_pre_launch, override_hook_wrapper, override_hook_post_exit,
                protocol_version, launcher_feature_version
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
                $24, $25, $26,
                $27, $28
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
                override_hook_post_exit = $26,

                protocol_version = $27,
                launcher_feature_version = $28
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
            self.protocol_version,
            launcher_feature_version
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
        let mut all = Self::get_all(&state.pool).await?;

        let mut keys = vec![];
        let mut migrations = JoinSet::new();

        for profile in &mut all {
            let path = get_full_path(&profile.path).await?;

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

            if profile.install_stage == ProfileInstallStage::MinecraftInstalling
            {
                profile.install_stage = ProfileInstallStage::PackInstalled;
                profile.upsert(&state.pool).await?;
            } else if profile.install_stage
                == ProfileInstallStage::PackInstalling
            {
                profile.install_stage = ProfileInstallStage::NotInstalled;
                profile.upsert(&state.pool).await?;
            }

            if profile.launcher_feature_version
                < LauncherFeatureVersion::MOST_RECENT
            {
                let state = state.clone();
                let profile_path = profile.path.clone();
                migrations.spawn(async move {
                    let Ok(Some(mut profile)) = Self::get(&profile_path, &state.pool).await else {
                        tracing::error!("Failed to find instance '{}' for migration", profile_path);
                        return;
                    };
                    drop(profile_path);

                    tracing::info!(
                        "Migrating profile '{}' from launcher feature version {:?} to {:?}",
                        profile.path, profile.launcher_feature_version, LauncherFeatureVersion::MOST_RECENT
                    );
                    loop {
                        let result = profile.perform_launcher_feature_migration(&state).await;
                        if result.is_err() || profile.launcher_feature_version == LauncherFeatureVersion::MOST_RECENT {
                            if let Err(err) = result {
                                tracing::error!("Failed to migrate instance '{}': {}", profile.path, err);
                                return;
                            }
                            if let Err(err) = profile.upsert(&state.pool).await {
                                tracing::error!("Failed to update instance '{}' migration state: {}", profile.path, err);
                                return;
                            }
                            break;
                        }
                    }
                    tracing::info!("Finished migration for profile '{}'", profile.path);
                });
            }
        }
        migrations.join_all().await;

        let file_hashes = CachedEntry::get_file_hash_many(
            &keys.iter().map(|s| &**s).collect::<Vec<_>>(),
            None,
            &state.pool,
            &state.fetch_semaphore,
        )
        .await?;

        let file_updates = file_hashes
            .iter()
            .filter_map(|file| {
                all.iter()
                    .find(|prof| file.path.contains(&prof.path))
                    .map(|profile| Self::get_cache_key(file, profile))
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

    async fn perform_launcher_feature_migration(
        &mut self,
        state: &crate::State,
    ) -> crate::Result<()> {
        match self.launcher_feature_version {
            LauncherFeatureVersion::None => {
                if self.last_played.is_none() {
                    self.launcher_feature_version =
                        LauncherFeatureVersion::MigratedServerLastPlayTime;
                    return Ok(());
                }
                let mut join_log_entry = JoinLogEntry {
                    profile_path: self.path.clone(),
                    ..Default::default()
                };
                let logs_path = state.directories.profile_logs_dir(&self.path);
                let Ok(mut directory) = io::read_dir(&logs_path).await else {
                    self.launcher_feature_version =
                        LauncherFeatureVersion::MigratedServerLastPlayTime;
                    return Ok(());
                };
                let existing_joins_map =
                    super::server_join_log::get_joins(&self.path, &state.pool)
                        .await?;
                let existing_joins = existing_joins_map
                    .keys()
                    .map(|x| (&x.0 as &str, x.1))
                    .collect::<HashSet<_>>();
                while let Some(log_file) = directory.next_entry().await? {
                    if let Err(err) = Self::parse_log_file(
                        &log_file,
                        |host, port| existing_joins.contains(&(host, port)),
                        state,
                        &mut join_log_entry,
                    )
                    .await
                    {
                        tracing::error!(
                            "Failed to parse log file '{}': {}",
                            log_file.path().display(),
                            err
                        );
                    }
                }
                self.launcher_feature_version =
                    LauncherFeatureVersion::MigratedServerLastPlayTime;
            }
            LauncherFeatureVersion::MOST_RECENT => unreachable!(
                "LauncherFeatureVersion::MOST_RECENT was not updated"
            ),
        }
        Ok(())
    }

    // Parses a log file on a best-effort basis, using the log's creation time, rather than the
    // actual times mentioned in the log file, which are missing date information.
    async fn parse_log_file(
        log_file: &DirEntry,
        should_skip: impl Fn(&str, u16) -> bool,
        state: &crate::State,
        join_entry: &mut JoinLogEntry,
    ) -> crate::Result<()> {
        let file_name = log_file.file_name();
        let Some(file_name) = file_name.to_str() else {
            return Ok(());
        };
        let log_time = io::metadata(&log_file.path()).await?.created()?.into();
        if file_name == "latest.log" {
            let file = io::open_file(&log_file.path()).await?;
            Self::parse_open_log_file(
                file,
                should_skip,
                log_time,
                state,
                join_entry,
            )
            .await
        } else if file_name.ends_with(".log.gz") {
            let file = io::open_file(&log_file.path()).await?;
            let file = tokio::io::BufReader::new(file);
            let file =
                async_compression::tokio::bufread::GzipDecoder::new(file);
            Self::parse_open_log_file(
                file,
                should_skip,
                log_time,
                state,
                join_entry,
            )
            .await
        } else {
            Ok(())
        }
    }

    async fn parse_open_log_file(
        reader: impl AsyncRead + Unpin,
        should_skip: impl Fn(&str, u16) -> bool,
        mut log_time: DateTime<Utc>,
        state: &crate::State,
        join_entry: &mut JoinLogEntry,
    ) -> crate::Result<()> {
        static LOG_LINE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"^\[[0-9]{2}(?::[0-9]{2}){2}] \[.+?/[A-Z]+?]: Connecting to (.+?), ([1-9][0-9]{0,4})$").unwrap()
        });
        let reader = tokio::io::BufReader::new(reader);
        let mut lines = reader.lines();
        while let Some(log_line) = lines.next_line().await? {
            let Some(log_line) = LOG_LINE_REGEX.captures(&log_line) else {
                continue;
            };

            let Some(host) = log_line.get(1) else {
                continue;
            };
            let host = host.as_str();

            let Some(port) = log_line.get(2) else {
                continue;
            };
            let Ok(port) = port.as_str().parse::<u16>() else {
                continue;
            };

            if should_skip(host, port) {
                continue;
            }

            join_entry.host = host.to_string();
            join_entry.port = port;
            join_entry.join_time = log_time;
            join_entry.upsert(&state.pool).await?;

            log_time += TimeDelta::seconds(1);
        }
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
                                    file_name.trim_end_matches(".disabled")
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
            .map(|x| Self::get_cache_key(x, self))
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

            if let Some(initial_file_index) = keys
                .iter()
                .position(|x| x.path == hash.path.trim_end_matches(".disabled"))
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

    fn get_cache_key(file: &CachedFileHash, profile: &Profile) -> String {
        format!(
            "{}-{}-{}",
            file.hash,
            file.project_type
                .filter(|x| *x != ProjectType::Mod)
                .map(|x| x.get_loaders().join("+"))
                .unwrap_or_else(|| profile.loader.as_str().to_string()),
            profile.game_version
        )
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

        cache_file_hash(
            bytes.clone(),
            profile_path,
            &project_path,
            hash,
            Some(project_type),
            exec,
        )
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
            project_path.trim_end_matches(".disabled").to_string()
        } else {
            format!("{project_path}.disabled")
        };

        io::rename_or_move(&path.join(project_path), &path.join(&new_path))
            .await?;

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
