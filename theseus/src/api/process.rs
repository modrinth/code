//! Theseus process management interface
use crate::state::MinecraftChild;
pub use crate::{
    state::{
        Hooks, JavaSettings, MemorySettings, Profile, Settings, WindowSize,
    },
    State,
};

// Gets whether a child process stored in the state by PID has finished
#[tracing::instrument]
pub async fn has_finished_by_pid(pid: u32) -> crate::Result<bool> {
    Ok(get_exit_status_by_pid(pid).await?.is_some())
}

// Gets the exit status of a child process stored in the state by PID
#[tracing::instrument]
pub async fn get_exit_status_by_pid(pid: u32) -> crate::Result<Option<i32>> {
    let state = State::get().await?;
    let children = state.children.read().await;
    Ok(children.exit_status(&pid).await?.map(|f| f.code()).flatten())
}

// Gets the PID of each stored process in the state
#[tracing::instrument]
pub async fn get_all_pids() -> crate::Result<Vec<u32>> {
    let state = State::get().await?;
    let children = state.children.read().await;
    Ok(children.keys())
}

// Gets the PID of each *running* stored process in the state
#[tracing::instrument]
pub async fn get_all_running_pids() -> crate::Result<Vec<u32>> 
{
    let state = State::get().await?;
    let children = state.children.read().await;
    Ok(children.running_keys().await)
}


// Gets stdout of a child process stored in the state by PID, as a string
#[tracing::instrument]
pub async fn get_stdout_by_pid(pid : u32) -> crate::Result<String> {
    let state = State::get().await?;
    // Get stdout from child
    let children = state.children.read().await;

    // Extract child or return crate::Error
    if let Some(child) = children.get(&pid) {
        let child = child.read().await;
        Ok(child.stdout.get_output().await?)
    } else {
        Err(crate::ErrorKind::LauncherError(format!(
            "No child process with PID {}",
            pid
        ))
        .as_error())
    }
}

// Gets stderr of a child process stored in the state by PID, as a string
#[tracing::instrument]
pub async fn get_stderr_by_pid(pid : u32) -> crate::Result<String> {
    let state = State::get().await?;
    // Get stdout from child
    let children = state.children.read().await;

    // Extract child or return crate::Error
    if let Some(child) = children.get(&pid) {
        let child = child.read().await;
        Ok(child.stderr.get_output().await?)
    } else {
        Err(crate::ErrorKind::LauncherError(format!(
            "No child process with PID {}",
            pid
        ))
        .as_error())
    }
}

// Kill a child process stored in the state by PID, as a string
#[tracing::instrument]
pub async fn kill_by_pid(pid: u32) -> crate::Result<()> {
    let state = State::get().await?;
    let children = state.children.read().await;
    if let Some(mchild) = children.get(&pid) {
        let mut mchild = mchild.write().await;
        kill(&mut mchild).await
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
    if let Some(mchild) = children.get(&pid) {
        let mut mchild = mchild.write().await;
        wait_for(&mut mchild).await
    } else {
        // No error returned for already finished process
        Ok(())
    }
}

// Kill a running child process directly, and wait for it to be killed
#[tracing::instrument]
pub async fn kill(running: &mut MinecraftChild) -> crate::Result<()> {
    running.child.kill().await?;
    wait_for(running).await
}

// Await on the completion of a child process directly
#[tracing::instrument]
pub async fn wait_for(running: &mut MinecraftChild) -> crate::Result<()> {
    let result = running.child.wait().await.map_err(|err| {
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
