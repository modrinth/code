//! Theseus process management interface
use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::state::MinecraftChild;
pub use crate::{
    state::{
        Hooks, JavaSettings, MemorySettings, Profile, Settings, WindowSize,
    },
    State,
};

// Gets whether a child process stored in the state by UUID has finished
#[tracing::instrument]
pub async fn has_finished_by_uuid(uuid: &Uuid) -> crate::Result<bool> {
    Ok(get_exit_status_by_uuid(uuid).await?.is_some())
}

// Gets the exit status of a child process stored in the state by UUID
#[tracing::instrument]
pub async fn get_exit_status_by_uuid(
    uuid: &Uuid,
) -> crate::Result<Option<i32>> {
    let state = State::get().await?;
    let children = state.children.read().await;
    Ok(children.exit_status(uuid).await?.and_then(|f| f.code()))
}

// Gets the UUID of each stored process in the state
#[tracing::instrument]
pub async fn get_all_uuids() -> crate::Result<Vec<Uuid>> {
    let state = State::get().await?;
    let children = state.children.read().await;
    Ok(children.keys())
}

// Gets the UUID of each *running* stored process in the state
#[tracing::instrument]
pub async fn get_all_running_uuids() -> crate::Result<Vec<Uuid>> {
    let state = State::get().await?;
    let children = state.children.read().await;
    children.running_keys().await
}

// Gets the Profile paths of each *running* stored process in the state
#[tracing::instrument]
pub async fn get_all_running_profile_paths() -> crate::Result<Vec<PathBuf>> {
    let state = State::get().await?;
    let children = state.children.read().await;
    children.running_profile_paths().await
}

// Gets the Profiles (cloned) of each *running* stored process in the state
#[tracing::instrument]
pub async fn get_all_running_profiles() -> crate::Result<Vec<Profile>> {
    let state = State::get().await?;
    let children = state.children.read().await;
    children.running_profiles().await
}

// Gets the UUID of each stored process in the state by profile path
#[tracing::instrument]
pub async fn get_uuids_by_profile_path(
    profile_path: &Path,
) -> crate::Result<Vec<Uuid>> {
    let state = State::get().await?;
    let children = state.children.read().await;
    children.running_keys_with_profile(profile_path).await
}

// Gets stdout of a child process stored in the state by UUID, as a string
#[tracing::instrument]
pub async fn get_stdout_by_uuid(uuid: &Uuid) -> crate::Result<String> {
    let state = State::get().await?;
    // Get stdout from child
    let children = state.children.read().await;

    // Extract child or return crate::Error
    if let Some(child) = children.get(uuid) {
        let child = child.read().await;
        Ok(child.stdout.get_output().await?)
    } else {
        Err(crate::ErrorKind::LauncherError(format!(
            "No child process by UUID {}",
            uuid
        ))
        .as_error())
    }
}

// Gets stderr of a child process stored in the state by UUID, as a string
#[tracing::instrument]
pub async fn get_stderr_by_uuid(uuid: &Uuid) -> crate::Result<String> {
    let state = State::get().await?;
    // Get stdout from child
    let children = state.children.read().await;

    // Extract child or return crate::Error
    if let Some(child) = children.get(uuid) {
        let child = child.read().await;
        Ok(child.stderr.get_output().await?)
    } else {
        Err(crate::ErrorKind::LauncherError(format!(
            "No child process with UUID {}",
            uuid
        ))
        .as_error())
    }
}

// Kill a child process stored in the state by UUID, as a string
#[tracing::instrument]
pub async fn kill_by_uuid(uuid: &Uuid) -> crate::Result<()> {
    let state = State::get().await?;
    let children = state.children.read().await;
    if let Some(mchild) = children.get(uuid) {
        let mut mchild = mchild.write().await;
        kill(&mut mchild).await
    } else {
        // No error returned for already finished process
        Ok(())
    }
}

// Wait for a child process stored in the state by UUID
#[tracing::instrument]
pub async fn wait_for_by_uuid(uuid: &Uuid) -> crate::Result<()> {
    let state = State::get().await?;
    let children = state.children.read().await;
    // No error returned for already killed process
    if let Some(mchild) = children.get(uuid) {
        let mut mchild = mchild.write().await;
        wait_for(&mut mchild).await
    } else {
        // No error returned for already finished process
        Ok(())
    }
}

// Kill a running child process directly, and wait for it to be killed
#[tracing::instrument(skip(running))]
pub async fn kill(running: &mut MinecraftChild) -> crate::Result<()> {
    running.current_child.write().await.kill().await?;
    wait_for(running).await
}

// Await on the completion of a child process directly
#[tracing::instrument(skip(running))]
pub async fn wait_for(running: &mut MinecraftChild) -> crate::Result<()> {
    // We do not wait on the Child directly, but wait on the thread manager.
    // This way we can still run all cleanup hook functions that happen after.
    let result = running
        .manager
        .take()
        .ok_or_else(|| {
            crate::ErrorKind::LauncherError(format!(
                "Process manager already completed or missing for process {}",
                running.uuid
            ))
        })?
        .await?
        .map_err(|err| {
            crate::ErrorKind::LauncherError(format!(
                "Error running minecraft: {err}"
            ))
        })?;

    match result.success() {
        false => Err(crate::ErrorKind::LauncherError(format!(
            "Minecraft exited with non-zero code {}",
            result.code().unwrap_or(-1)
        ))
        .as_error()),
        true => Ok(()),
    }
}
