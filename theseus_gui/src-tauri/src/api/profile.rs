use crate::api::Result;
use daedalus::modded::LoaderVersion;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use theseus::prelude::*;
use uuid::Uuid;

// Remove a profile
// invoke('profile_add_path',path)
#[tauri::command]
pub async fn profile_remove(path: &Path) -> Result<()> {
    profile::remove(path).await?;
    Ok(())
}

// Get a profile by path
// invoke('profile_add_path',path)
#[tauri::command]
pub async fn profile_get(
    path: &Path,
    clear_projects: Option<bool>,
) -> Result<Option<Profile>> {
    let res = profile::get(path, clear_projects).await?;
    Ok(res)
}

// Get optimal java version from profile
#[tauri::command]
pub async fn profile_get_optimal_jre_key(
    path: &Path,
) -> Result<Option<JavaVersion>> {
    let res = profile::get_optimal_jre_key(path).await?;
    Ok(res)
}

// Get a copy of the profile set
// invoke('profile_list')
#[tauri::command]
pub async fn profile_list(
    clear_projects: Option<bool>,
) -> Result<HashMap<PathBuf, Profile>> {
    let res = profile::list(clear_projects).await?;
    Ok(res)
}

#[tauri::command]
pub async fn profile_check_installed(
    path: &Path,
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
/// invoke('profile_install')
#[tauri::command]
pub async fn profile_install(path: &Path) -> Result<()> {
    profile::install(path).await?;
    Ok(())
}

/// Updates all of the profile's projects
/// invoke('profile_update_all')
#[tauri::command]
pub async fn profile_update_all(
    path: &Path,
) -> Result<HashMap<PathBuf, PathBuf>> {
    Ok(profile::update_all(path).await?)
}

/// Updates a specified project
/// invoke('profile_update_project')
#[tauri::command]
pub async fn profile_update_project(
    path: &Path,
    project_path: &Path,
) -> Result<PathBuf> {
    Ok(profile::update_project(path, project_path, None).await?)
}

// Adds a project to a profile from a version ID
// invoke('profile_add_project_from_version')
#[tauri::command]
pub async fn profile_add_project_from_version(
    path: &Path,
    version_id: String,
) -> Result<PathBuf> {
    Ok(profile::add_project_from_version(path, version_id).await?)
}

// Adds a project to a profile from a path
// invoke('profile_add_project_from_path')
#[tauri::command]
pub async fn profile_add_project_from_path(
    path: &Path,
    project_path: &Path,
    project_type: Option<String>,
) -> Result<PathBuf> {
    let res = profile::add_project_from_path(path, project_path, project_type)
        .await?;
    Ok(res)
}

// Toggles disabling a project from its path
// invoke('profile_toggle_disable_project')
#[tauri::command]
pub async fn profile_toggle_disable_project(
    path: &Path,
    project_path: &Path,
) -> Result<PathBuf> {
    Ok(profile::toggle_disable_project(path, project_path).await?)
}

// Removes a project from a profile
// invoke('profile_remove_project')
#[tauri::command]
pub async fn profile_remove_project(
    path: &Path,
    project_path: &Path,
) -> Result<()> {
    profile::remove_project(path, project_path).await?;
    Ok(())
}
// Run minecraft using a profile using the default credentials
// Returns the UUID, which can be used to poll
// for the actual Child in the state.
// invoke('profile_run', path)
#[tauri::command]
pub async fn profile_run(path: &Path) -> Result<Uuid> {
    let minecraft_child = profile::run(path).await?;
    let uuid = minecraft_child.read().await.uuid;
    Ok(uuid)
}

// Run Minecraft using a profile using the default credentials, and wait for the result
// invoke('profile_run_wait', path)
#[tauri::command]
pub async fn profile_run_wait(path: &Path) -> Result<()> {
    let proc_lock = profile::run(path).await?;
    let mut proc = proc_lock.write().await;
    Ok(process::wait_for(&mut proc).await?)
}

// Run Minecraft using a profile using chosen credentials
// Returns the UUID, which can be used to poll
// for the actual Child in the state.
// invoke('profile_run_credentials', {path, credentials})')
#[tauri::command]
pub async fn profile_run_credentials(
    path: &Path,
    credentials: Credentials,
) -> Result<Uuid> {
    let minecraft_child = profile::run_credentials(path, &credentials).await?;
    let uuid = minecraft_child.read().await.uuid;
    Ok(uuid)
}

// Run Minecraft using a profile using the chosen credentials, and wait for the result
// invoke('profile_run_wait', {path, credentials)
#[tauri::command]
pub async fn profile_run_wait_credentials(
    path: &Path,
    credentials: Credentials,
) -> Result<()> {
    let proc_lock = profile::run_credentials(path, &credentials).await?;
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
}

// Edits a profile
// invoke('profile_edit', {path, editProfile})
#[tauri::command]
pub async fn profile_edit(
    path: &Path,
    edit_profile: EditProfile,
) -> Result<()> {
    profile::edit(path, |prof| {
        if let Some(metadata) = edit_profile.metadata.clone() {
            if let Some(name) = metadata.name {
                prof.metadata.name = name
            }
            if let Some(game_version) = metadata.game_version {
                prof.metadata.game_version = game_version
            }
            if let Some(loader) = metadata.loader {
                prof.metadata.loader = loader
            }
            prof.metadata.loader_version = metadata.loader_version
        }

        prof.java = edit_profile.java.clone();
        prof.memory = edit_profile.memory;
        prof.resolution = edit_profile.resolution;
        prof.hooks = edit_profile.hooks.clone();

        async { Ok(()) }
    })
    .await?;

    Ok(())
}

// Edits a profile's icon
// invoke('profile_edit_icon')
#[tauri::command]
pub async fn profile_edit_icon(
    path: &Path,
    icon_path: Option<&Path>,
) -> Result<()> {
    profile::edit_icon(path, icon_path).await?;
    Ok(())
}
