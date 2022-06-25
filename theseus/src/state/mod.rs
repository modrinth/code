//! Theseus state management system
use std::sync::Arc;
use tokio::sync::{Mutex, OnceCell, RwLock, Semaphore};

// Submodules
mod dirs;
pub use self::dirs::*;

mod metadata;
pub use metadata::*;

mod settings;
pub use settings::*;

mod profiles;
pub use profiles::*;

// Global state
static LAUNCHER_STATE: OnceCell<Arc<State>> = OnceCell::const_new();
#[derive(Debug)]
pub struct State {
    /// Database, used to store some information
    pub(self) database: sled::Db,
    /// Information on the location of files used in the launcher
    pub directories: DirectoryInfo,
    /// Semaphore used to limit concurrent downloads, caps I/O to avoid errors
    pub io_semaphore: Semaphore,
    /// Launcher metadata
    pub metadata: Metadata,
    /// Launcher configuration
    pub settings: RwLock<Settings>,
    /// Launcher profile metadata
    pub profiles: RwLock<Profiles>,
}

impl State {
    /// Get the current launcher state, initializing it if needed
    pub async fn get() -> crate::Result<Arc<Self>> {
        LAUNCHER_STATE
            .get_or_try_init(|| async {
                // Directories
                let directories = DirectoryInfo::init().await?;

                // Database
                // TODO: make database versioned
                let database = sled::open(directories.database_file())?;

                // Settings
                let settings =
                    Settings::init(&directories.settings_file()).await?;

                // Metadata
                let metadata = Metadata::init(&database).await?;

                // Profiles
                let profiles = Profiles::init(&database).await?;

                // Loose initializations
                let io_semaphore =
                    Semaphore::new(settings.max_concurrent_downloads);

                Ok(Arc::new(Self {
                    database,
                    directories,
                    io_semaphore,
                    metadata,
                    settings: RwLock::new(settings),
                    profiles: RwLock::new(profiles),
                }))
            })
            .await
            .map(Arc::clone)
    }

    /// Synchronize in-memory state with persistent state
    pub async fn sync() -> crate::Result<()> {
        let state = Self::get().await?;
        let batch = Arc::new(Mutex::new(sled::Batch::default()));

        let sync_settings = async {
            let state = Arc::clone(&state);

            tokio::spawn(async move {
                let reader = state.settings.read().await;
                reader.sync(&state.directories.settings_file()).await?;
                Ok::<_, crate::Error>(())
            })
            .await
            .unwrap()
        };

        let sync_profiles = async {
            let state = Arc::clone(&state);
            let batch = Arc::clone(&batch);

            tokio::spawn(async move {
                let profiles = state.profiles.read().await;
                let mut batch = batch.lock().await;

                profiles.sync(&mut batch).await?;
                Ok::<_, crate::Error>(())
            })
            .await
            .unwrap()
        };

        tokio::try_join!(sync_settings, sync_profiles)?;

        state
            .database
            .apply_batch(Arc::try_unwrap(batch).unwrap().into_inner())?;
        state.database.flush_async().await?;

        Ok(())
    }
}
