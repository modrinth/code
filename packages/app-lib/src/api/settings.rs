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

    if settings.max_concurrent_writes != old_settings.max_concurrent_writes {
        let mut io_semaphore = state.io_semaphore.0.write().await;
        *io_semaphore =
            tokio::sync::Semaphore::new(settings.max_concurrent_writes);
    }
    if settings.max_concurrent_downloads
        != old_settings.max_concurrent_downloads
    {
        let mut fetch_semaphore = state.fetch_semaphore.0.write().await;
        *fetch_semaphore =
            tokio::sync::Semaphore::new(settings.max_concurrent_downloads);
    }
    if settings.discord_rpc != old_settings.discord_rpc {
        state.discord_rpc.clear_to_default(true).await?;
    }

    settings.update(&state.pool).await?;

    Ok(())
}
