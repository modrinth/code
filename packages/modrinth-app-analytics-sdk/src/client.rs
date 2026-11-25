use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use parking_lot::Mutex;
use reqwest::Url;
use smol_str::SmolStr;
use tokio::sync::{mpsc, oneshot};
use tracing::warn;
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
}

impl AppAnalytics {
    pub async fn new(
        AppAnalyticsParameters {
            push_url,
            opted_out,
            user_agent,
            installation_id,
            os,
            arch,
            app_version,
        }: AppAnalyticsParameters,
    ) -> AppAnalytics {
        assert_eq!(
            installation_id.get_version_num(),
            4,
            "installation_id must be a UUIDv4"
        );

        let (drain_tx, drain_rx) = mpsc::unbounded_channel();

        let this = AppAnalytics {
            inner: Arc::new(AnalyticsInner {
                backend: Mutex::new(if opted_out {
                    AnalyticsBackend::OptedOut
                } else {
                    AnalyticsBackend::Enabled {
                        http_pool: init_pool(&user_agent),
                    }
                }),
                user_agent,
                push_url,
                installation_id,
                queue: EventQueue::new(),
                drain_tx,
                os,
                arch,
                app_version,
            }),
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
        if self.inner.drain_tx.send(tx).is_err() {
            return false;
        }

        matches!(tokio::time::timeout(timeout, rx).await, Ok(Ok(true)))
    }

    pub fn set_telemetry_opt_out(&mut self, opted_out: bool) {
        if opted_out {
            // drops the http_pool and closes connections to the analytics server
            *self.inner.backend.lock() = AnalyticsBackend::OptedOut;
            self.inner.queue.clear();
        } else {
            *self.inner.backend.lock() = AnalyticsBackend::Enabled {
                http_pool: init_pool(&self.inner.user_agent),
            };
        }
    }
}

impl fmt::Debug for AppAnalytics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppAnalytics")
            .field("push_url", &self.inner.push_url)
            .field("backend", &self.inner.backend)
            .field("installation_id", &self.inner.installation_id)
            .finish()
    }
}

pub struct AppAnalyticsParameters {
    pub push_url: Url,
    pub opted_out: bool,
    pub user_agent: String,
    pub installation_id: Uuid,
    pub os: AppOs,
    pub arch: AppArch,
    pub app_version: SmolStr,
}

/// State shared between [`AppAnalytics`] handles and the background task.
pub(crate) struct AnalyticsInner {
    push_url: Url,
    backend: Mutex<AnalyticsBackend>,
    pub(crate) queue: EventQueue,
    /// Drain requests for the background task, answered with whether the
    /// queue was emptied.
    drain_tx: mpsc::UnboundedSender<oneshot::Sender<bool>>,
    user_agent: String,
    installation_id: Uuid,
    os: AppOs,
    arch: AppArch,
    app_version: SmolStr,
}

impl AnalyticsInner {
    /// Send a batch of events, if analytics haven't been opted-out of.
    ///
    /// `events` must be at most [`MAX_EVENTS_PER_BATCH`] events.
    pub(crate) async fn send_batch(
        &self,
        events: Vec<AppEvent>,
    ) -> Result<(), Vec<AppEvent>> {
        assert!(
            events.len() <= MAX_EVENTS_PER_BATCH,
            "events len must be at most {MAX_EVENTS_PER_BATCH}"
        );

        let Some(pool) = self.backend.lock().clone_pool() else {
            return Ok(());
        };

        let batch = AppEventBatch {
            sent_at: Utc::now(),
            installation_id: self.installation_id,
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
            Ok(_) => Ok(()),
            Err(e) => {
                warn!(
                    events.len = batch.events.len(),
                    error = format!("{e:#}"),
                    "Failed to send a batch of events for analytics"
                );
                Err(batch.events)
            }
        }
    }
}
