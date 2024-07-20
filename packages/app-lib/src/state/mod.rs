//! Theseus state management system
use crate::event::emit::{emit_loading, init_loading_unsafe};

use crate::event::LoadingBarType;
use crate::loading_join;

use crate::util::fetch::{FetchSemaphore, IoSemaphore};
use std::sync::Arc;
use tokio::sync::{OnceCell, RwLock, Semaphore};

use crate::state::fs_watcher::FileWatcher;
use sqlx::SqlitePool;

// Submodules
mod dirs;
pub use self::dirs::*;

mod profiles;
pub use self::profiles::*;

mod settings;
pub use self::settings::*;

mod children;
pub use self::children::*;

mod java_globals;
pub use self::java_globals::*;

mod discord;
pub use self::discord::*;

mod minecraft_auth;
pub use self::minecraft_auth::*;

mod cache;
pub use self::cache::*;

mod db;
pub mod fs_watcher;
mod mr_auth;

pub use self::mr_auth::*;

// TODO: Cache home page queries?
// TODO: UI: Profile Options
// TODO: UI: Settings Java Versions
// TODO: pass credentials to modrinth cdn

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

    /// Reference to minecraft process children
    pub children: RwLock<Children>,
    /// Launcher user account info
    pub(crate) users: RwLock<MinecraftAuthStore>,
    /// Modrinth Credentials Store
    pub credentials: RwLock<CredentialsStore>,
    /// Modrinth auth flow
    pub modrinth_auth_flow: RwLock<Option<ModrinthAuthFlow>>,

    /// Discord RPC
    pub discord_rpc: DiscordGuard,

    pub(crate) pool: SqlitePool,

    pub(crate) file_watcher: FileWatcher,
}

impl State {
    pub async fn init() -> crate::Result<()> {
        LAUNCHER_STATE
            .get_or_try_init(Self::initialize_state)
            .await?;

        Ok(())
    }

    /// Get the current launcher state, waiting for initialization
    pub async fn get() -> crate::Result<Arc<Self>> {
        if !LAUNCHER_STATE.initialized() {
            while !LAUNCHER_STATE.initialized() {}
        }

        Ok(Arc::clone(
            LAUNCHER_STATE.get().expect("State is not initialized!"),
        ))
    }

    pub fn initialized() -> bool {
        LAUNCHER_STATE.initialized()
    }

    #[tracing::instrument]
    #[theseus_macros::debug_pin]
    async fn initialize_state() -> crate::Result<Arc<Self>> {
        let loading_bar = init_loading_unsafe(
            LoadingBarType::StateInit,
            100.0,
            "Initializing launcher",
        )
        .await?;

        let directories = DirectoryInfo::init()?;

        let pool = db::connect().await?;

        let settings = Settings::get(&pool).await?;

        emit_loading(&loading_bar, 10.0, None).await?;

        let fetch_semaphore =
            FetchSemaphore(Semaphore::new(settings.max_concurrent_downloads));
        let io_semaphore =
            IoSemaphore(Semaphore::new(settings.max_concurrent_writes));
        emit_loading(&loading_bar, 10.0, None).await?;

        let users_fut = MinecraftAuthStore::init(&directories, &io_semaphore);
        let creds_fut = CredentialsStore::init(&directories, &io_semaphore);
        // Launcher data
        let (users, creds) = loading_join! {
            Some(&loading_bar), 70.0, Some("Loading metadata");
            users_fut,
            creds_fut,
        }?;

        let discord_rpc = DiscordGuard::init().await?;
        if settings.discord_rpc {
            // Add default Idling to discord rich presence
            // Force add to avoid recursion
            let _ = discord_rpc.force_set_activity("Idling...", true).await;
        }

        let children = Children::new();

        let file_watcher = fs_watcher::init_watcher().await?;
        fs_watcher::watch_profiles_init(&file_watcher, &directories).await?;

        emit_loading(&loading_bar, 10.0, None).await?;

        Ok(Arc::new(Self {
            directories,
            fetch_semaphore,
            io_semaphore,
            users: RwLock::new(users),
            children: RwLock::new(children),
            credentials: RwLock::new(creds),
            discord_rpc,
            modrinth_auth_flow: RwLock::new(None),
            pool,
            file_watcher,
        }))
    }

    /// Updates state with data from the web, if we are online
    pub fn update() {
        tokio::task::spawn(async {
            let res6 = CredentialsStore::update_creds();

            let _ = res6.await;
        });
    }
}
