use actix_web::web;
use eyre::WrapErr;
use futures::FutureExt;
use rdkafka::{
    Message,
    consumer::{CommitMode, Consumer, StreamConsumer},
    message::BorrowedMessage,
};
use serde::Deserialize;
use std::collections::HashSet;

use crate::{
    database::{PgPool, redis::RedisPool},
    models::ids::ProjectId,
    search::{
        SearchBackend, incremental::SEARCH_PROJECT_INDEX_QUEUE_TOPIC,
        indexing::index_project_documents,
    },
    util::kafka::{
        INCREMENTAL_INDEX_SEARCH_TASK, KAFKA_OPERATION_INTERVAL,
        KafkaClientState,
    },
};

const BATCH_SIZE: usize = 100;

pub async fn run(
    ro_pool: PgPool,
    redis_pool: RedisPool,
    search_backend: web::Data<dyn SearchBackend>,
    kafka_client: web::Data<KafkaClientState>,
) -> eyre::Result<()> {
    let consumer = &kafka_client.incremental_index_search_consumer;
    consumer
        .subscribe(&[SEARCH_PROJECT_INDEX_QUEUE_TOPIC])
        .wrap_err("failed to subscribe to Kafka topic")?;

    tracing::info!(
        kafka.topic = SEARCH_PROJECT_INDEX_QUEUE_TOPIC,
        kafka.consumer_group = INCREMENTAL_INDEX_SEARCH_TASK,
        "Started Kafka consumer"
    );

    loop {
        if let Err(err) =
            consume(&ro_pool, &redis_pool, search_backend.as_ref(), consumer)
                .await
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
    consumer: &StreamConsumer,
) -> eyre::Result<()> {
    loop {
        let mut messages = Vec::with_capacity(BATCH_SIZE);
        messages.push(
            consumer
                .recv()
                .await
                .wrap_err("failed to receive Kafka message")?,
        );

        while messages.len() < BATCH_SIZE {
            let Some(message) = consumer.recv().now_or_never() else {
                break;
            };

            messages.push(message.wrap_err("failed to receive Kafka message")?);
        }

        consume_batch(ro_pool, redis_pool, search_backend, consumer, messages)
            .await?;
    }
}

async fn consume_batch(
    ro_pool: &PgPool,
    redis_pool: &RedisPool,
    search_backend: &dyn SearchBackend,
    consumer: &StreamConsumer,
    messages: Vec<BorrowedMessage<'_>>,
) -> eyre::Result<()> {
    let mut project_ids = Vec::new();
    let mut seen_project_ids = HashSet::new();
    let mut messages_to_commit = Vec::new();

    for message in messages {
        let Some(payload) = message.payload() else {
            tracing::error!(
                kafka.topic = message.topic(),
                kafka.partition = message.partition(),
                kafka.offset = message.offset(),
                "Skipping incremental search index event without payload"
            );
            consumer
                .commit_message(&message, CommitMode::Async)
                .wrap_err("failed to commit empty Kafka message")?;
            continue;
        };

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

        if seen_project_ids.insert(event.project_id) {
            project_ids.push(event.project_id);
        }
        messages_to_commit.push(message);
    }

    if project_ids.is_empty() {
        return Ok(());
    }

    tracing::info!(
        kafka.message_count = messages_to_commit.len(),
        project_count = project_ids.len(),
        "Consumed incremental search index event batch"
    );

    reindex_projects(ro_pool, redis_pool, search_backend, &project_ids)
        .await
        .wrap_err("failed to reindex project batch")?;

    for message in messages_to_commit {
        consumer
            .commit_message(&message, CommitMode::Async)
            .wrap_err("failed to commit Kafka message")?;
    }

    Ok(())
}

pub async fn reindex_project(
    ro_pool: &PgPool,
    redis_pool: &RedisPool,
    search_backend: &dyn SearchBackend,
    project_id: ProjectId,
) -> eyre::Result<()> {
    reindex_projects(ro_pool, redis_pool, search_backend, &[project_id]).await
}

pub async fn reindex_projects(
    ro_pool: &PgPool,
    redis_pool: &RedisPool,
    search_backend: &dyn SearchBackend,
    project_ids: &[ProjectId],
) -> eyre::Result<()> {
    search_backend.remove_project_documents(project_ids).await?;

    let mut documents = Vec::new();
    for project_id in project_ids {
        documents.extend(
            index_project_documents(ro_pool, redis_pool, *project_id)
                .await
                .wrap_err_with(|| {
                    format!(
                        "failed to build project {project_id} search documents"
                    )
                })?,
        );
    }

    search_backend.index_documents(&documents).await?;

    Ok(())
}

#[derive(Debug, Deserialize)]
struct SearchProjectIndexQueueEvent {
    project_id: ProjectId,
}
