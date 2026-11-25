use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{mpsc, oneshot};
use tokio::time::{MissedTickBehavior, interval, sleep};

use crate::FLUSH_INTERVAL;
use crate::client::AnalyticsInner;

pub(crate) type DrainReceiver = mpsc::UnboundedReceiver<oneshot::Sender<bool>>;

/// Background task that sends queued events and answers drain requests.
///
/// Returns once every [`AppAnalytics`](crate::AppAnalytics) handle is
/// dropped, discarding any events still queued.
pub(crate) async fn bg(a: Arc<AnalyticsInner>, mut drain_rx: DrainReceiver) {
    const BACKOFF_DEFAULT: Duration = Duration::from_secs(5);
    const BACKOFF_MAX: Duration = Duration::from_mins(5);
    let mut backoff = BACKOFF_DEFAULT;

    let mut flush = interval(FLUSH_INTERVAL);
    flush.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            _ = flush.tick() => {}
            () = a.queue.full_batch.notified() => {}
            reply = drain_rx.recv() => {
                let Some(reply) = reply else { return };
                drain(&a, reply, &mut drain_rx).await;
                continue;
            }
        }

        loop {
            let batch = a.queue.take_batch();
            if batch.is_empty() {
                break;
            }

            match a.send_batch(batch).await {
                Ok(()) => backoff = BACKOFF_DEFAULT,
                Err(events) => {
                    a.queue.requeue(events);

                    tokio::select! {
                        () = sleep(backoff) => {}
                        reply = drain_rx.recv() => {
                            let Some(reply) = reply else { return };
                            drain(&a, reply, &mut drain_rx).await;
                        }
                    }
                    backoff = (backoff * 3).min(BACKOFF_MAX); // 5s, 15s, 45s, ~2 mins, 5 mins
                }
            }
        }
    }
}

/// Sends queued batches until the queue is empty or a send fails, then
/// answers `reply` and any other waiting drain requests.
async fn drain(
    a: &AnalyticsInner,
    reply: oneshot::Sender<bool>,
    drain_rx: &mut DrainReceiver,
) {
    let emptied = loop {
        let batch = a.queue.take_batch();
        if batch.is_empty() {
            break true;
        }

        if let Err(events) = a.send_batch(batch).await {
            a.queue.requeue(events);
            break false;
        }
    };

    let _ = reply.send(emptied);
    while let Ok(reply) = drain_rx.try_recv() {
        let _ = reply.send(emptied);
    }
}
