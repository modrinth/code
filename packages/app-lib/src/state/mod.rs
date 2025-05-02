//! Theseus state management system
use crate::util::fetch::{FetchSemaphore, IoSemaphore};
use std::sync::Arc;
use tokio::sync::{OnceCell, Semaphore};

use crate::state::fs_watcher::FileWatcher;
use sqlx::SqlitePool;

// Submodules
mod dirs;
pub use self::dirs::*;

mod profiles;
pub use self::profiles::*;

mod settings;
pub use self::settings::*;

mod process;
pub use self::process::*;

mod java_globals;
pub use self::java_globals::*;

mod discord;
pub use self::discord::*;

mod minecraft_auth;
pub use self::minecraft_auth::*;

mod cache;
pub use self::cache::*;

mod friends;
pub use self::friends::*;

mod tunnel;
pub use self::tunnel::*;

pub mod db;
pub mod fs_watcher;
mod mr_auth;

pub use self::mr_auth::*;

mod legacy_converter;

pub mod attached_world_data;
pub mod server_join_log;

// Global state
// RwLock on state only has concurrent reads, except for config dir change which takes control of the State
static LAUNCHER_STATE: OnceCell<Arc<State>> = OnceCell::const_new();
pub struct State {
    /// Information on the location of files used in the launcher
    pub directories: DirectoryInfo,

    /// Semaphore used to limit concurrent network requests and avoid errors
    pub fetch_semaphore: FetchSemaphore,
    /// Semaphore used to limit concurrent I/O and avoid errors
    pub io_semaphore: IoSemaphore,
    /// Semaphore to limit concurrent API requests. This is separate from the fetch semaphore
    /// to keep API functionality while the app is performing intensive tasks.
    pub api_semaphore: FetchSemaphore,

    /// Discord RPC
    pub discord_rpc: DiscordGuard,

    /// Process manager
    pub process_manager: ProcessManager,

    /// Friends socket
    pub friends_socket: FriendsSocket,

    pub(crate) pool: SqlitePool,

    pub(crate) file_watcher: FileWatcher,
}

impl State {
    pub async fn init() -> crate::Result<()> {
        let state = LAUNCHER_STATE
            .get_or_try_init(Self::initialize_state)
            .await?;

        tokio::task::spawn(async move {
            let res = tokio::try_join!(
                state.discord_rpc.clear_to_default(true),
                Profile::refresh_all(),
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

    /// Get the current launcher state, waiting for initialization
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

    #[tracing::instrument]
    async fn initialize_state() -> crate::Result<Arc<Self>> {
        tracing::info!("Connecting to app database");
        let pool = db::connect().await?;

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
        )
        .await?;
        let directories = DirectoryInfo::init(settings.custom_dir).await?;

        let discord_rpc = DiscordGuard::init()?;

        tracing::info!("Initializing file watcher");
        let file_watcher = fs_watcher::init_watcher().await?;
        fs_watcher::watch_profiles_init(&file_watcher, &directories).await;

        let process_manager = ProcessManager::new();

        let friends_socket = FriendsSocket::new();

        Ok(Arc::new(Self {
            directories,
            fetch_semaphore,
            io_semaphore,
            api_semaphore,
            discord_rpc,
            process_manager,
            friends_socket,
            pool,
            file_watcher,
        }))
    }
}
