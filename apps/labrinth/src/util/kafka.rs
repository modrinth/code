use std::{ops::Deref, time::Duration};

use crate::{env::ENV, util::error::Context};
use chrono::{DateTime, Utc};
use kafka::client::KafkaClient;
use serde::Serialize;
use tokio::sync::Mutex;
use uuid::Uuid;

pub const KAFKA_OPERATION_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub struct KafkaClientState {
    client: Mutex<KafkaClient>,
}

impl KafkaClientState {
    pub fn new() -> eyre::Result<Self> {
        let mut client =
            KafkaClient::new(ENV.KAFKA_BOOTSTRAP_SERVERS.0.clone());
        client.set_client_id(ENV.KAFKA_CLIENT_ID.clone());
        client
            .load_metadata(&[
                crate::search::incremental::SEARCH_PROJECT_INDEX_QUEUE_TOPIC,
            ])
            .wrap_err("failed to load Kafka metadata")?;

        let topic_names = client
            .topics()
            .names()
            .map(str::to_string)
            .collect::<Vec<_>>();

        tracing::info!(
            kafka.bootstrap_servers = ?client.hosts(),
            kafka.client_id = %client.client_id(),
            kafka.topic_count = topic_names.len(),
            kafka.topics = ?topic_names,
            "Connected to Kafka"
        );

        Ok(Self {
            client: Mutex::new(client),
        })
    }
}

impl Deref for KafkaClientState {
    type Target = Mutex<KafkaClient>;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

#[derive(Debug, Serialize)]
pub struct KafkaEvent<T> {
    pub event_type: &'static str,
    pub event_metadata: EventMetadata,
    #[serde(flatten)]
    pub data: T,
}

impl<T> KafkaEvent<T> {
    pub fn new(event_type: &'static str, data: T) -> Self {
        Self {
            event_type,
            event_metadata: EventMetadata::new(),
            data,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct EventMetadata {
    pub event_id: Uuid,
    pub event_time: DateTime<Utc>,
    pub service: &'static ServiceMetadata,
}

impl EventMetadata {
    pub fn new() -> Self {
        Self {
            event_id: Uuid::now_v7(),
            event_time: Utc::now(),
            service: &SERVICE_METADATA,
        }
    }
}

impl Default for EventMetadata {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize)]
pub struct ServiceMetadata {
    pub service_name: &'static str,
    pub service_version: &'static str,
}

pub static SERVICE_METADATA: ServiceMetadata = ServiceMetadata {
    service_name: "labrinth",
    service_version: env!("CARGO_PKG_VERSION"),
};
