//! Theseus state management system
use crate::event::emit::{emit_loading, init_loading_unsafe};
use std::path::PathBuf;

use crate::event::LoadingBarType;
use crate::loading_join;

use crate::state::users::Users;
use crate::util::fetch::{FetchSemaphore, IoSemaphore};
use notify::RecommendedWatcher;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{OnceCell, RwLock, Semaphore};

use futures::{channel::mpsc::channel, SinkExt, StreamExt};

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

mod children;
pub use self::children::*;

mod auth_task;
pub use self::auth_task::*;

mod tags;
pub use self::tags::*;

mod java_globals;
pub use self::java_globals::*;

mod safe_processes;
pub use self::safe_processes::*;

// Global state
// RwLock on state only has concurrent reads, except for config dir change which takes control of the State
static LAUNCHER_STATE: OnceCell<RwLock<State>> = OnceCell::const_new();
pub struct State {
    /// Information on the location of files used in the launcher
    pub directories: DirectoryInfo,

    /// Semaphore used to limit concurrent network requests and avoid errors
    pub fetch_semaphore: FetchSemaphore,
    /// Stored maximum number of sempahores of current fetch_semaphore
    pub fetch_semaphore_max: RwLock<u32>,
    /// Semaphore used to limit concurrent I/O and avoid errors
    pub io_semaphore: IoSemaphore,
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
    /// Launcher processes that should be safely exited on shutdown
    pub(crate) safety_processes: RwLock<SafeProcesses>,

    /// File watcher debouncer
    pub(crate) file_watcher: RwLock<Debouncer<RecommendedWatcher>>,
}

impl State {
    /// Get the current launcher state, initializing it if needed
    pub async fn get(
    ) -> crate::Result<Arc<tokio::sync::RwLockReadGuard<'static, Self>>> {
        Ok(Arc::new(
            LAUNCHER_STATE
                .get_or_try_init(Self::initialize_state)
                .await?
                .read()
                .await,
        ))
    }

    /// Get the current launcher state, initializing it if needed
    /// Takes writing control of the state, blocking all other uses of it
    /// Only used for state change such as changing the config directory
    pub async fn get_write(
    ) -> crate::Result<tokio::sync::RwLockWriteGuard<'static, Self>> {
        Ok(LAUNCHER_STATE
            .get_or_try_init(Self::initialize_state)
            .await?
            .write()
            .await)
    }

    #[tracing::instrument]
    #[theseus_macros::debug_pin]
    async fn initialize_state() -> crate::Result<RwLock<State>> {
        let loading_bar = init_loading_unsafe(
            LoadingBarType::StateInit,
            100.0,
            "Initializing launcher",
        )
        .await?;

        // Settings
        let settings =
            Settings::init(&DirectoryInfo::get_initial_settings_file()?)
                .await?;

        let directories = DirectoryInfo::init(&settings)?;

        emit_loading(&loading_bar, 10.0, None).await?;

        let mut file_watcher = init_watcher().await?;

        let fetch_semaphore = FetchSemaphore(RwLock::new(Semaphore::new(
            settings.max_concurrent_downloads,
        )));
        let io_semaphore = IoSemaphore(RwLock::new(Semaphore::new(
            settings.max_concurrent_writes,
        )));
        emit_loading(&loading_bar, 10.0, None).await?;

        let metadata_fut = Metadata::init(&directories, &io_semaphore);
        let profiles_fut = Profiles::init(&directories, &mut file_watcher);
        let tags_fut =
            Tags::init(&directories, &io_semaphore, &fetch_semaphore);
        let users_fut = Users::init(&directories, &io_semaphore);
        // Launcher data
        let (metadata, profiles, tags, users) = loading_join! {
            Some(&loading_bar), 70.0, Some("Loading metadata");
            metadata_fut,
            profiles_fut,
            tags_fut,
            users_fut,
        }?;

        let children = Children::new();
        let auth_flow = AuthTask::new();
        let safety_processes = SafeProcesses::new();
        emit_loading(&loading_bar, 10.0, None).await?;

        Ok::<RwLock<Self>, crate::Error>(RwLock::new(Self {
            directories,
            fetch_semaphore,
            fetch_semaphore_max: RwLock::new(
                settings.max_concurrent_downloads as u32,
            ),
            io_semaphore,
            io_semaphore_max: RwLock::new(
                settings.max_concurrent_writes as u32,
            ),
            metadata: RwLock::new(metadata),
            settings: RwLock::new(settings),
            profiles: RwLock::new(profiles),
            users: RwLock::new(users),
            children: RwLock::new(children),
            auth_flow: RwLock::new(auth_flow),
            tags: RwLock::new(tags),
            safety_processes: RwLock::new(safety_processes),
            file_watcher: RwLock::new(file_watcher),
        }))
    }

    /// Updates state with data from the web
    pub fn update() {
        tokio::task::spawn(Metadata::update());
        tokio::task::spawn(Tags::update());
        tokio::task::spawn(Profiles::update_projects());
        tokio::task::spawn(Settings::update_java());
    }

    #[tracing::instrument]
    #[theseus_macros::debug_pin]
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

    /// Reset IO semaphore to default values
    /// This will block until all uses of the semaphore are complete, so it should only be called
    /// when we are not in the middle of downloading something (ie: changing the settings!)
    pub async fn reset_io_semaphore(&self) {
        let settings = self.settings.read().await;
        let mut io_semaphore = self.io_semaphore.0.write().await;
        let mut total_permits = self.io_semaphore_max.write().await;

        // Wait to get all permits back
        let _ = io_semaphore.acquire_many(*total_permits).await;

        // Reset the semaphore
        io_semaphore.close();
        *total_permits = settings.max_concurrent_writes as u32;
        *io_semaphore = Semaphore::new(settings.max_concurrent_writes);
    }

    /// Reset IO semaphore to default values
    /// This will block until all uses of the semaphore are complete, so it should only be called
    /// when we are not in the middle of downloading something (ie: changing the settings!)
    pub async fn reset_fetch_semaphore(&self) {
        let settings = self.settings.read().await;
        let mut io_semaphore = self.fetch_semaphore.0.write().await;
        let mut total_permits = self.fetch_semaphore_max.write().await;

        // Wait to get all permits back
        let _ = io_semaphore.acquire_many(*total_permits).await;

        // Reset the semaphore
        io_semaphore.close();
        *total_permits = settings.max_concurrent_downloads as u32;
        *io_semaphore = Semaphore::new(settings.max_concurrent_downloads);
    }
}

pub async fn init_watcher() -> crate::Result<Debouncer<RecommendedWatcher>> {
    let (mut tx, mut rx) = channel(1);

    let file_watcher = new_debouncer(
        Duration::from_secs(2),
        None,
        move |res: DebounceEventResult| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
    )?;

    tokio::task::spawn(async move {
        while let Some(res) = rx.next().await {
            match res {
                Ok(mut events) => {
                    let mut visited_paths = Vec::new();
                    // sort events by e.path
                    events.sort_by(|a, b| a.path.cmp(&b.path));
                    events.iter().for_each(|e| {
                        tracing::debug!(
                            "File watcher event: {:?}",
                            serde_json::to_string(&e.path).unwrap()
                        );
                        let mut new_path = PathBuf::new();
                        let mut components_iterator = e.path.components();
                        let mut found = false;
                        for component in components_iterator.by_ref() {
                            new_path.push(component);
                            if found {
                                break;
                            }
                            if component.as_os_str() == "profiles" {
                                found = true;
                            }
                        }
                        // if any remain, it's a subfile of the profile folder and not the profile folder itself
                        let subfile = components_iterator.next().is_some();

                        // At this point, new_path is the path to the profile, and subfile is whether it's a subfile of the profile or not
                        let profile_path_id =
                            ProfilePathId::new(&PathBuf::from(
                                new_path.file_name().unwrap_or_default(),
                            ));

                        if e.path
                            .components()
                            .any(|x| x.as_os_str() == "crash-reports")
                            && e.path
                                .extension()
                                .map(|x| x == "txt")
                                .unwrap_or(false)
                        {
                            Profile::crash_task(profile_path_id);
                        } else if !visited_paths.contains(&new_path) {
                            if subfile {
                                Profile::sync_projects_task(profile_path_id);
                                visited_paths.push(new_path);
                            } else {
                                Profiles::sync_available_profiles_task(
                                    profile_path_id,
                                );
                            }
                        }
                    });
                }
                Err(errors) => errors.iter().for_each(|err| {
                    tracing::warn!("Unable to watch file: {err}")
                }),
            }
        }
    });

    Ok(file_watcher)
}
