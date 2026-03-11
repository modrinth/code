//! Theseus profile management interface

use std::path::PathBuf;

pub use crate::{
    State,
    state::{Hooks, MemorySettings, Profile, Settings, WindowSize},
};

/// Ensures the default options file exists; creates it empty if missing. Returns its path.
#[tracing::instrument]
pub async fn ensure_default_options_file() -> crate::Result<PathBuf> {
    let state = State::get().await?;
    let path = state.directories.default_options_file_path();
    if !path.exists() {
        tokio::fs::write(&path, []).await?;
    }
    Ok(path)
}

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
pub async fn cancel_directory_change(
    app_identifier: &str,
) -> crate::Result<()> {
    // This is called to handle state initialization errors due to folder migrations
    // failing, so fetching a DB connection pool from `State::get` is not reliable here
    let pool = crate::state::db::connect(app_identifier).await?;
    let mut settings = Settings::get(&pool).await?;

    if let Some(prev_custom_dir) = settings.prev_custom_dir {
        settings.prev_custom_dir = None;
        settings.custom_dir = Some(prev_custom_dir);
    }

    settings.update(&pool).await?;

    Ok(())
}
