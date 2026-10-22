use tokio::sync::{Mutex, Semaphore};

const MAX_CONCURRENT_INSTALL_JOBS: usize = 3;

/// Limits install job concurrency.
pub(crate) struct Installs {
    /// Admission covers setup and deletion. A target reservation stays with its worker
    /// until cleanup finishes, so backups and rollback cannot overlap another install.
    pub(crate) admission: Mutex<()>,
    pub(crate) job_semaphore: Semaphore,
    pub(crate) db_semaphore: Semaphore,
}

impl Installs {
    pub(crate) fn new() -> Self {
        Self {
            admission: Mutex::new(()),
            job_semaphore: Semaphore::new(MAX_CONCURRENT_INSTALL_JOBS),
            db_semaphore: Semaphore::new(1),
        }
    }
}
