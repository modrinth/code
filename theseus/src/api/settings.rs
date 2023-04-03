//! Theseus profile management interface
pub use crate::{
    state::{
        Hooks, JavaSettings, MemorySettings, Profile, Settings, WindowSize,
    },
    State,
};
use std::path::PathBuf;
use uuid::Uuid;

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
    Ok(())
}

/// Get memory settings
#[tracing::instrument]
pub async fn get_memory() -> crate::Result<MemorySettings> {
    let state = State::get().await?;
    let settings = state.settings.read().await;
    Ok(settings.memory)
}

/// Set memory settings
#[tracing::instrument]
pub async fn set_memory(memory_settings: MemorySettings) -> crate::Result<()> {
    let state = State::get().await?;
    let mut settings = state.settings.write().await;
    settings.memory = memory_settings;
    Ok(())
}

/// Get game resolution
#[tracing::instrument]
pub async fn get_resolution() -> crate::Result<WindowSize> {
    let state = State::get().await?;
    let settings = state.settings.read().await;
    Ok(settings.game_resolution)
}

/// Set game resolution
#[tracing::instrument]
pub async fn set_resolution(game_resolution: WindowSize) -> crate::Result<()> {
    let state = State::get().await?;
    let mut settings = state.settings.write().await;
    settings.game_resolution = game_resolution;
    Ok(())
}

/// Get custom java args
#[tracing::instrument]
pub async fn get_java_args() -> crate::Result<Vec<String>> {
    let state = State::get().await?;
    let settings = state.settings.read().await;
    Ok(settings.custom_java_args.clone())
}

/// Set custom java args
#[tracing::instrument]
pub async fn set_java_args(custom_java_args: Vec<String>) -> crate::Result<()> {
    let state = State::get().await?;
    let mut settings = state.settings.write().await;
    settings.custom_java_args = custom_java_args;
    Ok(())
}

/// Get custom env args
#[tracing::instrument]
pub async fn get_env_args() -> crate::Result<Vec<(String, String)>> {
    let state = State::get().await?;
    let settings = state.settings.read().await;
    Ok(settings.custom_env_args.clone())
}

/// Set custom env args
#[tracing::instrument]
pub async fn set_env_args(
    custom_env_args: Vec<(String, String)>,
) -> crate::Result<()> {
    let state = State::get().await?;
    let mut settings = state.settings.write().await;
    settings.custom_env_args = custom_env_args;
    Ok(())
}

/// Get java 17 path
#[tracing::instrument]
pub async fn get_java_17_path() -> crate::Result<Option<PathBuf>> {
    let state = State::get().await?;
    let settings = state.settings.read().await;
    Ok(settings.java_17_path.clone())
}

/// Set java 17 path
#[tracing::instrument]
pub async fn set_java_17_path(
    java_17_path: Option<PathBuf>,
) -> crate::Result<()> {
    let state = State::get().await?;
    let mut settings = state.settings.write().await;
    settings.java_17_path = java_17_path;
    Ok(())
}

/// Get java 8 path
#[tracing::instrument]
pub async fn get_java_8_path() -> crate::Result<Option<PathBuf>> {
    let state = State::get().await?;
    let settings = state.settings.read().await;
    Ok(settings.java_8_path.clone())
}

/// Set java 8 path
#[tracing::instrument]
pub async fn set_java_8_path(
    java_8_path: Option<PathBuf>,
) -> crate::Result<()> {
    let state = State::get().await?;
    let mut settings = state.settings.write().await;
    settings.java_8_path = java_8_path;
    Ok(())
}

/// Get default user
#[tracing::instrument]
pub async fn get_default_user() -> crate::Result<Option<Uuid>> {
    let state = State::get().await?;
    let settings = state.settings.read().await;
    Ok(settings.default_user)
}

/// Set default user
#[tracing::instrument]
pub async fn set_default_user(default_user: Option<Uuid>) -> crate::Result<()> {
    let state = State::get().await?;
    let mut settings = state.settings.write().await;
    settings.default_user = default_user;
    Ok(())
}

/// Get hooks
#[tracing::instrument]
pub async fn get_hooks() -> crate::Result<Hooks> {
    let state = State::get().await?;
    let settings = state.settings.read().await;
    Ok(settings.hooks.clone())
}

/// Set hooks
#[tracing::instrument]
pub async fn set_hooks(hooks: Hooks) -> crate::Result<()> {
    let state = State::get().await?;
    let mut settings = state.settings.write().await;
    settings.hooks = hooks;
    Ok(())
}

/// Get max concurrent downloads
#[tracing::instrument]
pub async fn get_max_concurrent_downloads() -> crate::Result<usize> {
    let state = State::get().await?;
    let settings = state.settings.read().await;
    Ok(settings.max_concurrent_downloads)
}

/// Set max concurrent downloads
#[tracing::instrument]
pub async fn set_max_concurrent_downloads(
    max_concurrent_downloads: usize,
) -> crate::Result<()> {
    let state = State::get().await?;
    let mut settings = state.settings.write().await;
    settings.max_concurrent_downloads = max_concurrent_downloads;
    Ok(())
}

/// Get version
#[tracing::instrument]
pub async fn get_version() -> crate::Result<u32> {
    let state = State::get().await?;
    let settings = state.settings.read().await;
    Ok(settings.version)
}

/// Set version
#[tracing::instrument]
pub async fn set_version(version: u32) -> crate::Result<()> {
    let state = State::get().await?;
    let mut settings = state.settings.write().await;
    settings.version = version;
    Ok(())
}
