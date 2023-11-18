//! Theseus profile management interface

use std::path::PathBuf;
use tokio::fs;

use io::IOError;
use tokio::sync::RwLock;

use crate::{
    event::emit::{emit_loading, init_loading},
    prelude::DirectoryInfo,
    state::{self, Profiles},
    util::io,
};
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

    if settings.loaded_config_dir
        != state.settings.read().await.loaded_config_dir
    {
        return Err(crate::ErrorKind::OtherError(
            "Cannot change config directory as setting".to_string(),
        )
        .as_error());
    }

    let (reset_io, reset_fetch) = async {
        let read = state.settings.read().await;
        (
            settings.max_concurrent_writes != read.max_concurrent_writes,
            settings.max_concurrent_downloads != read.max_concurrent_downloads,
        )
    }
    .await;

    let updated_discord_rpc = {
        let read = state.settings.read().await;
        settings.disable_discord_rpc != read.disable_discord_rpc
    };

    {
        *state.settings.write().await = settings;
    }

    if updated_discord_rpc {
        state.discord_rpc.clear_to_default(true).await?;
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

/// Sets the new config dir, the location of all Theseus data except for the settings.json and caches
/// Takes control of the entire state and blocks until completion
pub async fn set_config_dir(new_config_dir: PathBuf) -> crate::Result<()> {
    if !new_config_dir.is_dir() {
        return Err(crate::ErrorKind::FSError(format!(
            "New config dir is not a folder: {}",
            new_config_dir.display()
        ))
        .as_error());
    }

    let loading_bar = init_loading(
        crate::LoadingBarType::ConfigChange {
            new_path: new_config_dir.clone(),
        },
        100.0,
        "Changing configuration directory",
    )
    .await?;

    tracing::trace!("Changing config dir, taking control of the state");
    // Take control of the state
    let mut state_write = State::get_write().await?;
    let old_config_dir =
        state_write.directories.config_dir.read().await.clone();

    tracing::trace!("Setting configuration setting");
    // Set load config dir setting
    let settings = {
        let mut settings = state_write.settings.write().await;
        settings.loaded_config_dir = Some(new_config_dir.clone());

        // Some java paths are hardcoded to within our config dir, so we need to update them
        tracing::trace!("Updating java keys");
        for key in settings.java_globals.keys() {
            if let Some(java) = settings.java_globals.get_mut(&key) {
                // If the path is within the old config dir path, update it to the new config dir
                if let Ok(relative_path) = PathBuf::from(java.path.clone())
                    .strip_prefix(&old_config_dir)
                {
                    java.path = new_config_dir
                        .join(relative_path)
                        .to_string_lossy()
                        .to_string();
                }
            }
        }
        tracing::trace!("Syncing settings");

        settings
            .sync(&state_write.directories.settings_file())
            .await?;
        settings.clone()
    };

    tracing::trace!("Reinitializing directory");
    // Set new state information
    state_write.directories = DirectoryInfo::init(&settings)?;
    let total_entries = std::fs::read_dir(&old_config_dir)
        .map_err(|e| IOError::with_path(e, &old_config_dir))?
        .count() as f64;

    // Move all files over from state_write.directories.config_dir to new_config_dir
    tracing::trace!("Renaming folder structure");
    let mut i = 0.0;
    let mut entries = io::read_dir(&old_config_dir).await?;
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| IOError::with_path(e, &old_config_dir))?
    {
        let entry_path = entry.path();
        if let Some(file_name) = entry_path.file_name() {
            // Ignore settings.json
            if file_name == state::SETTINGS_FILE_NAME {
                continue;
            }
            // Ignore caches folder
            if file_name == state::CACHES_FOLDER_NAME {
                continue;
            }
            // Ignore modrinth_logs folder
            if file_name == state::LAUNCHER_LOGS_FOLDER_NAME {
                continue;
            }

            let new_path = new_config_dir.join(file_name);
            io::rename(entry_path, new_path).await?;

            i += 1.0;
            emit_loading(&loading_bar, 90.0 * (i / total_entries), None)
                .await?;
        }
    }

    // Reset file watcher
    tracing::trace!("Reset file watcher");
    let mut file_watcher = state::init_watcher().await?;

    // Reset profiles (for filepaths, file watcher, etc)
    state_write.profiles = RwLock::new(
        Profiles::init(&state_write.directories, &mut file_watcher).await?,
    );
    state_write.file_watcher = RwLock::new(file_watcher);

    emit_loading(&loading_bar, 10.0, None).await?;

    // TODO: need to be able to safely error out of this function, reverting the changes
    tracing::info!(
        "Successfully switched config folder to: {}",
        new_config_dir.display()
    );

    Ok(())
}

pub async fn is_dir_writeable(new_config_dir: PathBuf) -> crate::Result<bool> {
    let temp_path = new_config_dir.join(".tmp");
    match fs::write(temp_path.clone(), "test").await {
        Ok(_) => {
            fs::remove_file(temp_path).await?;
            Ok(true)
        }
        Err(e) => {
            tracing::error!("Error writing to new config dir: {}", e);
            Ok(false)
        }
    }
}
