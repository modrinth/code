//! Theseus app analytics interface
use std::time::Duration;

use crate::State;

/// Sends queued analytics events, waiting at most `timeout`.
///
/// Call this before the app exits. Returns whether the queue was emptied;
/// returns `true` when the app state or analytics aren't initialized.
pub async fn drain_events(timeout: Duration) -> bool {
    let Some(analytics) =
        State::get_if_initialized().and_then(|state| state.analytics.clone())
    else {
        return true;
    };

    analytics.drain(timeout).await
}
