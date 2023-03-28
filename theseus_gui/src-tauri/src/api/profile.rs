use crate::api::Result;
use crate::models::serializable_child::SerializableChild;
use std::path::{Path, PathBuf};
use theseus::prelude::*;

// Add a profile to the in-memory state
// invoke('profile_add',profile)
#[tauri::command]
pub async fn profile_add(profile: Profile) -> Result<()> {
    Ok(profile::add(profile).await?)
}

// Add a path as a profile in-memory
// invoke('profile_add_path',path)
#[tauri::command]
pub async fn profile_add_path(path: &Path) -> Result<()> {
    Ok(profile::add_path(path).await?)
}

// Remove a profile
// invoke('profile_add_path',path)
#[tauri::command]
pub async fn profile_remove(path: &Path) -> Result<()> {
    Ok(profile::remove(path).await?)
}

// Get a profile by path
// invoke('profile_add_path',path)
#[tauri::command]
pub async fn profile_get(path: &Path) -> Result<Option<Profile>> {
    Ok(profile::get(path).await?)
}

// Check if a profile is already managed by Theseus
// invoke('profile_is_managed',profile)
#[tauri::command]
pub async fn profile_is_managed(profile: &Path) -> Result<bool> {
    Ok(profile::is_managed(profile).await?)
}

// Check if a profile is loaded
// invoke('profile_is_loaded',profile)
#[tauri::command]
pub async fn profile_is_loaded(profile: &Path) -> Result<bool> {
    Ok(profile::is_loaded(profile).await?)
}

// Get a copy of the profile set
// invoke('profile_list')
#[tauri::command]
pub async fn profile_list(
) -> Result<std::collections::HashMap<PathBuf, Option<Profile>>> {
    Ok(profile::list().await?)
}

// Run Minecraft using a profile
// invoke('profile_run')
#[tauri::command]
pub async fn profile_run(
    path: &Path,
    credentials: theseus::auth::Credentials,
) -> Result<SerializableChild> {
    Ok(profile::run(path, &credentials).await?.into())
}

// TODO: We need to create a version of these functions in the API that are able to kill/wait for from the SerializableChild
// Kill a running minecraft process
// invoke('profile_kill')
// #[tauri::command]
// pub async fn profile_kill(running: &mut SerializableChild) -> Result<()> {
//     Ok(profile::kill(running).await?)
// }

// Wait for a running process
// invoke('profile_wait_for')
// #[tauri::command]
// pub async fn profile_wait_for(running: &mut Child) -> Result<()> {
//     Ok(profile::wait_for(running).await?)
// }
