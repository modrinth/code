//! Theseus profile management interface
pub use crate::{
    state::{
        Hooks, JavaSettings, MemorySettings, Profile, Settings, WindowSize,
    },
    State,
};

/// Gets entire settings
#[tracing::instrument]
pub async fn get() -> crate::Result<Settings> {
    let state = State::get().await?;
    let settings = state.settings.read().await;
    Ok(settings.clone())
}

/// Sets entire settings
#[tracing::instrument]
pub async fn set(settings: Settings) -> crate::Result<()> {
    let state = State::get().await?;
    // Replaces the settings struct in the RwLock with the passed argument
    *state.settings.write().await = settings;
    State::sync().await?;
    Ok(())
}
