use std::{collections::HashMap, sync::Arc};
use tokio::process::Child;
use tokio::sync::RwLock;

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
}

impl Default for Children {
    fn default() -> Self {
        Self::new()
    }
}
