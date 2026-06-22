use std::time::Duration;

use crate::env::ENV;
use chrono::{DateTime, Utc};
use rdkafka::{ClientConfig, producer::FutureProducer};
use serde::Serialize;
use uuid::Uuid;

pub const KAFKA_OPERATION_INTERVAL: Duration = Duration::from_secs(5);

pub struct KafkaClientState {
    pub client: FutureProducer,
}

impl KafkaClientState {
    pub fn new() -> eyre::Result<Self> {
        let client = ClientConfig::new()
            .set("bootstrap.servers", ENV.KAFKA_BOOTSTRAP_SERVERS.0.join(","))
            .set("client.id", &ENV.KAFKA_CLIENT_ID)
            .set("broker.address.family", "v4")
            .create()?;

        tracing::info!(
            kafka.bootstrap_servers = ?ENV.KAFKA_BOOTSTRAP_SERVERS.0,
            kafka.client_id = %ENV.KAFKA_CLIENT_ID,
            "Connected to Kafka"
        );

        Ok(Self { client })
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
