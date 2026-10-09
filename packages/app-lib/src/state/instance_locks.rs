use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, OwnedMutexGuard};

/// Serializes concurrent changes per instance.
#[derive(Default)]
pub(crate) struct InstanceLocks {
    /// Serializes filesystem reconciliation and content mutations per instance.
    content: DashMap<String, Arc<Mutex<()>>>,
    screenshots: DashMap<String, Arc<Mutex<()>>>,
    /// Serializes shared instance attachment and recipient mutations per instance.
    shared: DashMap<String, Arc<Mutex<()>>>,
}

impl InstanceLocks {
    pub(crate) async fn lock_content(
        &self,
        instance_id: &str,
    ) -> OwnedMutexGuard<()> {
        lock(&self.content, instance_id).await
    }

    pub(crate) async fn lock_screenshots(
        &self,
        instance_id: &str,
    ) -> OwnedMutexGuard<()> {
        lock(&self.screenshots, instance_id).await
    }

    pub(crate) async fn lock_shared(
        &self,
        instance_id: &str,
    ) -> OwnedMutexGuard<()> {
        lock(&self.shared, instance_id).await
    }

    pub(crate) fn remove(&self, instance_id: &str) {
        let _ = self.content.remove(instance_id);
        let _ = self.screenshots.remove(instance_id);
        let _ = self.shared.remove(instance_id);
    }
}

async fn lock(
    locks: &DashMap<String, Arc<Mutex<()>>>,
    instance_id: &str,
) -> OwnedMutexGuard<()> {
    let lock = locks
        .entry(instance_id.to_string())
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone();

    lock.lock_owned().await
}
