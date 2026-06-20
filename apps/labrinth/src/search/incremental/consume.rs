use actix_web::web;
use eyre::WrapErr;
use rdkafka::{
    ClientConfig, Message,
    consumer::{CommitMode, Consumer, StreamConsumer},
};
use serde::Deserialize;

use crate::{
    database::{PgPool, redis::RedisPool},
    env::ENV,
    models::ids::ProjectId,
    search::{
        SearchBackend, incremental::SEARCH_PROJECT_INDEX_QUEUE_TOPIC,
        indexing::index_project_documents,
    },
    util::kafka::KAFKA_OPERATION_INTERVAL,
};

pub const INCREMENTAL_INDEX_SEARCH_TASK: &str = "incremental-index-search";

pub async fn run(
    ro_pool: PgPool,
    redis_pool: RedisPool,
    search_backend: web::Data<dyn SearchBackend>,
) -> eyre::Result<()> {
    loop {
        if let Err(err) =
            consume(&ro_pool, &redis_pool, search_backend.as_ref()).await
        {
            tracing::error!(
                "Background task {INCREMENTAL_INDEX_SEARCH_TASK} failed: {err:?}"
            );
            tokio::time::sleep(KAFKA_OPERATION_INTERVAL).await;
        }
    }
}

async fn consume(
    ro_pool: &PgPool,
    redis_pool: &RedisPool,
    search_backend: &dyn SearchBackend,
) -> eyre::Result<()> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", ENV.KAFKA_BOOTSTRAP_SERVERS.0.join(","))
        .set("client.id", &ENV.KAFKA_CLIENT_ID)
        .set("group.id", INCREMENTAL_INDEX_SEARCH_TASK)
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "earliest")
        .set("broker.address.family", "v4")
        .create()
        .wrap_err("failed to create Kafka consumer")?;
    consumer
        .subscribe(&[SEARCH_PROJECT_INDEX_QUEUE_TOPIC])
        .wrap_err("failed to subscribe to Kafka topic")?;

    tracing::info!(
        kafka.bootstrap_servers = ?ENV.KAFKA_BOOTSTRAP_SERVERS.0,
        kafka.client_id = %ENV.KAFKA_CLIENT_ID,
        kafka.topic = SEARCH_PROJECT_INDEX_QUEUE_TOPIC,
        kafka.consumer_group = INCREMENTAL_INDEX_SEARCH_TASK,
        "Started Kafka consumer"
    );

    loop {
        let message = consumer
            .recv()
            .await
            .wrap_err("failed to receive Kafka message")?;
        let payload = message.payload().ok_or_else(|| {
            eyre::eyre!("Kafka message did not have a payload")
        })?;
        let event = match serde_json::from_slice::<SearchProjectIndexQueueEvent>(
            payload,
        ) {
            Ok(event) => event,
            Err(err) => {
                tracing::error!(
                    kafka.topic = message.topic(),
                    kafka.partition = message.partition(),
                    kafka.offset = message.offset(),
                    "Skipping malformed incremental search index event: {err:?}"
                );
                consumer
                    .commit_message(&message, CommitMode::Async)
                    .wrap_err("failed to commit malformed Kafka message")?;
                continue;
            }
        };

        tracing::info!(
            kafka.topic = message.topic(),
            kafka.partition = message.partition(),
            kafka.offset = message.offset(),
            project_id = %event.project_id,
            "Consumed incremental search index event"
        );

        reindex_project(ro_pool, redis_pool, search_backend, event.project_id)
            .await
            .wrap_err_with(|| {
                format!("failed to reindex project {}", event.project_id)
            })?;

        consumer
            .commit_message(&message, CommitMode::Async)
            .wrap_err("failed to commit Kafka message")?;
    }
}

async fn reindex_project(
    ro_pool: &PgPool,
    redis_pool: &RedisPool,
    search_backend: &dyn SearchBackend,
    project_id: ProjectId,
) -> eyre::Result<()> {
    search_backend
        .remove_project_documents(&[project_id])
        .await?;

    let documents = index_project_documents(ro_pool, redis_pool, project_id)
        .await
        .wrap_err("failed to build project search documents")?;
    search_backend.index_documents(&documents).await?;

    Ok(())
}

#[derive(Debug, Deserialize)]
struct SearchProjectIndexQueueEvent {
    project_id: ProjectId,
}
