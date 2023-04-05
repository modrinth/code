//! Theseus state management system
use crate::config::sled_config;
use std::sync::Arc;
use tokio::sync::{Mutex, OnceCell, RwLock, Semaphore};

// Submodules
mod dirs;
pub use self::dirs::*;

mod metadata;
pub use self::metadata::*;

mod profiles;
pub use self::profiles::*;

mod settings;
pub use self::settings::*;

mod projects;
pub use self::projects::*;

mod users;
pub use self::users::*;

mod children;
pub use self::children::*;

mod auth_task;
pub use self::auth_task::*;

mod tags;
pub use self::tags::*;

// Global state
static LAUNCHER_STATE: OnceCell<Arc<State>> = OnceCell::const_new();
pub struct State {
    /// Database, used to store some information
    pub(self) database: sled::Db,
    /// Information on the location of files used in the launcher
    pub directories: DirectoryInfo,
    /// Semaphore used to limit concurrent I/O and avoid errors
    pub io_semaphore: Semaphore,
    /// Launcher metadata
    pub metadata: Metadata,
    // TODO: settings API
    /// Launcher configuration
    pub settings: RwLock<Settings>,
    /// Reference to minecraft process children
    pub children: RwLock<Children>,
    /// Authentication flow
    pub auth_flow: RwLock<AuthTask>,
    /// Launcher profile metadata
    pub(crate) profiles: RwLock<Profiles>,
    /// Launcher user account info
    pub(crate) users: RwLock<Users>,
    /// Launcher tags
    pub(crate) tags: RwLock<Tags>,
}

impl State {
    #[tracing::instrument]
    /// Get the current launcher state, initializing it if needed
    pub async fn get() -> crate::Result<Arc<Self>> {
        LAUNCHER_STATE
            .get_or_try_init(|| {
                async {
                    // Directories
                    let directories = DirectoryInfo::init().await?;

                    // Database
                    // TODO: make database versioned
                    let database = sled_config()
                        .path(directories.database_file())
                        .open()?;

                    // Settings
                    let settings =
                        Settings::init(&directories.settings_file()).await?;

                    // Loose initializations
                    let io_semaphore =
                        Semaphore::new(settings.max_concurrent_downloads);

                    // Launcher data
                    let (metadata, profiles) = tokio::try_join! {
                        Metadata::init(&database),
                        Profiles::init(&database, &directories, &io_semaphore),
                    }?;
                    let users = Users::init(&database)?;

                    let children = Children::new();

                    let auth_flow = AuthTask::new();

                    // On launcher initialization, attempt a tag fetch after tags init
                    let mut tags = Tags::init(&database)?;
                    if let Err(tag_fetch_err) = tags.fetch_update().await {
                        tracing::error!(
                            "Failed to fetch tags on launcher init: {}",
                            tag_fetch_err
                        );
                    };

                    Ok(Arc::new(Self {
                        database,
                        directories,
                        io_semaphore,
                        metadata,
                        settings: RwLock::new(settings),
                        profiles: RwLock::new(profiles),
                        users: RwLock::new(users),
                        children: RwLock::new(children),
                        auth_flow: RwLock::new(auth_flow),
                        tags: RwLock::new(tags),
                    }))
                }
            })
            .await
            .map(Arc::clone)
    }

    #[tracing::instrument]
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
            .await?
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
            .await?
        };

        tokio::try_join!(sync_settings, sync_profiles)?;

        state.database.apply_batch(
            Arc::try_unwrap(batch)
                .expect("Error saving state by acquiring Arc")
                .into_inner(),
        )?;
        state.database.flush_async().await?;

        Ok(())
    }
}
