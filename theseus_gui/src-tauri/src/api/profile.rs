use crate::api::Result;
use daedalus::modded::LoaderVersion;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use theseus::prelude::*;
use uuid::Uuid;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("profile")
        .invoke_handler(tauri::generate_handler![
            profile_remove,
            profile_get,
            profile_get_optimal_jre_key,
            profile_list,
            profile_check_installed,
            profile_install,
            profile_update_all,
            profile_update_project,
            profile_add_project_from_version,
            profile_add_project_from_path,
            profile_toggle_disable_project,
            profile_remove_project,
            profile_run,
            profile_run_wait,
            profile_run_credentials,
            profile_run_wait_credentials,
            profile_edit,
            profile_edit_icon,
            profile_export_mrpack,
            profile_get_potential_override_folders,
        ])
        .build()
}

// Remove a profile
// invoke('plugin:profile|profile_add_path',path)
#[tauri::command]
pub async fn profile_remove(path: ProfilePathId) -> Result<()> {
    profile::remove(&path).await?;
    Ok(())
}

// Get a profile by path
// invoke('plugin:profile|profile_add_path',path)
#[tauri::command]
pub async fn profile_get(
    path: ProfilePathId,
    clear_projects: Option<bool>,
) -> Result<Option<Profile>> {
    let res = profile::get(&path, clear_projects).await?;
    Ok(res)
}

// Get optimal java version from profile
#[tauri::command]
pub async fn profile_get_optimal_jre_key(
    path: ProfilePathId,
) -> Result<Option<JavaVersion>> {
    let res = profile::get_optimal_jre_key(&path).await?;
    Ok(res)
}

// Get a copy of the profile set
// invoke('plugin:profile|profile_list')
#[tauri::command]
pub async fn profile_list(
    clear_projects: Option<bool>,
) -> Result<HashMap<ProfilePathId, Profile>> {
    let res = profile::list(clear_projects).await?;
    Ok(res)
}

#[tauri::command]
pub async fn profile_check_installed(
    path: ProfilePathId,
    project_id: String,
) -> Result<bool> {
    let profile = profile_get(path, None).await?;
    if let Some(profile) = profile {
        Ok(profile.projects.into_iter().any(|(_, project)| {
            if let ProjectMetadata::Modrinth { project, .. } = &project.metadata
            {
                project.id == project_id
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
pub async fn profile_install(path: ProfilePathId) -> Result<()> {
    profile::install(&path).await?;
    Ok(())
}

/// Updates all of the profile's projects
/// invoke('plugin:profile|profile_update_all')
#[tauri::command]
pub async fn profile_update_all(
    path: ProfilePathId,
) -> Result<HashMap<ProjectPathId, ProjectPathId>> {
    Ok(profile::update_all(&path).await?)
}

/// Updates a specified project
/// invoke('plugin:profile|profile_update_project')
#[tauri::command]
pub async fn profile_update_project(
    path: ProfilePathId,
    project_path: ProjectPathId,
) -> Result<ProjectPathId> {
    Ok(profile::update_project(&path, &project_path, None).await?)
}

// Adds a project to a profile from a version ID
// invoke('plugin:profile|profile_add_project_from_version')
#[tauri::command]
pub async fn profile_add_project_from_version(
    path: ProfilePathId,
    version_id: String,
) -> Result<ProjectPathId> {
    Ok(profile::add_project_from_version(&path, version_id).await?)
}

// Adds a project to a profile from a path
// invoke('plugin:profile|profile_add_project_from_path')
#[tauri::command]
pub async fn profile_add_project_from_path(
    path: ProfilePathId,
    project_path: &Path,
    project_type: Option<String>,
) -> Result<ProjectPathId> {
    let res = profile::add_project_from_path(&path, project_path, project_type)
        .await?;
    Ok(res)
}

// Toggles disabling a project from its path
// invoke('plugin:profile|profile_toggle_disable_project')
#[tauri::command]
pub async fn profile_toggle_disable_project(
    path: ProfilePathId,
    project_path: ProjectPathId,
) -> Result<ProjectPathId> {
    Ok(profile::toggle_disable_project(&path, &project_path).await?)
}

// Removes a project from a profile
// invoke('plugin:profile|profile_remove_project')
#[tauri::command]
pub async fn profile_remove_project(
    path: ProfilePathId,
    project_path: ProjectPathId,
) -> Result<()> {
    profile::remove_project(&path, &project_path).await?;
    Ok(())
}

// Exports a profile to a .mrpack file (export_location should end in .mrpack)
// invoke('profile_export_mrpack')
#[tauri::command]
pub async fn profile_export_mrpack(
    path: ProfilePathId,
    export_location: PathBuf,
    included_overrides: Vec<String>,
    version_id: Option<String>,
) -> Result<()> {
    profile::export_mrpack(
        &path,
        export_location,
        included_overrides,
        version_id,
    )
    .await?;
    Ok(())
}

// Given a folder path, populate a Vec of all the subfolders
// Intended to be used for finding potential override folders
// profile
// -- folder1
// -- folder2
// -- file1
// => [folder1, folder2]
#[tauri::command]
pub async fn profile_get_potential_override_folders(
    profile_path: ProfilePathId,
) -> Result<Vec<PathBuf>> {
    let overrides =
        profile::get_potential_override_folders(profile_path).await?;
    Ok(overrides)
}

// Run minecraft using a profile using the default credentials
// Returns the UUID, which can be used to poll
// for the actual Child in the state.
// invoke('plugin:profile|profile_run', path)
#[tauri::command]
pub async fn profile_run(path: ProfilePathId) -> Result<Uuid> {
    let minecraft_child = profile::run(&path).await?;
    let uuid = minecraft_child.read().await.uuid;
    Ok(uuid)
}

// Run Minecraft using a profile using the default credentials, and wait for the result
// invoke('plugin:profile|profile_run_wait', path)
#[tauri::command]
pub async fn profile_run_wait(path: ProfilePathId) -> Result<()> {
    let proc_lock = profile::run(&path).await?;
    let mut proc = proc_lock.write().await;
    Ok(process::wait_for(&mut proc).await?)
}

// Run Minecraft using a profile using chosen credentials
// Returns the UUID, which can be used to poll
// for the actual Child in the state.
// invoke('plugin:profile|profile_run_credentials', {path, credentials})')
#[tauri::command]
pub async fn profile_run_credentials(
    path: ProfilePathId,
    credentials: Credentials,
) -> Result<Uuid> {
    let minecraft_child = profile::run_credentials(&path, &credentials).await?;
    let uuid = minecraft_child.read().await.uuid;

    Ok(uuid)
}

// Run Minecraft using a profile using the chosen credentials, and wait for the result
// invoke('plugin:profile|profile_run_wait', {path, credentials)
#[tauri::command]
pub async fn profile_run_wait_credentials(
    path: ProfilePathId,
    credentials: Credentials,
) -> Result<()> {
    let proc_lock = profile::run_credentials(&path, &credentials).await?;
    let mut proc = proc_lock.write().await;
    Ok(process::wait_for(&mut proc).await?)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditProfile {
    pub metadata: Option<EditProfileMetadata>,
    pub java: Option<JavaSettings>,
    pub memory: Option<MemorySettings>,
    pub resolution: Option<WindowSize>,
    pub hooks: Option<Hooks>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EditProfileMetadata {
    pub name: Option<String>,
    pub game_version: Option<String>,
    pub loader: Option<ModLoader>,
    pub loader_version: Option<LoaderVersion>,
    pub groups: Option<Vec<String>>,
}

// Edits a profile
// invoke('plugin:profile|profile_edit', {path, editProfile})
#[tauri::command]
pub async fn profile_edit(
    path: ProfilePathId,
    edit_profile: EditProfile,
) -> Result<()> {
    profile::edit(&path, |prof| {
        if let Some(metadata) = edit_profile.metadata.clone() {
            if let Some(name) = metadata.name {
                prof.metadata.name = name;
            }
            if let Some(game_version) = metadata.game_version {
                prof.metadata.game_version = game_version;
            }
            if let Some(loader) = metadata.loader {
                prof.metadata.loader = loader;
            }
            prof.metadata.loader_version = metadata.loader_version;

            if let Some(groups) = metadata.groups {
                prof.metadata.groups = groups;
            }
        }

        prof.java = edit_profile.java.clone();
        prof.memory = edit_profile.memory;
        prof.resolution = edit_profile.resolution;
        prof.hooks = edit_profile.hooks.clone();

        prof.metadata.date_modified = chrono::Utc::now();

        async { Ok(()) }
    })
    .await?;
    State::sync().await?;

    Ok(())
}

// Edits a profile's icon
// invoke('plugin:profile|profile_edit_icon')
#[tauri::command]
pub async fn profile_edit_icon(
    path: ProfilePathId,
    icon_path: Option<&Path>,
) -> Result<()> {
    profile::edit_icon(&path, icon_path).await?;
    Ok(())
}
