//! Theseus process management interface
use std::process::ExitStatus;

use tokio::process::Child;
use tokio::io::AsyncReadExt;

pub use crate::{
    state::{
        Hooks, JavaSettings, MemorySettings, Profile, Settings, WindowSize,
    },
    State,
};

// Gets whether a child process stored in the state by PID has finished
#[tracing::instrument]
pub async fn has_finished_by_pid(pid: u32) -> crate::Result<bool> {
    Ok(!get_exit_status_by_pid(pid).await?.is_some())
}

// Gets the exit status of a child process stored in the state by PID
#[tracing::instrument]
pub async fn get_exit_status_by_pid(pid: u32) -> crate::Result<Option<ExitStatus>> {
    let state = State::get().await?;
    let children = state.children.read().await;
    Ok(children.exit_status(&pid).await?)
}

// Gets the PID of each stored process in the state
#[tracing::instrument]
pub async fn get_all_process_pid() -> crate::Result<Vec<u32>> {
    let state = State::get().await?;
    let children = state.children.read().await;
    Ok(children.keys())
}

// Gets the PID of each *running* stored process in the state
#[tracing::instrument]
pub async fn get_all_running_process_pid() -> crate::Result<Vec<u32>> 
{
    let state = State::get().await?;
    let children = state.children.read().await;
    Ok(children.running_keys().await)
}

// Gets stderr of a child process stored in the state by PID
#[tracing::instrument]
pub async fn get_stderr_as_string_by_pid(pid : u32) -> crate::Result<String> {
    let state = State::get().await?;
    let children = state.children.read().await;
    let child = children.get(&pid).unwrap();
    let mut child = child.write().await;

    let mut stderr = child.stderr.take().unwrap();
    let mut stderr_string = String::new();
    stderr.read_to_string(&mut stderr_string).await?;
    Ok(stderr_string)
}

// Gets stdout of a child process stored in the state by PID, as a string
#[tracing::instrument]
pub async fn get_stdout_as_string_by_pid(pid : u32) -> crate::Result<String> {
    let state = State::get().await?;
    let children = state.children.read().await;
    let child = children.get(&pid).unwrap();
    let mut child = child.write().await;

    let mut stdout = child.stdout.take().unwrap();
    let mut stdout_string = String::new();
    stdout.read_to_string(&mut stdout_string).await?;
    Ok(stdout_string)
}

// Kill a child process stored in the state by PID, as a string
#[tracing::instrument]
pub async fn kill_by_pid(pid: u32) -> crate::Result<()> {
    let state = State::get().await?;
    let children = state.children.read().await;
    if let Some(child) = children.get(&pid) {
        let mut child = child.write().await;
        kill(&mut child).await
    } else {
        // No error returned for already finished process
        Ok(())
    }
}

// Wait for a child process stored in the state by PID
#[tracing::instrument]
pub async fn wait_for_by_pid(pid: u32) -> crate::Result<()> {
    let state = State::get().await?;
    let children = state.children.read().await;
    // No error returned for already killed process
    if let Some(child) = children.get(&pid) {
        let mut child = child.write().await;
        wait_for(&mut child).await
    } else {
        // No error returned for already finished process
        Ok(())
    }
}

// Kill a running child process directly, and wait for it to be killed
#[tracing::instrument]
pub async fn kill(running: &mut Child) -> crate::Result<()> {
    running.kill().await?;
    wait_for(running).await
}

// Await on the completion of a child process directly
#[tracing::instrument]
pub async fn wait_for(running: &mut Child) -> crate::Result<()> {
    let result = running.wait().await.map_err(|err| {
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
