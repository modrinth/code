use crate::api::Result;
use std::path::{Path, PathBuf};
use theseus::prelude::*;

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

// Run minecraft using a profile using the default credentials
// Returns a u32 representing the PID, which can be used to poll
// for the actual Child in the state.
// invoke('profile_run', path)
#[tauri::command]
pub async fn profile_run(path: &Path) -> Result<u32> {
    let proc_lock = profile::run(path).await?;
    let pid = proc_lock.read().await.child.id().ok_or_else(|| {
        theseus::Error::from(theseus::ErrorKind::LauncherError(
            "Process failed to stay open.".to_string(),
        ))
    })?;
    Ok(pid)
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
// Returns a u32 representing the PID, which can be used to poll
// for the actual Child in the state.
// invoke('profile_run_credentials', {path, credentials})')
#[tauri::command]
pub async fn profile_run_credentials(path: &Path, credentials: Credentials) -> Result<u32> {
    let proc_lock = profile::run_credentials(path, &credentials).await?;
    let pid = proc_lock.read().await.child.id().ok_or_else(|| {
        theseus::Error::from(theseus::ErrorKind::LauncherError(
            "Process failed to stay open.".to_string(),
        ))
    })?;
    Ok(pid)
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
