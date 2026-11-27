use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use parking_lot::Mutex;
use reqwest::Url;
use smol_str::SmolStr;
use tokio::sync::{mpsc, oneshot};
use tracing::{trace, warn};
use uuid::Uuid;

use crate::backend::{AnalyticsBackend, init_pool};
use crate::queue::EventQueue;
use crate::types::enums::{AppArch, AppOs};
use crate::types::http::batch::{AppEvent, AppEventBatch};
use crate::types::http::extract::{MAX_EVENTS_PER_BATCH, is_valid_source};
use crate::worker::bg;

#[derive(Clone)]
pub struct AppAnalytics {
    inner: Arc<AnalyticsInner>,
    /// Drain requests for the background task, answered with whether the
    /// queue was emptied.
    ///
    /// Only handles hold a sender, so the channel closes and the background
    /// task exits once the last handle is dropped.
    drain_tx: mpsc::UnboundedSender<oneshot::Sender<bool>>,
}

impl AppAnalytics {
    pub async fn new(
        AppAnalyticsParameters {
            push_url,
            installation_id,
            user_agent,
            os,
            arch,
            app_version,
        }: AppAnalyticsParameters,
    ) -> AppAnalytics {
        let backend = match installation_id {
            Some(installation_id) => {
                enabled_backend(&user_agent, installation_id)
            }
            None => AnalyticsBackend::OptedOut,
        };

        let (drain_tx, drain_rx) = mpsc::unbounded_channel();

        let this = AppAnalytics {
            inner: Arc::new(AnalyticsInner {
                backend: Mutex::new(backend),
                user_agent,
                push_url,
                queue: EventQueue::new(),
                os,
                arch,
                app_version,
            }),
            drain_tx,
        };

        tokio::spawn(bg(Arc::clone(&this.inner), drain_rx));

        this
    }

    /// Queues an event to be sent.
    ///
    /// Events can be dropped if they are not valid.
    pub fn add_event(&self, event: AppEvent) {
        if !is_valid_source(&event.source) {
            warn!(
                source = %event.source,
                event = event.event.name(),
                "Dropping analytics event with an invalid source"
            );
            return;
        }

        if !self.inner.backend.lock().is_enabled() {
            return;
        };

        self.inner.queue.push(event);
    }

    /// Sends every queued event now, waiting at most `timeout`.
    ///
    /// Call this before the app exits, while the Tokio runtime is still
    /// running. Each batch is tried once, without backoff. Returns whether
    /// the queue was emptied in time; on failure the unsent events stay
    /// queued.
    pub async fn drain(&self, timeout: Duration) -> bool {
        let (tx, rx) = oneshot::channel();
        if self.drain_tx.send(tx).is_err() {
            return false;
        }

        matches!(tokio::time::timeout(timeout, rx).await, Ok(Ok(true)))
    }

    /// Starts recording and sending events under `installation_id`, or
    /// switches to it if already opted in.
    ///
    /// Events still queued are dropped, so none recorded under a previous ID
    /// are sent under the new one.
    ///
    /// # Panics
    ///
    /// Panics if `installation_id` is not a UUIDv4.
    pub fn opt_in(&self, installation_id: Uuid) {
        let mut backend = self.inner.backend.lock();
        match &mut *backend {
            AnalyticsBackend::Enabled {
                installation_id: current,
                ..
            } => {
                assert_installation_id(installation_id);
                *current = installation_id;
            }
            AnalyticsBackend::OptedOut => {
                *backend =
                    enabled_backend(&self.inner.user_agent, installation_id);
            }
        }
        self.inner.queue.clear();
    }

    /// Stops recording events, drops the queued ones and forgets the
    /// installation ID.
    pub fn opt_out(&self) {
        // drops the http_pool and closes connections to the analytics server
        *self.inner.backend.lock() = AnalyticsBackend::OptedOut;
        self.inner.queue.clear();
    }
}

/// # Panics
///
/// Panics if `installation_id` is not a UUIDv4.
fn enabled_backend(
    user_agent: &str,
    installation_id: Uuid,
) -> AnalyticsBackend {
    assert_installation_id(installation_id);
    AnalyticsBackend::Enabled {
        http_pool: init_pool(user_agent),
        installation_id,
    }
}

fn assert_installation_id(installation_id: Uuid) {
    assert_eq!(
        installation_id.get_version_num(),
        4,
        "installation_id must be a UUIDv4"
    );
}

impl fmt::Debug for AppAnalytics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppAnalytics")
            .field("push_url", &self.inner.push_url)
            .field("backend", &self.inner.backend)
            .finish()
    }
}

pub struct AppAnalyticsParameters {
    pub push_url: Url,
    /// UUIDv4 identifying this installation, or `None` to start opted out.
    pub installation_id: Option<Uuid>,
    pub user_agent: String,
    pub os: AppOs,
    pub arch: AppArch,
    pub app_version: SmolStr,
}

/// State shared between [`AppAnalytics`] handles and the background task.
pub(crate) struct AnalyticsInner {
    push_url: Url,
    backend: Mutex<AnalyticsBackend>,
    pub(crate) queue: EventQueue,
    user_agent: String,
    os: AppOs,
    arch: AppArch,
    app_version: SmolStr,
}

impl AnalyticsInner {
    /// Send a batch of events, if analytics haven't been opted-out of.
    ///
    /// When the send fails, the events are put back in the queue to be
    /// retried, unless the installation ID changed or analytics were opted
    /// out of while sending.
    ///
    /// `events` must be at most [`MAX_EVENTS_PER_BATCH`] events.
    pub(crate) async fn send_batch(
        &self,
        events: Vec<AppEvent>,
    ) -> Result<(), ()> {
        assert!(
            events.len() <= MAX_EVENTS_PER_BATCH,
            "events len must be at most {MAX_EVENTS_PER_BATCH}"
        );

        let Some((pool, installation_id)) = self.backend.lock().sender() else {
            return Ok(());
        };

        let batch = AppEventBatch {
            sent_at: Utc::now(),
            installation_id,
            os: self.os,
            arch: self.arch,
            app_version: self.app_version.clone(), // should be cheap
            events,
        };

        // todo: compress
        let result = pool
            .post(self.push_url.clone())
            .json(&batch)
            .send()
            .await
            .and_then(|r| r.error_for_status());

        match result {
            // we don't care about the response
            Ok(_) => {
                trace!(
                    events.len = batch.events.len(),
                    "Sent a batch of events for analytics"
                );
                Ok(())
            }
            Err(e) => {
                warn!(
                    events.len = batch.events.len(),
                    error = format!("{e:#}"),
                    "Failed to send a batch of events for analytics"
                );
                let backend = self.backend.lock();
                if backend.installation_id() == Some(installation_id) {
                    self.queue.requeue(batch.events);
                }
                Err(())
            }
        }
    }
}
