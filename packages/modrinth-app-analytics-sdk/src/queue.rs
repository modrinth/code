use std::collections::VecDeque;

use parking_lot::Mutex;
use tokio::sync::Notify;
use tracing::warn;

use crate::MAX_QUEUED_EVENTS;
use crate::types::http::batch::AppEvent;
use crate::types::http::extract::MAX_EVENTS_PER_BATCH;

/// Events waiting to be sent, oldest first.
pub(crate) struct EventQueue {
    events: Mutex<VecDeque<AppEvent>>,
    /// Notified when a full batch is queued.
    pub(crate) full_batch: Notify,
}

impl EventQueue {
    pub(crate) fn new() -> Self {
        Self {
            events: Mutex::new(VecDeque::with_capacity(MAX_EVENTS_PER_BATCH)),
            full_batch: Notify::new(),
        }
    }

    pub(crate) fn clear(&self) {
        let mut events = self.events.lock();
        events.clear();
    }

    pub(crate) fn push(&self, event: AppEvent) {
        let mut events = self.events.lock();
        if events.len() >= MAX_QUEUED_EVENTS {
            events.pop_front();
        }
        events.push_back(event);

        if events.len() >= MAX_EVENTS_PER_BATCH {
            drop(events);
            self.full_batch.notify_one();
        }
    }

    /// Takes up to [`MAX_EVENTS_PER_BATCH`] of the oldest events.
    pub(crate) fn take_batch(&self) -> Vec<AppEvent> {
        let mut events = self.events.lock();
        let len = events.len().min(MAX_EVENTS_PER_BATCH);
        events.drain(..len).collect()
    }

    /// Puts a batch that failed to send back at the front of the queue,
    /// dropping the oldest events if the queue is full.
    pub(crate) fn requeue(&self, batch: Vec<AppEvent>) {
        let mut events = self.events.lock();
        for event in batch.into_iter().rev() {
            events.push_front(event);
        }

        let excess = events.len().saturating_sub(MAX_QUEUED_EVENTS);
        if excess > 0 {
            events.drain(..excess);
            warn!(
                "Dropping {excess} events from the analytics queue because sends are failing"
            );
        }
    }
}
