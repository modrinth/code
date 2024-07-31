//! Theseus process management interface

use crate::state::Process;
pub use crate::{
    state::{Hooks, MemorySettings, Profile, Settings, WindowSize},
    State,
};

// Gets the Profile paths of each *running* stored process in the state
#[tracing::instrument]
pub async fn get_all() -> crate::Result<Vec<Process>> {
    let state = State::get().await?;
    let processes = Process::get_all(&state.pool).await?;
    Ok(processes)
}

// Gets the UUID of each stored process in the state by profile path
#[tracing::instrument]
pub async fn get_by_profile_path(
    profile_path: &str,
) -> crate::Result<Vec<Process>> {
    let state = State::get().await?;
    let processes =
        Process::get_from_profile(profile_path, &state.pool).await?;
    Ok(processes)
}

// Kill a child process stored in the state by UUID, as a string
#[tracing::instrument]
pub async fn kill(pid: i32) -> crate::Result<()> {
    let state = State::get().await?;
    let process = Process::get(pid, &state.pool).await?;

    if let Some(process) = process {
        process.kill().await?;

        Ok(())
    } else {
        Ok(())
    }
}

// Wait for a child process stored in the state by UUID
#[tracing::instrument]
pub async fn wait_for(pid: i32) -> crate::Result<()> {
    let state = State::get().await?;
    let process = Process::get(pid, &state.pool).await?;

    if let Some(process) = process {
        process.wait_for().await?;

        Ok(())
    } else {
        Ok(())
    }
}
