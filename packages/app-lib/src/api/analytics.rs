//! Theseus app analytics interface
use std::time::{Duration, Instant};

pub use modrinth_app_analytics_sdk::types::event::{
    AppEventType, MicrosoftSignInFailureReason, ModrinthSignInFailureReason,
};

use uuid::Uuid;

use crate::State;
use crate::state::Settings;

/// Queues an analytics event, if the app state and analytics are initialized.
///
/// `source` must be a PascalCase identifier of where in the app the action
/// was initiated from.
pub fn add_event(source: &'static str, event: AppEventType) {
    if let Some(state) = State::get_if_initialized()
        && let Some(analytics) = &state.analytics
    {
        analytics
            .add_event(modrinth_app_analytics_sdk::create_event(source, event));
    }
}

/// Replaces this installation's analytics ID with a new random one and
/// returns it.
///
/// Events queued under the old ID are dropped. Fails when telemetry is
/// disabled, since there is no ID then.
pub async fn reset_installation_id() -> crate::Result<Uuid> {
    let state = State::get().await?;
    let _guard = state.synced_options.lock().await;

    if !Settings::get(&state.pool).await?.telemetry {
        return Err(crate::ErrorKind::InputError(
            "Telemetry is disabled, so there is no installation ID to reset"
                .to_string(),
        )
        .into());
    }

    let id =
        crate::state::app_analytics::reset_installation_id(&state.pool).await?;

    if let Some(analytics) = &state.analytics {
        analytics.opt_in(id);
    }

    Ok(id)
}

/// Milliseconds since `start`, for an event's `duration_ms`.
#[must_use]
pub fn elapsed_ms(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX)
}

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
