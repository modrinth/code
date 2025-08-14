//! Theseus process management interface

use crate::state::ProcessMetadata;
pub use crate::{
    State,
    state::{Hooks, MemorySettings, Profile, Settings, WindowSize},
};
use uuid::Uuid;

// Gets the Profile paths of each *running* stored process in the state
#[tracing::instrument]
pub async fn get_all() -> crate::Result<Vec<ProcessMetadata>> {
    let state = State::get().await?;
    let processes = state.process_manager.get_all();
    Ok(processes)
}

// Gets the UUID of each stored process in the state by profile path
#[tracing::instrument]
pub async fn get_by_profile_path(
    profile_path: &str,
) -> crate::Result<Vec<ProcessMetadata>> {
    let state = State::get().await?;
    let processes = state
        .process_manager
        .get_all()
        .into_iter()
        .filter(|x| x.profile_path == profile_path)
        .collect();
    Ok(processes)
}

// Kill a child process stored in the state by UUID, as a string
#[tracing::instrument]
pub async fn kill(uuid: Uuid) -> crate::Result<()> {
    let state = State::get().await?;
    state.process_manager.kill(uuid).await?;

    Ok(())
}

// Wait for a child process stored in the state by UUID
#[tracing::instrument]
pub async fn wait_for(uuid: Uuid) -> crate::Result<()> {
    let state = State::get().await?;
    state.process_manager.wait_for(uuid).await?;

    Ok(())
}
