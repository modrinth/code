//! Theseus state management system
use crate::util::fetch::{FetchSemaphore, IoSemaphore};
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::{Mutex, MutexGuard, OnceCell, OwnedMutexGuard, Semaphore};

use crate::state::instances::watcher::FileWatcher;
use sqlx::SqlitePool;

// Submodules
mod dirs;
pub use self::dirs::*;

mod instance_types;
pub use self::instance_types::*;

pub(crate) mod instances;
pub use self::instances::*;
pub(crate) use self::instances::{StoredOption, StoredPreference};
pub(crate) use self::instances::{
    game_options_sync_is_enabled, load_game_option_preferences,
    load_game_options_sync_state, load_shared_game_options,
    shared_game_options_exist,
};

mod settings;
pub use self::settings::*;

mod onboarding_checklist;
pub use self::onboarding_checklist::*;

mod process;
pub use self::process::*;

mod java_globals;
pub use self::java_globals::*;

mod discord;
pub use self::discord::*;

mod minecraft_auth;
pub use self::minecraft_auth::*;

pub mod minecraft_skins;

mod cache;
pub use self::cache::*;

pub mod content_store;

mod friends;
pub use self::friends::*;

mod tunnel;
pub use self::tunnel::*;

pub mod db;
pub(crate) mod db_backup;
mod mr_auth;

pub use self::mr_auth::*;

mod legacy_converter;

pub mod attached_world_data;
pub mod server_join_log;

// Global state
// RwLock on state only has concurrent reads, except for config dir change which takes control of the State
static LAUNCHER_STATE: OnceCell<Arc<State>> = OnceCell::const_new();
static STATE_STARTUP_LOCK: Mutex<()> = Mutex::const_new(());
const MAX_CONCURRENT_INSTALL_JOBS: usize = 3;
pub struct State {
    startup_complete: AtomicBool,
    /// Information on the location of files used in the launcher
    pub directories: DirectoryInfo,
    pub content_store: content_store::ContentStore,

    /// Semaphore used to limit concurrent network requests and avoid errors
    pub fetch_semaphore: FetchSemaphore,
    /// Semaphore used to limit concurrent I/O and avoid errors
    pub io_semaphore: IoSemaphore,
    /// Semaphore to limit concurrent API requests. This is separate from the fetch semaphore
    /// to keep API functionality while the app is performing intensive tasks.
    pub api_semaphore: FetchSemaphore,
    pub(crate) install_job_semaphore: Semaphore,
    pub(crate) install_db_semaphore: Semaphore,
    /// Serializes filesystem reconciliation and content mutations per instance.
    instance_content_locks: DashMap<String, Arc<Mutex<()>>>,
    /// Serializes screenshot filesystem reconciliation per instance.
    instance_screenshot_locks: DashMap<String, Arc<Mutex<()>>>,
    /// Serializes shared instance attachment and recipient mutations per instance.
    shared_instance_locks: DashMap<String, Arc<Mutex<()>>>,
    /// Serializes canonical synced-option mutations and checkpoint updates.
    synced_options_lock: Mutex<()>,
    pub(crate) game_locale_indexer: crate::api::instance::GameLocaleIndexer,

    /// Discord RPC
    pub discord_rpc: DiscordGuard,

    /// Process manager
    pub process_manager: ProcessManager,

    // NOTE: we explicitly must NOT store the app identifier in the state object,
    // because creating the state object is fallible (e.g. database missing),
    // but we rely on the app identifier to create the state (data dir).
    //
    // /// App identifier string (like com.modrinth.ModrinthApp)
    // pub app_identifier: String,
    /// Friends socket
    pub friends_socket: FriendsSocket,

    pub restart_after_pending_update: AtomicBool,

    pub(crate) pool: SqlitePool,

    pub(crate) file_watcher: FileWatcher,
}

impl State {
    pub(crate) async fn lock_synced_options(&self) -> MutexGuard<'_, ()> {
        self.synced_options_lock.lock().await
    }

    pub(crate) async fn lock_instance_content(
        &self,
        instance_id: &str,
    ) -> OwnedMutexGuard<()> {
        let lock = self
            .instance_content_locks
            .entry(instance_id.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone();

        lock.lock_owned().await
    }

    pub(crate) async fn lock_shared_instance(
        &self,
        instance_id: &str,
    ) -> OwnedMutexGuard<()> {
        let lock = self
            .shared_instance_locks
            .entry(instance_id.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone();

        lock.lock_owned().await
    }

    pub(crate) async fn lock_instance_screenshots(
        &self,
        instance_id: &str,
    ) -> OwnedMutexGuard<()> {
        let lock = self
            .instance_screenshot_locks
            .entry(instance_id.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone();

        lock.lock_owned().await
    }

    pub(crate) fn remove_instance_locks(&self, instance_id: &str) {
        let _ = self.instance_content_locks.remove(instance_id);
        let _ = self.instance_screenshot_locks.remove(instance_id);
        let _ = self.shared_instance_locks.remove(instance_id);
    }

    pub async fn init(app_identifier: String) -> crate::Result<()> {
        let _startup = STATE_STARTUP_LOCK.lock().await;
        let state = LAUNCHER_STATE
            .get_or_try_init(move || Self::initialize_state(app_identifier))
            .await?;

        if state
            .startup_complete
            .load(std::sync::atomic::Ordering::Acquire)
        {
            return Ok(());
        }
        state.content_store.recover(None).await?;
        crate::install::recovery::recover_interrupted_jobs(state).await?;
        content_store::migration::migrate(state).await?;
        state
            .startup_complete
            .store(true, std::sync::atomic::Ordering::Release);
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(std::time::Duration::from_secs(600));
            interval.set_missed_tick_behavior(
                tokio::time::MissedTickBehavior::Skip,
            );
            loop {
                interval.tick().await;
                match instances::adapters::sqlite::instance_rows::list_instances(
					&state.pool,
				).await {
					Ok(instances) => {
						for instance in instances {
							if let Err(error) = instances::commands::migrate_legacy_content(
								&instance.id, state, true,
							).await {
								tracing::warn!(
									instance_id = %instance.id,
									"Legacy content migration deferred: {error}",
								);
							}
						}
					}
					Err(error) => tracing::warn!(
						"Could not list instances for content migration: {error}",
					),
				}
                if let Err(error) =
                    crate::api::instance::synced_packs::migrate_store(state)
                        .await
                {
                    tracing::warn!("Synced-pack migration deferred: {error}");
                }
                if let Err(error) = state.content_store.cleanup(false).await {
                    tracing::debug!(
                        "Shared content cache cleanup deferred: {error}"
                    );
                }
            }
        });

        tokio::task::spawn(async move {
            crate::api::instance::start_game_locale_indexer(Arc::clone(state));
            instances::watcher::watch_instances_init(
                &state.file_watcher,
                &state.directories,
                &state.pool,
            )
            .await;

            if let Err(error) =
                crate::api::instance::monitor_persisted_processes().await
            {
                tracing::error!(
                    "Failed to monitor persisted Minecraft processes: {error}"
                );
            }

            if let Err(error) =
                crate::api::instance::reconcile_all_synced_options().await
            {
                tracing::error!(
                    "Failed to reconcile instance synced options during startup: {error}"
                );
            }

            if let Err(e) = crate::api::instance::migrate_legacy_icons().await {
                tracing::error!("Error migrating legacy instance icons: {e}");
            }

            let res = tokio::try_join!(
                state.discord_rpc.clear_to_default(true),
                instances::refresh_all_instances(),
                Settings::migrate(&state.pool),
                ModrinthCredentials::refresh_all(),
            );

            if let Err(e) = res {
                tracing::error!("Error running discord RPC: {e}");
            }

            let _ = state
                .friends_socket
                .connect(
                    &state.pool,
                    &state.api_semaphore,
                    &state.process_manager,
                )
                .await;
            let _ = FriendsSocket::socket_loop().await;
        });

        Ok(())
    }

    /// Get the current launcher state, waiting for initialization.
    pub async fn get() -> crate::Result<Arc<Self>> {
        if !LAUNCHER_STATE.initialized() {
            tracing::error!(
                "Attempted to get state before it is initialized - this should never happen!"
            );
            while !LAUNCHER_STATE.initialized() {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }

        Ok(Arc::clone(
            LAUNCHER_STATE.get().expect("State is not initialized!"),
        ))
    }

    pub fn initialized() -> bool {
        LAUNCHER_STATE.initialized()
    }

    pub fn get_if_initialized() -> Option<Arc<Self>> {
        LAUNCHER_STATE.get().map(Arc::clone)
    }

    #[tracing::instrument]
    async fn initialize_state(
        app_identifier: String,
    ) -> crate::Result<Arc<Self>> {
        tracing::info!("Connecting to app database");
        let settings_dir =
            DirectoryInfo::initial_settings_dir_path(&app_identifier)
                .ok_or_else(|| {
                    crate::ErrorKind::FSError(
                        "Could not find the application directory".to_string(),
                    )
                })?;
        let store_lock =
            content_store::ContentStore::lock_process(&settings_dir).await?;
        let pool = db::connect(&app_identifier).await?;

        legacy_converter::migrate_legacy_data(&pool).await?;

        tracing::info!("Fetching app settings");
        let mut settings = Settings::get(&pool).await?;

        let fetch_semaphore =
            FetchSemaphore(Semaphore::new(settings.max_concurrent_downloads));
        let io_semaphore =
            IoSemaphore(Semaphore::new(settings.max_concurrent_writes));
        let api_semaphore =
            FetchSemaphore(Semaphore::new(settings.max_concurrent_downloads));

        tracing::info!("Initializing directories");
        DirectoryInfo::move_launcher_directory(
            &mut settings,
            &pool,
            &io_semaphore,
            &app_identifier,
        )
        .await?;

        let directories =
            DirectoryInfo::init(settings.custom_dir, &app_identifier).await?;
        let content_store = content_store::ContentStore::new(
            &directories,
            pool.clone(),
            store_lock,
        )
        .await?;

        let discord_rpc = DiscordGuard::init()?;

        tracing::info!("Initializing file watcher");
        let file_watcher = instances::watcher::init_watcher().await?;

        let process_manager = ProcessManager::new();

        let friends_socket = FriendsSocket::new();

        Ok(Arc::new(Self {
            startup_complete: AtomicBool::new(false),
            directories,
            content_store,
            fetch_semaphore,
            io_semaphore,
            api_semaphore,
            install_job_semaphore: Semaphore::new(MAX_CONCURRENT_INSTALL_JOBS),
            install_db_semaphore: Semaphore::new(1),
            instance_content_locks: DashMap::new(),
            instance_screenshot_locks: DashMap::new(),
            shared_instance_locks: DashMap::new(),
            synced_options_lock: Mutex::new(()),
            game_locale_indexer:
                crate::api::instance::GameLocaleIndexer::default(),
            discord_rpc,
            process_manager,
            friends_socket,
            restart_after_pending_update: AtomicBool::new(false),
            pool,
            file_watcher,
            // app_identifier,
        }))
    }
}
