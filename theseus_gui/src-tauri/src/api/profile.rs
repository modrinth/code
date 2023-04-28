use crate::api::Result;
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
pub async fn profile_get(path: &Path) -> Result<Option<Profile>> {
    let res = profile::get(path).await?;
    Ok(res)
}

// Get a copy of the profile set
// invoke('profile_list')
#[tauri::command]
pub async fn profile_list(
) -> Result<std::collections::HashMap<PathBuf, Profile>> {
    let res = profile::list().await?;
    Ok(res)
}

/// Syncs a profile's in memory state with the state on the disk
/// // invoke('profile_sync')
#[tauri::command]
pub async fn profile_sync(path: &Path) -> Result<()> {
    profile::sync(path).await?;
    Ok(())
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
pub async fn profile_update_all(path: &Path) -> Result<()> {
    profile::update_all(path).await?;
    Ok(())
}

/// Updates a specified project
/// invoke('profile_update_project')
#[tauri::command]
pub async fn profile_update_project(
    path: &Path,
    project_path: &Path,
) -> Result<()> {
    profile::update_project(path, project_path, None).await?;
    Ok(())
}

/// Replaces a project with the given version ID
/// invoke('profile_replace_project')
#[tauri::command]
pub async fn profile_replace_project(
    path: &Path,
    project: &Path,
    version_id: String,
) -> Result<PathBuf> {
    let res = profile::replace_project(path, project, version_id).await?;
    Ok(res)
}

// Adds a project to a profile from a version ID
// invoke('profile_add_project_from_version')
#[tauri::command]
pub async fn profile_add_project_from_version(
    path: &Path,
    version_id: String,
) -> Result<PathBuf> {
    let res = profile::add_project_from_version(path, version_id).await?;
    Ok(res)
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
// Returns the new filename of the object (added or removed .disabled)
// invoke('profile_toggle_disable_project')
#[tauri::command]
pub async fn profile_toggle_disable_project(
    path: &Path,
    project_path: &Path,
) -> Result<ChangedFilename> {
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
