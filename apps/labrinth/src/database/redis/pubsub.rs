use std::time::Duration;

use futures::StreamExt;
use redis::ToRedisArgs;
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::database::models::DatabaseError;

use super::RedisPool;

const PUBSUB_BUFFER_SIZE: usize = 1024;
const INITIAL_RECONNECT_BACKOFF: Duration = Duration::from_millis(250);
const MAX_RECONNECT_BACKOFF: Duration = Duration::from_secs(30);

enum SubscriptionOutcome {
    SubscriberClosed,
    Disconnected,
}

impl RedisPool {
    pub(crate) fn subscribe(
        &self,
        channel: &'static str,
    ) -> mpsc::Receiver<Vec<u8>> {
        let seed_urls = self.config.seed_urls().to_vec();
        let (sender, receiver) = mpsc::channel(PUBSUB_BUFFER_SIZE);
        tokio::spawn(run_subscription(seed_urls, channel, sender));
        receiver
    }

    pub async fn publish<M>(
        &self,
        channel: &str,
        message: M,
    ) -> Result<(), DatabaseError>
    where
        M: ToRedisArgs + Send + Sync,
    {
        let mut connection = self.connect().await?;
        let _: usize = redis::cmd("PUBLISH")
            .arg(channel)
            .arg(message)
            .query_async(&mut connection)
            .await?;
        Ok(())
    }
}

async fn run_subscription(
    seed_urls: Vec<String>,
    channel: &'static str,
    sender: mpsc::Sender<Vec<u8>>,
) {
    let mut next_seed = 0;
    let mut reconnect_backoff = INITIAL_RECONNECT_BACKOFF;

    loop {
        let mut connected = false;

        for _ in 0..seed_urls.len() {
            let seed_url = &seed_urls[next_seed];
            next_seed = (next_seed + 1) % seed_urls.len();

            match forward_from_seed(seed_url, channel, &sender).await {
                Ok(SubscriptionOutcome::SubscriberClosed) => return,
                Ok(SubscriptionOutcome::Disconnected) => {
                    warn!(channel, "Redis Pub/Sub connection disconnected");
                    connected = true;
                    break;
                }
                Err(error) => {
                    warn!(
                        ?error,
                        channel,
                        "Failed to establish Redis Pub/Sub subscription"
                    );
                }
            }
        }

        if sender.is_closed() {
            return;
        }

        let delay = if connected {
            INITIAL_RECONNECT_BACKOFF
        } else {
            reconnect_backoff
        };
        tokio::time::sleep(delay).await;

        reconnect_backoff = if connected {
            INITIAL_RECONNECT_BACKOFF
        } else {
            reconnect_backoff
                .saturating_mul(2)
                .min(MAX_RECONNECT_BACKOFF)
        };
    }
}

async fn forward_from_seed(
    seed_url: &str,
    channel: &'static str,
    sender: &mpsc::Sender<Vec<u8>>,
) -> redis::RedisResult<SubscriptionOutcome> {
    let client = redis::Client::open(seed_url)?;
    let mut pubsub = client.get_async_pubsub().await?;
    pubsub.subscribe(channel).await?;
    info!(channel, "Established Redis Pub/Sub subscription");

    let mut stream = pubsub.into_on_message();
    while let Some(message) = stream.next().await {
        if message.get_channel_name() != channel {
            continue;
        }

        if sender
            .send(message.get_payload_bytes().to_vec())
            .await
            .is_err()
        {
            return Ok(SubscriptionOutcome::SubscriberClosed);
        }
    }

    Ok(SubscriptionOutcome::Disconnected)
}
