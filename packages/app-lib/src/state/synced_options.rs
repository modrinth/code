use crate::api::instance::{GameLocaleIndexer, PackSyncWorker};
use tokio::sync::{Mutex, MutexGuard};

/// Coordinates options syncing across instances.
#[derive(Default)]
pub(crate) struct SyncedOptions {
    /// Serializes canonical synced-option mutations and checkpoint updates.
    lock: Mutex<()>,
    pub(crate) locales: GameLocaleIndexer,
    pub(crate) packs: PackSyncWorker,
}

impl SyncedOptions {
    pub(crate) async fn lock(&self) -> MutexGuard<'_, ()> {
        self.lock.lock().await
    }
}
