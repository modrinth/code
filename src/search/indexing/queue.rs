use super::{add_projects, IndexingError, UploadSearchProject};
use crate::search::SearchConfig;
use std::sync::Mutex;

pub struct CreationQueue {
    // There's probably a better structure for this, but a mutex works
    // and I don't think this can deadlock.  This queue requires fast
    // writes and then a single potentially slower read/write that
    // empties the queue.
    queue: Mutex<Vec<UploadSearchProject>>,
}

impl CreationQueue {
    pub fn new() -> Self {
        CreationQueue {
            queue: Mutex::new(Vec::with_capacity(10)),
        }
    }
    pub fn add(&self, search_project: UploadSearchProject) {
        // Can only panic if mutex is poisoned
        self.queue.lock().unwrap().push(search_project);
    }
    pub fn take(&self) -> Vec<UploadSearchProject> {
        std::mem::replace(
            &mut *self.queue.lock().unwrap(),
            Vec::with_capacity(10),
        )
    }
    pub async fn index(
        &self,
        config: &SearchConfig,
    ) -> Result<(), IndexingError> {
        let queue = self.take();
        add_projects(queue, config).await
    }
}
