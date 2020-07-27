use super::{add_mods, IndexingError, UploadSearchMod};
use std::sync::Mutex;

pub struct CreationQueue {
    // There's probably a better structure for this, but a mutex works
    // and I don't think this can deadlock.  This queue requires fast
    // writes and then a single potentially slower read/write that
    // empties the queue.
    queue: Mutex<Vec<UploadSearchMod>>,
}

impl CreationQueue {
    pub fn new() -> Self {
        CreationQueue {
            queue: Mutex::new(Vec::with_capacity(10)),
        }
    }

    pub fn add(&self, search_mod: UploadSearchMod) {
        // Can only panic if mutex is poisoned
        self.queue.lock().unwrap().push(search_mod);
    }
    pub fn take(&self) -> Vec<UploadSearchMod> {
        std::mem::replace(&mut *self.queue.lock().unwrap(), Vec::with_capacity(10))
    }
}

pub async fn index_queue(queue: &CreationQueue) -> Result<(), IndexingError> {
    let queue = queue.take();
    add_mods(queue).await
}
