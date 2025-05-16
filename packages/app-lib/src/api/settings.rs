//! Theseus profile management interface

pub use crate::{
    State,
    state::{Hooks, MemorySettings, Profile, Settings, WindowSize},
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
    settings.update(&state.pool).await?;

    Ok(())
}

#[tracing::instrument]
pub async fn cancel_directory_change() -> crate::Result<()> {
    let pool = crate::state::db::connect().await?;
    let mut settings = Settings::get(&pool).await?;

    if let Some(prev_custom_dir) = settings.prev_custom_dir {
        settings.prev_custom_dir = None;
        settings.custom_dir = Some(prev_custom_dir);
    }

    settings.update(&pool).await?;

    Ok(())
}
