use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, OwnedMutexGuard};

/// Serializes concurrent changes per instance.
#[derive(Default)]
pub(crate) struct InstanceLocks {
    /// Serializes launches and changes to launch settings per instance.
    launch: DashMap<String, Arc<Mutex<()>>>,
    /// Serializes filesystem reconciliation and content mutations per instance.
    content: DashMap<String, Arc<Mutex<()>>>,
    screenshots: DashMap<String, Arc<Mutex<()>>>,
    /// Serializes shared instance attachment and recipient mutations per instance.
    shared: DashMap<String, Arc<Mutex<()>>>,
}

impl InstanceLocks {
    pub(crate) async fn lock_launch(
        &self,
        instance_id: &str,
    ) -> OwnedMutexGuard<()> {
        lock(&self.launch, instance_id).await
    }

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
        let _ = self.launch.remove(instance_id);
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

#[cfg(test)]
mod tests {
    use super::InstanceLocks;
    use futures::poll;
    use std::time::Duration;

    #[tokio::test]
    async fn launch_lock_serializes_same_instance_without_blocking_content() {
        let locks = InstanceLocks::default();
        let first = locks.lock_launch("local:dev").await;
        let mut second = Box::pin(locks.lock_launch("local:dev"));
        assert!(poll!(&mut second).is_pending());
        let _other = tokio::time::timeout(
            Duration::from_secs(1),
            locks.lock_launch("local:other"),
        )
        .await
        .expect("other instances must remain independent");
        let _content = tokio::time::timeout(
            Duration::from_secs(1),
            locks.lock_content("local:dev"),
        )
        .await
        .expect("launch preparation must be able to acquire the content lock");
        drop(first);
        let _second = tokio::time::timeout(Duration::from_secs(1), second)
            .await
            .expect("the next launch must proceed after the first finishes");
    }
}
