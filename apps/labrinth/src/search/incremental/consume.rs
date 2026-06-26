use actix_web::web;
use eyre::WrapErr;
use futures::never::Never;
use rdkafka::{
    Message,
    consumer::{CommitMode, Consumer, StreamConsumer},
    message::BorrowedMessage,
};
use serde::Deserialize;
use std::{
    collections::HashSet,
    time::{Duration, Instant},
};
use tracing::info;

use crate::{
    database::{PgPool, redis::RedisPool},
    env::ENV,
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
    // keep buffer capacity (pre-)allocated
    let mut messages = Vec::with_capacity(1024);
    loop {
        messages.clear();

        // wait for a first message to come in...
        let first_message = consumer
            .recv()
            .await
            .wrap_err("failed to receive Kafka message")?;
        messages.push(first_message);

        let delay = Duration::from_secs(
            ENV.SEARCH_INCREMENTAL_INDEX_BATCH_DELAY_SECONDS,
        );
        info!(
            "Received initial Kafka message; waiting {delay:.2?} for more to batch",
        );

        // ..then wait a while for more messages to batch up
        // so that we can process a big batch to reindex
        //
        // do a little trick with an `AsyncFnMut` closure
        // so that we can explicitly specify the return type
        let mut collect_more_messages = async || -> eyre::Result<Never> {
            loop {
                let message = consumer
                    .recv()
                    .await
                    .wrap_err("failed to receive Kafka message")?;
                messages.push(message);
            }
        };
        match tokio::time::timeout(delay, collect_more_messages()).await {
            Err(_elapsed) => {}
            Ok(Err(err)) => {
                return Err(
                    err.wrap_err("failed to receive more Kafka messages")
                );
            }
        }

        info!("Consuming batch of {} messages", messages.len());
        consume_batch(
            ro_pool,
            redis_pool,
            search_backend,
            consumer,
            messages.drain(..),
        )
        .await?;
    }
}

async fn consume_batch(
    ro_pool: &PgPool,
    redis_pool: &RedisPool,
    search_backend: &dyn SearchBackend,
    consumer: &StreamConsumer,
    messages: impl IntoIterator<Item = BorrowedMessage<'_>>,
) -> eyre::Result<()> {
    let start = Instant::now();

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

    info!(
        kafka.message_count = messages_to_commit.len(),
        "Read all Kafka messages in {:.2?}, found {} projects to reindex",
        start.elapsed(),
        project_ids.len(),
    );
    let start = Instant::now();

    if !project_ids.is_empty() {
        reindex_projects(ro_pool, redis_pool, search_backend, &project_ids)
            .await
            .wrap_err("failed to reindex project batch")?;
    }

    for message in messages_to_commit {
        consumer
            .commit_message(&message, CommitMode::Async)
            .wrap_err("failed to commit Kafka message")?;
    }

    info!(
        "Reindexed {} projects in {:.2?}",
        project_ids.len(),
        start.elapsed()
    );

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
