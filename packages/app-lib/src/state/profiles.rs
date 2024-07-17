use super::settings::{Hooks, MemorySettings, WindowSize};
use crate::util::fetch::{write_cached_icon, IoSemaphore};
use crate::util::io::{self};
use chrono::{DateTime, TimeZone, Utc};
use dashmap::DashMap;
use futures::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

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

    pub fn from_str(val: &str) -> Self {
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

impl Profile {
    pub async fn get(
        path: &str,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<Option<Self>> {
        let res = sqlx::query!(
            r#"
            SELECT
                path, install_stage, name, icon_path,
                game_version, mod_loader, mod_loader_version,
                linked_project_id, linked_version_id, locked,
                created, modified, last_played,
                submitted_time_played, recent_time_played,
                override_java_path,
                json(override_extra_launch_args) as "override_extra_launch_args!: serde_json::Value", json(override_custom_env_vars) as "override_custom_env_vars!: serde_json::Value",
                override_mc_memory_max, override_mc_force_fullscreen, override_mc_game_resolution_x, override_mc_game_resolution_y,
                override_hook_pre_launch, override_hook_wrapper, override_hook_post_exit
            FROM profiles
            WHERE path = $1
            "#,
            path
        )
            .fetch_optional(exec)
            .await?;

        Ok(res.map(|x| Profile {
            path: x.path,
            install_stage: ProfileInstallStage::from_str(&x.install_stage),
            name: x.name,
            icon_path: x.icon_path,
            game_version: x.game_version,
            loader: ModLoader::from_str(&x.mod_loader),
            loader_version: x.mod_loader_version,
            linked_data: None,
            created: Utc
                .timestamp_opt(x.created, 0)
                .single()
                .unwrap_or_else(|| Utc::now()),
            modified: Utc
                .timestamp_opt(x.modified, 0)
                .single()
                .unwrap_or_else(|| Utc::now()),
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
                if let Some(y_res) = x.override_mc_game_resolution_y {
                    Some(WindowSize(x_res as u16, y_res as u16))
                } else {
                    None
                }
            } else {
                None
            },
            hooks: Hooks {
                pre_launch: x.override_hook_pre_launch,
                wrapper: x.override_hook_wrapper,
                post_exit: x.override_hook_post_exit,
            },
        }))
    }

    pub async fn get_all(
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<DashMap<String, Self>> {
        // TODO: remove duplicated code
        let res = sqlx::query!(
            r#"
            SELECT
                path, install_stage, name, icon_path,
                game_version, mod_loader, mod_loader_version,
                linked_project_id, linked_version_id, locked,
                created, modified, last_played,
                submitted_time_played, recent_time_played,
                override_java_path,
                json(override_extra_launch_args) as "override_extra_launch_args!: serde_json::Value", json(override_custom_env_vars) as "override_custom_env_vars!: serde_json::Value",
                override_mc_memory_max, override_mc_force_fullscreen, override_mc_game_resolution_x, override_mc_game_resolution_y,
                override_hook_pre_launch, override_hook_wrapper, override_hook_post_exit
            FROM profiles
            "#
        )
            .fetch(exec)
            .try_fold(DashMap::new(), |acc, x| {
                acc.insert(
                    x.path.clone(),
                    Profile {
                        path: x.path,
                        install_stage: ProfileInstallStage::from_str(&x.install_stage),
                        name: x.name,
                        icon_path: x.icon_path,
                        game_version: x.game_version,
                        loader: ModLoader::from_str(&x.mod_loader),
                        loader_version: x.mod_loader_version,
                        linked_data: if let Some(project_id) = x.linked_project_id {
                            if let Some(version_id) = x.linked_version_id {
                                if let Some(locked) = x.locked {
                                    Some(LinkedData {
                                        project_id,
                                        version_id,
                                        locked: locked == 1,
                                    })
                                } else { None }
                            } else { None }
                        } else { None },
                        created: Utc.timestamp_opt(x.created, 0).single().unwrap_or_else(Utc::now),
                        modified: Utc.timestamp_opt(x.modified, 0).single().unwrap_or_else(Utc::now),
                        last_played: x.last_played.and_then(|x| Utc.timestamp_opt(x, 0).single()),
                        submitted_time_played: x.submitted_time_played as u64,
                        recent_time_played: x.recent_time_played as u64,
                        java_path: x.override_java_path,
                        extra_launch_args: serde_json::from_value(x
                            .override_extra_launch_args).ok(),
                        custom_env_vars: serde_json::from_value(x.override_custom_env_vars).ok(),
                        memory: x.override_mc_memory_max.map(|x| MemorySettings {
                            maximum: x as u32,
                        }),
                        force_fullscreen: x.override_mc_force_fullscreen.map(|x| x == 1),
                        game_resolution: if let Some(x_res) = x.override_mc_game_resolution_x {
                            if let Some(y_res) = x.override_mc_game_resolution_y {
                                Some(WindowSize(
                                    x_res as u16,
                                    y_res as u16,
                                ))
                            } else { None }

                        } else { None },
                        hooks: Hooks {
                            pre_launch: x.override_hook_pre_launch,
                            wrapper: x.override_hook_wrapper,
                            post_exit: x.override_hook_post_exit,
                        },
                    },
                );

                async move { Ok(acc) }
            })
            .await?;

        Ok(res)
    }

    pub async fn upsert(
        &self,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<()> {
        let install_stage = self.install_stage.as_str();
        let mod_loader = self.loader.as_str();

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
                $8, $9, $10,
                $11, $12, $13,
                $14, $15,
                $16, $17, $18,
                $19, $20, $21, $22,
                $23, $24, $25
            )
            ON CONFLICT (path) DO UPDATE SET
                install_stage = $2,
                name = $3,
                icon_path = $4,

                game_version = $5,
                mod_loader = $6,
                mod_loader_version = $7,

                linked_project_id = $8,
                linked_version_id = $9,
                locked = $10,

                created = $11,
                modified = $12,
                last_played = $13,

                submitted_time_played = $14,
                recent_time_played = $15,

                override_java_path = $16,
                override_extra_launch_args = jsonb($17),
                override_custom_env_vars = jsonb($18),
                override_mc_memory_max = $19,
                override_mc_force_fullscreen = $20,
                override_mc_game_resolution_x = $21,
                override_mc_game_resolution_y = $22,

                override_hook_pre_launch = $23,
                override_hook_wrapper = $24,
                override_hook_post_exit = $25
            ",
            self.path,
            install_stage,
            self.name,
            self.icon_path,
            self.game_version,
            mod_loader,
            self.loader_version,
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
        &self,
        transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> crate::Result<()> {
        let path = crate::api::profile::get_full_path(&self.path).await?;

        if path.exists() {
            io::remove_dir_all(&path).await?;
        }

        sqlx::query!(
            "
            DELETE FROM profiles
            WHERE path = $1
            ",
            self.path
        )
        .execute(&mut **transaction)
        .await?;

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

    // pub fn crash_task(path: ProfilePathId) {
    //     tokio::task::spawn(async move {
    //         let res = async {
    //             let profile = crate::api::profile::get(&path).await?;
    //
    //             if let Some(profile) = profile {
    //                 // Hide warning if profile is not yet installed
    //                 if profile.install_stage == ProfileInstallStage::Installed {
    //                     emit_warning(&format!("Profile {} has crashed! Visit the logs page to see a crash report.", profile.metadata.name)).await?;
    //                 }
    //             }
    //
    //             Ok::<(), crate::Error>(())
    //         }
    //             .await;
    //
    //         match res {
    //             Ok(()) => {}
    //             Err(err) => {
    //                 tracing::warn!(
    //                     "Unable to send crash report to frontend: {err}"
    //                 )
    //             }
    //         };
    //     });
    // }

    // #[tracing::instrument(skip(watcher))]
    // #[theseus_macros::debug_pin]
    // pub async fn watch_fs(
    //     profile_path: &Path,
    //     watcher: &mut Debouncer<RecommendedWatcher>,
    // ) -> crate::Result<()> {
    //     async fn watch_path(
    //         profile_path: &Path,
    //         watcher: &mut Debouncer<RecommendedWatcher>,
    //         path: &str,
    //     ) -> crate::Result<()> {
    //         let path = profile_path.join(path);
    //
    //         io::create_dir_all(&path).await?;
    //
    //         watcher
    //             .watcher()
    //             .watch(&profile_path.join(path), RecursiveMode::Recursive)?;
    //
    //         Ok(())
    //     }
    //
    //     watch_path(profile_path, watcher, ProjectType::Mod.get_folder())
    //         .await?;
    //     watch_path(profile_path, watcher, ProjectType::ShaderPack.get_folder())
    //         .await?;
    //     watch_path(
    //         profile_path,
    //         watcher,
    //         ProjectType::ResourcePack.get_folder(),
    //     )
    //     .await?;
    //     watch_path(profile_path, watcher, ProjectType::DataPack.get_folder())
    //         .await?;
    //     watch_path(profile_path, watcher, "crash-reports").await?;
    //
    //     Ok(())
    // }

    // #[tracing::instrument(skip(self))]
    // #[theseus_macros::debug_pin]
    // pub async fn add_project_version(
    //     &self,
    //     version_id: String,
    // ) -> crate::Result<(ProjectPathId, ModrinthVersion)> {
    //     let state = State::get().await?;
    //     let creds = state.credentials.read().await;
    //     let version = fetch_json::<ModrinthVersion>(
    //         Method::GET,
    //         &format!("{MODRINTH_API_URL}version/{version_id}"),
    //         None,
    //         None,
    //         &state.fetch_semaphore,
    //         &creds,
    //     )
    //     .await?;
    //     drop(creds);
    //     let file = if let Some(file) = version.files.iter().find(|x| x.primary)
    //     {
    //         file
    //     } else if let Some(file) = version.files.first() {
    //         file
    //     } else {
    //         return Err(crate::ErrorKind::InputError(
    //             "No files for input version present!".to_string(),
    //         )
    //         .into());
    //     };
    //
    //     let creds = state.credentials.read().await;
    //     let bytes = fetch(
    //         &file.url,
    //         file.hashes.get("sha1").map(|x| &**x),
    //         &state.fetch_semaphore,
    //         &creds,
    //     )
    //     .await?;
    //     drop(creds);
    //     let path = self
    //         .add_project_bytes(
    //             &file.filename,
    //             bytes,
    //             ProjectType::get_from_loaders(version.loaders.clone()),
    //         )
    //         .await?;
    //     Ok((path, version))
    // }

    // #[tracing::instrument(skip(self, bytes))]
    // #[theseus_macros::debug_pin]
    // pub async fn add_project_bytes(
    //     &self,
    //     file_name: &str,
    //     bytes: bytes::Bytes,
    //     project_type: Option<ProjectType>,
    // ) -> crate::Result<ProjectPathId> {
    //     let project_type = if let Some(project_type) = project_type {
    //         project_type
    //     } else {
    //         let cursor = Cursor::new(&*bytes);
    //
    //         let mut archive = zip::ZipArchive::new(cursor).map_err(|_| {
    //             crate::ErrorKind::InputError(
    //                 "Unable to infer project type for input file".to_string(),
    //             )
    //         })?;
    //         if archive.by_name("fabric.mod.json").is_ok()
    //             || archive.by_name("quilt.mod.json").is_ok()
    //             || archive.by_name("META-INF/mods.toml").is_ok()
    //             || archive.by_name("mcmod.info").is_ok()
    //         {
    //             ProjectType::Mod
    //         } else if archive.by_name("pack.mcmeta").is_ok() {
    //             if archive.file_names().any(|x| x.starts_with("data/")) {
    //                 ProjectType::DataPack
    //             } else {
    //                 ProjectType::ResourcePack
    //             }
    //         } else {
    //             return Err(crate::ErrorKind::InputError(
    //                 "Unable to infer project type for input file".to_string(),
    //             )
    //             .into());
    //         }
    //     };
    //
    //     let state = State::get().await?;
    //     let relative_name = PathBuf::new()
    //         .join(project_type.get_folder())
    //         .join(file_name);
    //     let file_path = self
    //         .get_profile_full_path()
    //         .await?
    //         .join(relative_name.clone());
    //     let project_path_id = ProjectPathId::new(&relative_name);
    //     write(&file_path, &bytes, &state.io_semaphore).await?;
    //
    //     let hash = get_hash(bytes).await?;
    //     {
    //         let mut profiles = state.profiles.write().await;
    //
    //         if let Some(profile) = profiles.0.get_mut(&self.profile_id()) {
    //             profile.projects.insert(
    //                 project_path_id.clone(),
    //                 Project {
    //                     sha512: hash,
    //                     disabled: false,
    //                     metadata: ProjectMetadata::Unknown,
    //                     file_name: file_name.to_string(),
    //                 },
    //             );
    //             profile.metadata.date_modified = Utc::now();
    //         }
    //     }
    //
    //     Ok(project_path_id)
    // }
    //
    // /// Toggle a project's disabled state.
    // #[tracing::instrument(skip(self))]
    // #[theseus_macros::debug_pin]
    // pub async fn toggle_disable_project(
    //     &self,
    //     relative_path: &ProjectPathId,
    // ) -> crate::Result<ProjectPathId> {
    //     let state = State::get().await?;
    //     if let Some(mut project) = {
    //         let mut profiles: tokio::sync::RwLockWriteGuard<'_, Profiles> =
    //             state.profiles.write().await;
    //
    //         if let Some(profile) = profiles.0.get_mut(&self.profile_id()) {
    //             profile.projects.remove(relative_path)
    //         } else {
    //             None
    //         }
    //     } {
    //         // Get relative path from former ProjectPathId
    //         let relative_path = relative_path.0.to_path_buf();
    //         let mut new_path = relative_path.clone();
    //
    //         if relative_path
    //             .extension()
    //             .map_or(false, |ext| ext == "disabled")
    //         {
    //             project.disabled = false;
    //             new_path.set_file_name(
    //                 relative_path
    //                     .file_name()
    //                     .unwrap_or_default()
    //                     .to_string_lossy()
    //                     .replace(".disabled", ""),
    //             );
    //         } else {
    //             new_path.set_file_name(format!(
    //                 "{}.disabled",
    //                 relative_path
    //                     .file_name()
    //                     .unwrap_or_default()
    //                     .to_string_lossy()
    //             ));
    //             project.disabled = true;
    //         }
    //
    //         let true_path =
    //             self.get_profile_full_path().await?.join(&relative_path);
    //         let true_new_path =
    //             self.get_profile_full_path().await?.join(&new_path);
    //         io::rename(&true_path, &true_new_path).await?;
    //
    //         let new_project_path_id = ProjectPathId::new(&new_path);
    //
    //         let mut profiles = state.profiles.write().await;
    //         if let Some(profile) = profiles.0.get_mut(&self.profile_id()) {
    //             profile
    //                 .projects
    //                 .insert(new_project_path_id.clone(), project);
    //             profile.metadata.date_modified = Utc::now();
    //         }
    //
    //         Ok(new_project_path_id)
    //     } else {
    //         Err(crate::ErrorKind::InputError(format!(
    //             "Project path does not exist: {:?}",
    //             relative_path
    //         ))
    //         .into())
    //     }
    // }
    //
    // pub async fn remove_project(
    //     &self,
    //     relative_path: &ProjectPathId,
    //     dont_remove_arr: Option<bool>,
    // ) -> crate::Result<()> {
    //     let state = State::get().await?;
    //     if self.projects.contains_key(relative_path) {
    //         io::remove_file(
    //             self.get_profile_full_path()
    //                 .await?
    //                 .join(relative_path.0.clone()),
    //         )
    //         .await?;
    //         if !dont_remove_arr.unwrap_or(false) {
    //             let mut profiles = state.profiles.write().await;
    //
    //             if let Some(profile) = profiles.0.get_mut(&self.profile_id()) {
    //                 profile.projects.remove(relative_path);
    //                 profile.metadata.date_modified = Utc::now();
    //             }
    //         }
    //     } else {
    //         // If we are removing a project that doesn't exist, allow it to pass through without error, but warn
    //         tracing::warn!(
    //             "Attempted to remove non-existent project: {:?}",
    //             relative_path
    //         );
    //     }
    //
    //     Ok(())
    // }
}
