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
    let (reset_io, reset_fetch) = async {
        let read = state.settings.read().await;
        (
            settings.max_concurrent_writes != read.max_concurrent_writes,
            settings.max_concurrent_downloads != read.max_concurrent_downloads,
        )
    }
    .await;

    {
        *state.settings.write().await = settings;
    }

    if reset_io {
        state.reset_io_semaphore().await;
    }
    if reset_fetch {
        state.reset_fetch_semaphore().await;
    }

    State::sync().await?;
    Ok(())
}
