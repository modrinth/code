//! Records Modrinth App analytics events and sends them to the ingest
//! endpoint.
//!
//! Events are queued in memory and sent in batches by a background task,
//! every [`FLUSH_INTERVAL`] or as soon as a full batch is queued. Failed
//! sends are retried with backoff, and the oldest events are dropped once
//! [`MAX_QUEUED_EVENTS`] are queued. Call [`AppAnalytics::drain`] before the
//! app exits to send what is still queued.

mod backend;
mod client;
mod queue;
mod worker;

use std::time::Duration;

use smol_str::SmolStr;

pub use modrinth_app_analytics_types as types;
use types::http::batch::AppEvent;
use types::http::extract::MAX_EVENTS_PER_BATCH;
use uuid::ContextV7;

pub use crate::client::{AppAnalytics, AppAnalyticsParameters};

/// How often queued events are sent when no full batch is waiting.
pub const FLUSH_INTERVAL: Duration = Duration::from_secs(10);

/// Most events kept in memory while sends are failing.
pub const MAX_QUEUED_EVENTS: usize = 32 * MAX_EVENTS_PER_BATCH;

/// Creates an event, timestamped now.
///
/// `source` is where in the app the action was initiated from, such as
/// `browse`.
pub fn create_event(
    source: impl Into<SmolStr>,
    event: types::event::AppEventType,
) -> AppEvent {
    thread_local!(static CONTEXT_V7: ContextV7 = const { ContextV7::new() });
    let source = source.into();
    CONTEXT_V7.with(move |c| AppEvent::new(c, source, event))
}
