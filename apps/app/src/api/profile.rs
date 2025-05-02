use crate::api::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use theseus::prelude::*;
use theseus::profile::QuickPlayType;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("profile")
        .invoke_handler(tauri::generate_handler![
            profile_remove,
            profile_get,
            profile_get_many,
            profile_get_projects,
            profile_get_optimal_jre_key,
            profile_get_full_path,
            profile_get_mod_full_path,
            profile_list,
            profile_check_installed,
            profile_install,
            profile_update_all,
            profile_update_project,
            profile_add_project_from_version,
            profile_add_project_from_path,
            profile_toggle_disable_project,
            profile_remove_project,
            profile_update_managed_modrinth_version,
            profile_repair_managed_modrinth,
            profile_run,
            profile_run_credentials,
            profile_kill,
            profile_edit,
            profile_edit_icon,
            profile_export_mrpack,
            profile_get_pack_export_candidates,
        ])
        .build()
}

// Remove a profile
// invoke('plugin:profile|profile_add_path',path)
#[tauri::command]
pub async fn profile_remove(path: &str) -> Result<()> {
    profile::remove(path).await?;
    Ok(())
}

// Get a profile by path
// invoke('plugin:profile|profile_add_path',path)
#[tauri::command]
pub async fn profile_get(path: &str) -> Result<Option<Profile>> {
    let res = profile::get(path).await?;
    Ok(res)
}

#[tauri::command]
pub async fn profile_get_many(paths: Vec<String>) -> Result<Vec<Profile>> {
    let ids = paths.iter().map(|x| &**x).collect::<Vec<&str>>();
    let entries = profile::get_many(&ids).await?;
    Ok(entries)
}

#[tauri::command]
pub async fn profile_get_projects(
    path: &str,
    cache_behaviour: Option<CacheBehaviour>,
) -> Result<DashMap<String, ProfileFile>> {
    let res = profile::get_projects(path, cache_behaviour).await?;
    Ok(res)
}

// Get a profile's full path
// invoke('plugin:profile|profile_get_full_path',path)
#[tauri::command]
pub async fn profile_get_full_path(path: &str) -> Result<PathBuf> {
    let res = profile::get_full_path(path).await?;
    Ok(res)
}

// Get's a mod's full path
// invoke('plugin:profile|profile_get_mod_full_path',path)
#[tauri::command]
pub async fn profile_get_mod_full_path(
    path: &str,
    project_path: &str,
) -> Result<PathBuf> {
    let res = profile::get_mod_full_path(path, project_path).await?;
    Ok(res)
}

// Get optimal java version from profile
#[tauri::command]
pub async fn profile_get_optimal_jre_key(
    path: &str,
) -> Result<Option<JavaVersion>> {
    let res = profile::get_optimal_jre_key(path).await?;
    Ok(res)
}

// Get a copy of the profile set
// invoke('plugin:profile|profile_list')
#[tauri::command]
pub async fn profile_list() -> Result<Vec<Profile>> {
    let res = profile::list().await?;
    Ok(res)
}

#[tauri::command]
pub async fn profile_check_installed(
    path: &str,
    project_id: &str,
) -> Result<bool> {
    let check_project_id = project_id;

    if let Ok(projects) = profile::get_projects(path, None).await {
        Ok(projects.into_iter().any(|(_, project)| {
            if let Some(metadata) = &project.metadata {
                check_project_id == metadata.project_id
            } else {
                false
            }
        }))
    } else {
        Ok(false)
    }
}

/// Installs/Repairs a profile
/// invoke('plugin:profile|profile_install')
#[tauri::command]
pub async fn profile_install(path: &str, force: bool) -> Result<()> {
    profile::install(path, force).await?;
    Ok(())
}

/// Updates all of the profile's projects
/// invoke('plugin:profile|profile_update_all')
#[tauri::command]
pub async fn profile_update_all(path: &str) -> Result<HashMap<String, String>> {
    Ok(profile::update_all_projects(path).await?)
}

/// Updates a specified project
/// invoke('plugin:profile|profile_update_project')
#[tauri::command]
pub async fn profile_update_project(
    path: &str,
    project_path: &str,
) -> Result<String> {
    Ok(profile::update_project(path, project_path, None).await?)
}

// Adds a project to a profile from a version ID
// invoke('plugin:profile|profile_add_project_from_version')
#[tauri::command]
pub async fn profile_add_project_from_version(
    path: &str,
    version_id: &str,
) -> Result<String> {
    Ok(profile::add_project_from_version(path, version_id).await?)
}

// Adds a project to a profile from a path
// invoke('plugin:profile|profile_add_project_from_path')
#[tauri::command]
pub async fn profile_add_project_from_path(
    path: &str,
    project_path: &Path,
    project_type: Option<ProjectType>,
) -> Result<String> {
    let res = profile::add_project_from_path(path, project_path, project_type)
        .await?;
    Ok(res)
}

// Toggles disabling a project from its path
// invoke('plugin:profile|profile_toggle_disable_project')
#[tauri::command]
pub async fn profile_toggle_disable_project(
    path: &str,
    project_path: &str,
) -> Result<String> {
    Ok(profile::toggle_disable_project(path, project_path).await?)
}

// Removes a project from a profile
// invoke('plugin:profile|profile_remove_project')
#[tauri::command]
pub async fn profile_remove_project(
    path: &str,
    project_path: &str,
) -> Result<()> {
    profile::remove_project(path, project_path).await?;
    Ok(())
}

// Updates a managed Modrinth profile to a version of version_id
#[tauri::command]
pub async fn profile_update_managed_modrinth_version(
    path: String,
    version_id: String,
) -> Result<()> {
    Ok(
        profile::update::update_managed_modrinth_version(&path, &version_id)
            .await?,
    )
}

// Repairs a managed Modrinth profile by updating it to the current version
#[tauri::command]
pub async fn profile_repair_managed_modrinth(path: &str) -> Result<()> {
    Ok(profile::update::repair_managed_modrinth(path).await?)
}

// Exports a profile to a .mrpack file (export_location should end in .mrpack)
// invoke('profile_export_mrpack')
#[tauri::command]
pub async fn profile_export_mrpack(
    path: &str,
    export_location: PathBuf,
    included_overrides: Vec<String>,
    version_id: Option<String>,
    description: Option<String>,
    name: Option<String>, // only used to cache
) -> Result<()> {
    profile::export_mrpack(
        path,
        export_location,
        included_overrides,
        version_id,
        description,
        name,
    )
    .await?;
    Ok(())
}

/// See [`profile::get_pack_export_candidates`]
#[tauri::command]
pub async fn profile_get_pack_export_candidates(
    profile_path: &str,
) -> Result<Vec<String>> {
    let candidates = profile::get_pack_export_candidates(profile_path).await?;
    Ok(candidates)
}

// Run minecraft using a profile using the default credentials
// Returns the UUID, which can be used to poll
// for the actual Child in the state.
// invoke('plugin:profile|profile_run', path)
#[tauri::command]
pub async fn profile_run(path: &str) -> Result<ProcessMetadata> {
    let process = profile::run(path, &QuickPlayType::None).await?;

    Ok(process)
}

// Run Minecraft using a profile using chosen credentials
// Returns the UUID, which can be used to poll
// for the actual Child in the state.
// invoke('plugin:profile|profile_run_credentials', {path, credentials})')
#[tauri::command]
pub async fn profile_run_credentials(
    path: &str,
    credentials: Credentials,
) -> Result<ProcessMetadata> {
    let process =
        profile::run_credentials(path, &credentials, &QuickPlayType::None)
            .await?;

    Ok(process)
}

#[tauri::command]
pub async fn profile_kill(path: &str) -> Result<()> {
    profile::kill(path).await?;

    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditProfile {
    pub name: Option<String>,

    pub game_version: Option<String>,
    pub loader: Option<ModLoader>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub loader_version: Option<Option<String>>,

    pub groups: Option<Vec<String>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub linked_data: Option<Option<LinkedData>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub java_path: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub extra_launch_args: Option<Option<Vec<String>>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub custom_env_vars: Option<Option<Vec<(String, String)>>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub memory: Option<Option<MemorySettings>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub force_fullscreen: Option<Option<bool>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub game_resolution: Option<Option<WindowSize>>,
    pub hooks: Option<Hooks>,
}

// Edits a profile
// invoke('plugin:profile|profile_edit', {path, editProfile})
#[tauri::command]
pub async fn profile_edit(path: &str, edit_profile: EditProfile) -> Result<()> {
    profile::edit(path, |prof| {
        if let Some(name) = edit_profile.name.clone() {
            prof.name = name;
        }
        if let Some(game_version) = edit_profile.game_version.clone() {
            if game_version != prof.game_version {
                prof.protocol_version = None;
            }
            prof.game_version = game_version;
        }
        if let Some(loader) = edit_profile.loader {
            prof.loader = loader;
        }
        if let Some(loader_version) = edit_profile.loader_version.clone() {
            prof.loader_version = loader_version;
        }
        if let Some(linked_data) = edit_profile.linked_data.clone() {
            prof.linked_data = linked_data;
        }
        if let Some(groups) = edit_profile.groups.clone() {
            prof.groups = groups;
        }
        if let Some(java_path) = edit_profile.java_path.clone() {
            prof.java_path = java_path;
        }
        if let Some(memory) = edit_profile.memory {
            prof.memory = memory;
        }
        if let Some(game_resolution) = edit_profile.game_resolution {
            prof.game_resolution = game_resolution;
        }
        if let Some(force_fullscreen) = edit_profile.force_fullscreen {
            prof.force_fullscreen = force_fullscreen;
        }
        if let Some(hooks) = edit_profile.hooks.clone() {
            prof.hooks = hooks;
        }

        prof.modified = chrono::Utc::now();

        if let Some(custom_env_vars) = edit_profile.custom_env_vars.clone() {
            prof.custom_env_vars = custom_env_vars;
        }
        if let Some(extra_launch_args) = edit_profile.extra_launch_args.clone()
        {
            prof.extra_launch_args = extra_launch_args;
        }

        async { Ok(()) }
    })
    .await?;

    Ok(())
}

// Edits a profile's icon
// invoke('plugin:profile|profile_edit_icon')
#[tauri::command]
pub async fn profile_edit_icon(
    path: &str,
    icon_path: Option<&Path>,
) -> Result<()> {
    profile::edit_icon(path, icon_path).await?;
    Ok(())
}
