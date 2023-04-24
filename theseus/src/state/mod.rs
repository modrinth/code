//! Theseus state management system
use crate::event::emit::emit_loading;

use crate::event::emit::init_loading;
use crate::event::LoadingBarType;
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
    pub metadata: RwLock<Metadata>,
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
    /// Get the current launcher state, initializing it if needed
    pub async fn get() -> crate::Result<Arc<Self>> {
        LAUNCHER_STATE
            .get_or_try_init(|| {
                async {
                    let loading_bar = init_loading(
                        LoadingBarType::StateInit,
                        100.0,
                        "Initializing launcher...",
                    )
                    .await?;

                    let directories = DirectoryInfo::init().await?;
                    emit_loading(&loading_bar, 10.0, None).await?;

                    // Settings
                    let settings =
                        Settings::init(&directories.settings_file()).await?;
                    let io_semaphore = RwLock::new(Semaphore::new(
                        settings.max_concurrent_downloads,
                    ));
                    emit_loading(&loading_bar, 10.0, None).await?;

                    let metadata_fut =
                        Metadata::init(&directories, &io_semaphore);
                    let profiles_fut =
                        Profiles::init(&directories, &io_semaphore);
                    let tags_fut = Tags::init(&directories, &io_semaphore);
                    let users_fut = Users::init(&directories, &io_semaphore);
                    // Launcher data
                    let (metadata, profiles, tags, users) = loading_join! {
                        Some(&loading_bar), 70.0, Some("Initializing...");
                        metadata_fut,
                        profiles_fut,
                        tags_fut,
                        users_fut,
                    }?;

                    let children = Children::new();
                    let auth_flow = AuthTask::new();
                    emit_loading(&loading_bar, 10.0, None).await?;

                    Ok(Arc::new(Self {
                        directories,
                        io_semaphore,
                        io_semaphore_max: RwLock::new(
                            settings.max_concurrent_downloads as u32,
                        ),
                        metadata: RwLock::new(metadata),
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

    /// Updates state with data from the web
    pub fn update() {
        tokio::task::spawn(Metadata::update());
        tokio::task::spawn(Tags::update());
        tokio::task::spawn(Profiles::update_projects());
        tokio::task::spawn(Settings::update_java());
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
