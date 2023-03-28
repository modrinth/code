use std::{path::{Path, PathBuf}, future::Future};
use tokio::process::Child;

use theseus::prelude::*;

const DEFAULT_NAME : &'static str = "";
const PROFILE_FILE_PATH : &'static str = "";

// Add a profile to the in-memory state
// invoke('profile_add',profile)
#[tauri::command]
pub async fn profile_add(profile : Profile) -> theseus::Result<()> {
    Ok(profile::add(profile).await?)
}

// Add a path as a profile in-memory
// invoke('profile_add_path',path)
#[tauri::command]
pub async fn profile_add_path(path: &Path) -> theseus::Result<()> {
    Ok(profile::add_path(path).await?)
}

// Remove a profile
// invoke('profile_add_path',path)
#[tauri::command]
pub async fn profile_remove(path: &Path) -> theseus::Result<()> {
    Ok(profile::remove(path).await?)
}

// Get a profile by path
// invoke('profile_add_path',path)
#[tauri::command]
pub async fn profile_get(path: &Path) -> theseus::Result<Option<Profile>> {
    Ok(profile::get(path).await?)
}

// Check if a profile is already managed by Theseus
// invoke('profile_is_managed',profile)
#[tauri::command]
pub async fn profile_is_managed(profile: &Path) -> theseus::Result<bool> {
    Ok(profile::is_managed(profile).await?)
}

// Check if a profile is loaded
// invoke('profile_is_loaded',profile)
#[tauri::command]
pub async fn profile_is_loaded(profile: &Path) -> theseus::Result<bool> {
    Ok(profile::is_loaded(profile).await?)
}

// Edit a profile using a given asynchronous closure
// invoke('profile_edit',path,action)
#[tauri::command]
pub async fn profile_edit<Fut>(
    path: &Path,
    action: impl Fn(&mut Profile) -> Fut,
) -> theseus::Result<()>
where
    Fut: Future<Output = theseus::Result<()>>,
{
    Ok(profile::edit(path, action).await?)
}

// Get a copy of the profile set
// invoke('profile_list')
#[tauri::command]
pub async fn profile_list(
) -> theseus::Result<std::collections::HashMap<PathBuf, Option<Profile>>> {
    Ok(profile::list().await?)
}

// Run Minecraft using a profile
// invoke('profile_run')
#[tauri::command]
pub async fn profile_run(
    path: &Path,
    credentials: &theseus::auth::Credentials,
) -> theseus::Result<Child> {
    Ok(profile::run(path, credentials).await?)
}

// Kill a running minecraft process
// invoke('profile_kill')
#[tauri::command]
pub async fn profile_kill(running: &mut Child) -> theseus::Result<()> {
    Ok(profile::kill(running).await?)
}

// Wait for a running process
// invoke('profile_wait_for')
#[tauri::command]
pub async fn profile_wait_for(running: &mut Child) -> theseus::Result<()> {
    Ok(profile::wait_for(running).await?)
}
