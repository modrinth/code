mod local;

use std::time::Duration;

pub(super) use self::local::{LockAcquisition, LockCoordinator, LockWaiter};

pub(super) const WAIT_TIMEOUT: Duration = Duration::from_secs(5);

/// Normalize only the requested lookup form's case. Raw IDs and aliases remain
/// distinct lock identities and may therefore fill concurrently.
pub(super) fn normalize_key(key: &str, case_sensitive: bool) -> String {
    if case_sensitive {
        key.to_owned()
    } else {
        key.to_lowercase()
    }
}
