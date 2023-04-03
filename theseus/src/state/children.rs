use std::{collections::HashMap, sync::Arc};
use tokio::process::Child;
use tokio::sync::RwLock;
use futures::stream::{self, StreamExt};

// Child processes (instances of Minecraft)
// A wrapper over a Hashmap connecting PID -> Child
// Left open for future functionality re: polling children
pub struct Children(HashMap<u32, Arc<RwLock<Child>>>);

impl Children {
    pub fn new() -> Children {
        Children(HashMap::new())
    }

    // Inserts and returns a ref to the child
    // Unlike a Hashmap, this directly returns the reference to the Child rather than any previously stored Child that may exist
    pub fn insert(
        &mut self,
        pid: u32,
        child: tokio::process::Child,
    ) -> Arc<RwLock<Child>> {
        let child = Arc::new(RwLock::new(child));
        self.0.insert(pid, child.clone());
        child
    }

    // Returns a ref to the child
    pub fn get(&self, pid: &u32) -> Option<Arc<RwLock<Child>>> {
        self.0.get(pid).cloned()
    }

    // Gets all PID keys
    pub fn keys(&self) -> Vec<u32> {
        self.0.keys().cloned().collect()
    }

    // Get exit status of a child by PID
    // Returns None if the child is still running
    pub async fn exit_status(&self, pid: &u32) -> crate::Result<Option<std::process::ExitStatus>> {
        if let Some(child) = self.get(pid) {
            let child = child.clone();
            let mut child = child.write().await;
            Ok(child.try_wait()?)
        } else {
            Ok(None)
        }
    }

    // Gets all PID keys of running children
    // If an error was collected in accessing the lock/child, that PID is discarded
    pub async fn running_keys(&self) -> Vec<u32> {
        stream::iter(self.0.iter())
            .filter(|(_, child)| {
                let child = child.clone();
                async move { child.write().await.try_wait().ok().is_none() }
            })
            .map(|(pid, _)| *pid)
            .collect()
            .await
    }

}

impl Default for Children {
    fn default() -> Self {
        Self::new()
    }
}
