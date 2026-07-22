use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, OwnedMutexGuard};

#[derive(Default)]
pub(crate) struct InstanceContentLocks {
    locks: DashMap<String, Arc<Mutex<()>>>,
}

impl InstanceContentLocks {
    pub(crate) async fn lock(
        &self,
        instance_id: &str,
    ) -> InstanceContentGuard<'_> {
        let lock = self
            .locks
            .entry(instance_id.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone();
        let guard = lock.clone().lock_owned().await;

        InstanceContentGuard {
            guard: Some(guard),
            instance_id: instance_id.to_string(),
            lock,
            locks: self,
        }
    }
}

pub(crate) struct InstanceContentGuard<'a> {
    guard: Option<OwnedMutexGuard<()>>,
    instance_id: String,
    lock: Arc<Mutex<()>>,
    locks: &'a InstanceContentLocks,
}

impl Drop for InstanceContentGuard<'_> {
    fn drop(&mut self) {
        drop(self.guard.take());
        let _ = self.locks.locks.remove_if(&self.instance_id, |_, lock| {
            Arc::ptr_eq(lock, &self.lock) && Arc::strong_count(lock) == 2
        });
    }
}
