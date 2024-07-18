//! Theseus profile management interface

pub use crate::{
    state::{Hooks, MemorySettings, Profile, Settings, WindowSize},
    State,
};

/// Gets entire settings
#[tracing::instrument]
pub async fn get() -> crate::Result<Settings> {
    let state = State::get().await?;
    let settings = Settings::get(&state.pool).await?;
    Ok(settings)
}

/// Sets entire settings
#[tracing::instrument]
pub async fn set(settings: Settings) -> crate::Result<()> {
    let state = State::get().await?;
    let old_settings = Settings::get(&state.pool).await?;

    if settings.discord_rpc != old_settings.discord_rpc {
        state.discord_rpc.clear_to_default(true).await?;
    }

    settings.update(&state.pool).await?;

    Ok(())
}
