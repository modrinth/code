use std::time::Duration;

use crate::env::ENV;
use chrono::{DateTime, Utc};
use eyre::WrapErr;
use rdkafka::{ClientConfig, consumer::StreamConsumer, producer::FutureProducer};
use serde::Serialize;
use uuid::Uuid;

pub const KAFKA_OPERATION_INTERVAL: Duration = Duration::from_secs(5);
pub const INCREMENTAL_INDEX_SEARCH_TASK: &str = "incremental-index-search";

pub struct KafkaClientState {
    pub client: FutureProducer,
    pub incremental_index_search_consumer: StreamConsumer,
}

impl KafkaClientState {
	pub fn new() -> eyre::Result<Self> {
		let client = ClientConfig::new()
			.set("bootstrap.servers", ENV.KAFKA_BOOTSTRAP_SERVERS.0.join(","))
			.set("client.id", &ENV.KAFKA_CLIENT_ID)
			.set("broker.address.family", "v4")
			.create()
			.wrap_err("failed to create Kafka producer")?;
		let incremental_index_search_consumer =
			create_consumer(INCREMENTAL_INDEX_SEARCH_TASK)
				.wrap_err("failed to create incremental search Kafka consumer")?;

        tracing::info!(
            kafka.bootstrap_servers = ?ENV.KAFKA_BOOTSTRAP_SERVERS.0,
            kafka.client_id = %ENV.KAFKA_CLIENT_ID,
            "Connected to Kafka"
        );

        Ok(Self {
            client,
            incremental_index_search_consumer,
        })
	}
}

pub fn create_consumer(group_id: &str) -> eyre::Result<StreamConsumer> {
	ClientConfig::new()
		.set("bootstrap.servers", ENV.KAFKA_BOOTSTRAP_SERVERS.0.join(","))
		.set("client.id", &ENV.KAFKA_CLIENT_ID)
		.set("group.id", group_id)
		.set("enable.auto.commit", "false")
		.set("auto.offset.reset", "earliest")
		.set("broker.address.family", "v4")
		.create()
		.wrap_err("failed to create Kafka consumer")
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
