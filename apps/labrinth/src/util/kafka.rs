use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use crate::env::ENV;
use chrono::{DateTime, Utc};
use eyre::{WrapErr, ensure, eyre};
use rdkafka::{
    ClientConfig, Offset, TopicPartitionList,
    consumer::{Consumer, StreamConsumer},
    message::{Header, OwnedHeaders},
    producer::{FutureProducer, Producer},
};
use serde::Serialize;
use tokio::sync::Mutex;
use uuid::Uuid;

pub const KAFKA_OPERATION_INTERVAL: Duration = Duration::from_secs(5);
pub const KAFKA_DELIVERY_TIMEOUT: Duration = Duration::from_secs(30);
pub const INCREMENTAL_INDEX_SEARCH_TASK: &str = "incremental-index-search";

pub struct KafkaClientState {
    pub client: FutureProducer,
    pub project_flag_run_producer: Arc<Mutex<Option<FutureProducer>>>,
    pub incremental_index_search_consumer: StreamConsumer,
    project_flag_lag_consumer: Arc<StreamConsumer>,
}

impl KafkaClientState {
    pub async fn new() -> eyre::Result<Self> {
        let client = producer_config()
            .create()
            .wrap_err("failed to create Kafka producer")?;
        let incremental_index_search_consumer = create_consumer(
            INCREMENTAL_INDEX_SEARCH_TASK,
        )
        .wrap_err("failed to create incremental search Kafka consumer")?;

        tracing::info!(
            kafka.bootstrap_servers = ?ENV.KAFKA_BOOTSTRAP_SERVERS.0,
            kafka.client_id = %ENV.KAFKA_CLIENT_ID,
            "Connected to Kafka"
        );

        let project_flag_run_producer =
            tokio::task::spawn_blocking(create_project_flag_run_producer)
                .await
                .wrap_err(
                    "joining project flag run producer initialization",
                )??;

        let project_flag_lag_consumer = create_consumer(
            crate::queue::moderation::project_flag_run::PROJECT_FLAG_RUN_CONSUMER_GROUP,
        ).wrap_err("creating project flag lag client")?;

        Ok(Self {
            project_flag_lag_consumer: Arc::new(project_flag_lag_consumer),
            project_flag_run_producer: Arc::new(Mutex::new(Some(
                project_flag_run_producer,
            ))),
            client,
            incremental_index_search_consumer,
        })
    }

    /// Counts group members assigned partitions of the project scan topic.
    pub async fn project_flag_scan_consumers(&self) -> eyre::Result<usize> {
        let consumer = self.project_flag_lag_consumer.clone();
        tokio::task::spawn_blocking(move || {
            use crate::queue::moderation::project_flag_run::{
                PROJECT_FLAG_RUN_CONSUMER_GROUP, PROJECT_FLAG_RUN_TOPIC,
            };
            let groups = consumer
                .fetch_group_list(
                    Some(PROJECT_FLAG_RUN_CONSUMER_GROUP),
                    KAFKA_OPERATION_INTERVAL,
                )
                .wrap_err("fetching project flag consumer group members")?;
            let mut count = 0;
            for group in groups
                .groups()
                .iter()
                .filter(|group| group.name() == PROJECT_FLAG_RUN_CONSUMER_GROUP)
            {
                if group.members().is_empty() {
                    continue;
                }
                ensure!(
                    group.protocol_type() == "consumer",
                    "unexpected project flag consumer group protocol"
                );
                for member in group.members() {
                    if let Some(assignment) = member.assignment()
                        && assignment_has_topic(
                            assignment,
                            PROJECT_FLAG_RUN_TOPIC,
                        )?
                    {
                        count += 1;
                    }
                }
            }
            Ok(count)
        })
        .await
        .wrap_err("joining project flag consumer group lookup")?
    }

    /// Reads topic-wide group lag without subscribing, consuming, or committing offsets.
    pub async fn project_flag_scan_lag(&self) -> eyre::Result<i64> {
        let consumer = self.project_flag_lag_consumer.clone();
        tokio::task::spawn_blocking(move || {
            let topic = crate::queue::moderation::project_flag_run::PROJECT_FLAG_RUN_TOPIC;
            let deadline = Instant::now() + KAFKA_OPERATION_INTERVAL;
            let remaining = || deadline.checked_duration_since(Instant::now())
                .ok_or_else(|| eyre!("timed out fetching project flag scan lag"));
            let metadata = consumer.fetch_metadata(Some(topic), remaining()?)
                .wrap_err("fetching project flag topic metadata")?;
            let metadata = metadata.topics().iter().find(|metadata| metadata.name() == topic)
                .ok_or_else(|| eyre!("project flag topic metadata missing"))?;
            if let Some(error) = metadata.error() {
                return Err(eyre!("project flag topic metadata error: {error:?}"));
            }
            let mut partitions = TopicPartitionList::new();
            for partition in metadata.partitions() {
                partitions.add_partition(topic, partition.id());
            }
            let committed = consumer.committed_offsets(partitions, remaining()?)
                .wrap_err("fetching project flag consumer group offsets")?;
            let mut lag = 0i64;
            for partition in committed.elements() {
                partition.error().wrap_err("fetching project flag partition offset")?;
                let (low, high) = consumer.fetch_watermarks(topic, partition.partition(), remaining()?)
                    .wrap_err("fetching project flag partition watermarks")?;
                let offset = match partition.offset() {
                    Offset::Offset(offset) => offset.max(low),
                    Offset::Invalid | Offset::Beginning => low,
                    offset => return Err(eyre!("unexpected project flag consumer offset: {offset:?}")),
                };
                lag = lag.saturating_add(high.saturating_sub(offset).max(0));
            }
            Ok(lag)
        }).await.wrap_err("joining project flag scan lag lookup")?
    }
}

/// Kafka's classic ConsumerProtocolAssignment contains a version and topic/partition arrays.
fn assignment_has_topic(
    mut assignment: &[u8],
    topic: &str,
) -> eyre::Result<bool> {
    if assignment.is_empty() {
        return Ok(false);
    }
    let version = i16::from_be_bytes(take_assignment_bytes(&mut assignment)?);
    ensure!(
        version == 0,
        "unsupported Kafka consumer assignment version: {version}"
    );
    let topics = usize::try_from(i32::from_be_bytes(take_assignment_bytes(
        &mut assignment,
    )?))
    .wrap_err("decoding Kafka consumer assignment topic count")?;
    let mut assigned = false;
    for _ in 0..topics {
        let name_length = usize::try_from(i16::from_be_bytes(
            take_assignment_bytes(&mut assignment)?,
        ))
        .wrap_err("decoding Kafka assignment topic length")?;
        let (name, rest) = assignment
            .split_at_checked(name_length)
            .ok_or_else(|| eyre!("truncated Kafka assignment topic"))?;
        assignment = rest;
        let partitions = usize::try_from(i32::from_be_bytes(
            take_assignment_bytes(&mut assignment)?,
        ))
        .wrap_err("decoding Kafka assignment partition count")?;
        let length = partitions.checked_mul(4).ok_or_else(|| {
            eyre!("Kafka assignment partition count overflow")
        })?;
        let (_, rest) = assignment
            .split_at_checked(length)
            .ok_or_else(|| eyre!("truncated Kafka assignment partitions"))?;
        assignment = rest;
        assigned |= name == topic.as_bytes() && partitions > 0;
    }
    Ok(assigned)
}

fn take_assignment_bytes<const N: usize>(
    input: &mut &[u8],
) -> eyre::Result<[u8; N]> {
    let (bytes, rest) = input
        .split_first_chunk::<N>()
        .ok_or_else(|| eyre!("truncated Kafka consumer assignment"))?;
    *input = rest;
    Ok(*bytes)
}

pub fn create_project_flag_run_producer() -> eyre::Result<FutureProducer> {
    let producer: FutureProducer = producer_config()
        .set("enable.idempotence", "true")
        .set(
            "transactional.id",
            format!("project-flag-run-{}", Uuid::now_v7()),
        )
        .set("transaction.timeout.ms", "60000")
        .create()
        .wrap_err("creating project flag run Kafka producer")?;
    producer
        .init_transactions(KAFKA_DELIVERY_TIMEOUT)
        .wrap_err("initializing project flag run Kafka transaction")?;
    Ok(producer)
}

pub fn producer_config() -> ClientConfig {
    let mut config = ClientConfig::new();
    config
        .set("bootstrap.servers", ENV.KAFKA_BOOTSTRAP_SERVERS.0.join(","))
        .set("client.id", &ENV.KAFKA_CLIENT_ID)
        .set("acks", "all")
        .set("broker.address.family", "v4")
        .set(
            "delivery.timeout.ms",
            KAFKA_DELIVERY_TIMEOUT.as_millis().to_string(),
        );
    config
}

pub fn create_consumer(group_id: &str) -> eyre::Result<StreamConsumer> {
    consumer_config(group_id)
        .create()
        .wrap_err("failed to create Kafka consumer")
}

pub fn consumer_config(group_id: &str) -> ClientConfig {
    let mut config = ClientConfig::new();
    config
        .set("bootstrap.servers", ENV.KAFKA_BOOTSTRAP_SERVERS.0.join(","))
        .set("client.id", &ENV.KAFKA_CLIENT_ID)
        .set("group.id", group_id)
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "earliest")
        .set("max.poll.interval.ms", "1800000")
        .set("broker.address.family", "v4");
    config
}

/// Sentry distributed tracing headers for the active span, so consumers can
/// continue the producer's trace.
///
/// Empty outside of a span, since the scope's fallback propagation context
/// would otherwise join every background publish into one shared trace.
pub fn sentry_trace_headers() -> OwnedHeaders {
    sentry::configure_scope(|scope| {
        if scope.get_span().is_none() {
            return OwnedHeaders::new();
        }
        scope.iter_trace_propagation_headers().fold(
            OwnedHeaders::new(),
            |headers, (key, value)| {
                headers.insert(Header {
                    key,
                    value: Some(value.as_str()),
                })
            },
        )
    })
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
    service_version: env!("LABRINTH_VERSION"),
};
