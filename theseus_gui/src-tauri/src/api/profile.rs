use crate::api::Result;
use std::path::{Path, PathBuf};
use theseus::prelude::*;

// Add a profile to the in-memory state
// invoke('profile_add',profile)
#[tauri::command]
pub async fn profile_add(profile: Profile) -> Result<()> {
    let res = profile::add(profile).await?;
    State::sync().await?;
    Ok(res)
}

// Add a path as a profile in-memory
// invoke('profile_add_path',path)
#[tauri::command]
pub async fn profile_add_path(path: &Path) -> Result<()> {
    let res = profile::add_path(path).await?;
    State::sync().await?;
    Ok(res)
}

// Remove a profile
// invoke('profile_add_path',path)
#[tauri::command]
pub async fn profile_remove(path: &Path) -> Result<()> {
    let res = profile::remove(path).await?;
    State::sync().await?;
    Ok(res)
}

// Get a profile by path
// invoke('profile_add_path',path)
#[tauri::command]
pub async fn profile_get(path: &Path) -> Result<Option<Profile>> {
    let res = profile::get(path).await?;
    State::sync().await?;
    Ok(res)
}

// Check if a profile is already managed by Theseus
// invoke('profile_is_managed',profile)
#[tauri::command]
pub async fn profile_is_managed(profile: &Path) -> Result<bool> {
    let res = profile::is_managed(profile).await?;
    State::sync().await?;
    Ok(res)
}

// Check if a profile is loaded
// invoke('profile_is_loaded',profile)
#[tauri::command]
pub async fn profile_is_loaded(profile: &Path) -> Result<bool> {
    let res = profile::is_loaded(profile).await?;
    State::sync().await?;
    Ok(res)
}

// Get a copy of the profile set
// invoke('profile_list')
#[tauri::command]
pub async fn profile_list(
) -> Result<std::collections::HashMap<PathBuf, Option<Profile>>> {
    let res = profile::list().await?;
    State::sync().await?;
    Ok(res)
}

// Run Minecraft using a profile
// Returns a u32 representing the PID, which can be used to poll
// for the actual Child in the state.
// invoke('profile_run')
#[tauri::command]
pub async fn profile_run(
    path: &Path,
    credentials: theseus::auth::Credentials,
) -> Result<u32> {
    let proc_lock = profile::run(path, &credentials).await?;
    let pid = proc_lock.read().await.id().ok_or_else(||theseus::Error::from(theseus::ErrorKind::LauncherError(format!(
        "Process failed to stay open."
    ))))?;
    Ok(pid)
}


// Run Minecraft using a profile, and wait for the result
// invoke('profile_wait_for', path, credentials)
#[tauri::command]
pub async fn profile_run_wait(
    path: &Path,
    credentials: theseus::auth::Credentials,
) -> Result<()> {
    let proc_lock = profile::run(path, &credentials).await?;
    let mut proc = proc_lock.write().await; 
    Ok(profile::wait_for(&mut proc).await?)
}
