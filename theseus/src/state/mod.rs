//! Theseus state management system
use crate::config::sled_config;
use crate::emit_loading;
use crate::init_loading;
use crate::jre;
use crate::loading_join;
use std::sync::Arc;
use tokio::sync::{OnceCell, RwLock, Semaphore};

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

mod java_globals;
pub use self::java_globals::*;

// Global state
static LAUNCHER_STATE: OnceCell<Arc<State>> = OnceCell::const_new();
pub struct State {
    /// Information on the location of files used in the launcher
    pub directories: DirectoryInfo,
    /// Semaphore used to limit concurrent I/O and avoid errors
    pub io_semaphore: RwLock<Semaphore>,
    /// Stored maximum number of sempahores of current io_semaphore
    pub io_semaphore_max: RwLock<u32>,
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

                    init_loading("launcher_init", 100.0, "Initializing launcher...").await;

                    // Directories
                    let directories = DirectoryInfo::init().await?;

                    // Database
                    // TODO: make database versioned
                    let database = sled_config()
                        .path(directories.database_file())
                        .open()?;

                    emit_loading("launcher_init", 10.0, None).await;

                    // Settings
                    let mut settings =
                        Settings::init(&directories.settings_file()).await?;

                    // Loose initializations
                    let io_semaphore_max = settings.max_concurrent_downloads;

                    let io_semaphore =
                        RwLock::new(Semaphore::new(io_semaphore_max));

                    let metadata_fut = Metadata::init(&database);
                    let profiles_fut =
                        Profiles::init(&directories, &io_semaphore);

                    // Launcher data
                    let (metadata, profiles) = loading_join! {
                        "launcher_init", 20.0, Some("Initializing metadata and profiles...");
                        metadata_fut, profiles_fut
                    };

                    emit_loading("launcher_init", 10.0, None).await;
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

                    emit_loading("launcher_init", 10.0, None).await;

                    // On launcher initialization, if global java variables are unset, try to find and set them
                    // (they are required for the game to launch)
                    if settings.java_globals.count() == 0 {
                        settings.java_globals = jre::autodetect_java_globals()?;
                    }

                    Ok(Arc::new(Self {
                        directories,
                        io_semaphore,
                        io_semaphore_max: RwLock::new(io_semaphore_max as u32),
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

            tokio::spawn(async move {
                let profiles = state.profiles.read().await;

                profiles.sync().await?;
                Ok::<_, crate::Error>(())
            })
            .await?
        };

        tokio::try_join!(sync_settings, sync_profiles)?;

        Ok(())
    }

    /// Reset semaphores to default values
    /// This will block until all uses of the semaphore are complete, so it should only be called
    /// when we are not in the middle of downloading something (ie: changing the settings!)
    pub async fn reset_semaphore(&self) {
        let settings = self.settings.read().await;
        let mut io_semaphore = self.io_semaphore.write().await;
        let mut total_permits = self.io_semaphore_max.write().await;

        // Wait to get all permits back
        let _ = io_semaphore.acquire_many(*total_permits).await;

        // Reset the semaphore
        io_semaphore.close();
        *total_permits = settings.max_concurrent_downloads as u32;
        *io_semaphore = Semaphore::new(settings.max_concurrent_downloads);
    }
}
