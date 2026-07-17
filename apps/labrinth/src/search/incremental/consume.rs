use actix_web::web;
use eyre::WrapErr;
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
use tracing::{Instrument, info, info_span};

use crate::{
    database::{PgPool, redis::RedisPool},
    env::ENV,
    models::ids::{ProjectId, VersionId},
    search::{
        SearchBackend, SearchDocumentBatch, SearchIndexUpdate,
        UploadSearchProject,
        incremental::SEARCH_PROJECT_INDEX_QUEUE_TOPIC,
        indexing::{build_project_documents, build_version_change_documents},
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
        // so that we can process a big batch to reindex.
        // we stop until either we've reached the max batch size,
        // or we've waited enough time - whichever is first.
        //
        // do a little trick with an `AsyncFnMut` closure
        // so that we can explicitly specify the return type
        let mut collect_more_messages = async || -> eyre::Result<()> {
            while messages.len() < ENV.SEARCH_INCREMENTAL_INDEX_BATCH_MAX_SIZE {
                let message = consumer
                    .recv()
                    .await
                    .wrap_err("failed to receive Kafka message")?;
                messages.push(message);
            }
            eyre::Ok(())
        };
        match tokio::time::timeout(delay, collect_more_messages()).await {
            Ok(Ok(())) | Err(_) => {}
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

    let mut project_ids_to_change = HashSet::new();
    let mut project_ids_with_version_changes = HashSet::new();
    let mut project_ids_to_remove = HashSet::new();
    let mut version_ids_to_change = HashSet::new();
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

        match event.into_data() {
            SearchProjectIndexQueueEventData::Change { project_id } => {
                project_ids_to_change.insert(project_id);
            }
            SearchProjectIndexQueueEventData::VersionChange {
                project_id,
                version_ids,
            } => {
                if !version_ids.is_empty() {
                    project_ids_with_version_changes.insert(project_id);
                    version_ids_to_change.extend(version_ids);
                }
            }
            SearchProjectIndexQueueEventData::Removal { project_id } => {
                project_ids_to_remove.insert(project_id);
            }
        }
        messages_to_commit.push(message);
    }

    project_ids_to_change
        .retain(|project_id| !project_ids_to_remove.contains(project_id));
    project_ids_with_version_changes
        .retain(|project_id| !project_ids_to_remove.contains(project_id));
    project_ids_to_change.retain(|project_id| {
        !project_ids_with_version_changes.contains(project_id)
    });
    let project_ids_to_change =
        project_ids_to_change.into_iter().collect::<Vec<_>>();
    let project_ids_with_version_changes = project_ids_with_version_changes
        .into_iter()
        .collect::<Vec<_>>();
    let project_ids_to_remove =
        project_ids_to_remove.into_iter().collect::<Vec<_>>();
    let version_ids_to_change =
        version_ids_to_change.into_iter().collect::<Vec<_>>();

    info!(
        kafka.message_count = messages_to_commit.len(),
        "Read all Kafka messages in {:.2?}, found {} projects to change, {} projects with {} version changes, and {} projects to remove",
        start.elapsed(),
        project_ids_to_change.len(),
        project_ids_with_version_changes.len(),
        version_ids_to_change.len(),
        project_ids_to_remove.len(),
    );
    let start = Instant::now();
    let mut documents = SearchDocumentBatch::default();

    if !project_ids_with_version_changes.is_empty() {
        let operation_start = Instant::now();
        let changed_documents = build_version_change_documents(
            ro_pool,
            redis_pool,
            &project_ids_with_version_changes,
            &version_ids_to_change,
        )
        .instrument(info_span!(
            "index",
            batch_size = project_ids_with_version_changes.len(),
            version_count = version_ids_to_change.len()
        ))
        .await
        .wrap_err_with(|| {
            format!(
                "failed to build search documents for {} projects and {} versions",
                project_ids_with_version_changes.len(),
                version_ids_to_change.len()
            )
        })?;
        documents.projects.extend(changed_documents.projects);
        documents.versions.extend(changed_documents.versions);
        info!(
            project_count = project_ids_with_version_changes.len(),
            version_count = version_ids_to_change.len(),
            "Built changed project versions in {:.2?}",
            operation_start.elapsed()
        );
    }

    if !project_ids_to_change.is_empty() {
        let operation_start = Instant::now();
        info!(
            project_count = project_ids_to_change.len(),
            "Building changed projects"
        );
        documents.projects.extend(
            build_changed_project_documents(
                ro_pool,
                redis_pool,
                &project_ids_to_change,
            )
            .await
            .wrap_err("failed to build changed projects")?,
        );
        info!(
            project_count = project_ids_to_change.len(),
            "Built changed projects in {:.2?}",
            operation_start.elapsed()
        );
    }

    let operation_start = Instant::now();
    search_backend
        .apply_update(SearchIndexUpdate {
            projects: &documents.projects,
            versions: &documents.versions,
            removed_projects: &project_ids_to_remove,
            removed_versions: &version_ids_to_change,
        })
        .await
        .wrap_err("failed to apply search index update")?;
    info!(
        project_count = documents.projects.len(),
        version_count = documents.versions.len(),
        removed_project_count = project_ids_to_remove.len(),
        removed_version_count = version_ids_to_change.len(),
        "Applied search index update in {:.2?}",
        operation_start.elapsed()
    );

    for message in messages_to_commit {
        consumer
            .commit_message(&message, CommitMode::Async)
            .wrap_err("failed to commit Kafka message")?;
    }

    info!(
        "Changed {} projects and removed {} projects in {:.2?}",
        project_ids_to_change.len(),
        project_ids_to_remove.len(),
        start.elapsed()
    );

    Ok(())
}

pub async fn reindex_project_document(
    ro_pool: &PgPool,
    redis_pool: &RedisPool,
    search_backend: &dyn SearchBackend,
    project_id: ProjectId,
) -> eyre::Result<()> {
    reindex_project_documents(
        ro_pool,
        redis_pool,
        search_backend,
        &[project_id],
    )
    .await
}

pub async fn reindex_project_documents(
    ro_pool: &PgPool,
    redis_pool: &RedisPool,
    search_backend: &dyn SearchBackend,
    project_ids: &[ProjectId],
) -> eyre::Result<()> {
    info!("Creating project documents");
    let projects =
        build_changed_project_documents(ro_pool, redis_pool, project_ids)
            .await?;
    search_backend
        .apply_update(SearchIndexUpdate {
            projects: &projects,
            ..SearchIndexUpdate::default()
        })
        .await?;

    Ok(())
}

async fn build_changed_project_documents(
    ro_pool: &PgPool,
    redis_pool: &RedisPool,
    project_ids: &[ProjectId],
) -> eyre::Result<Vec<UploadSearchProject>> {
    let documents = build_project_documents(ro_pool, redis_pool, project_ids)
        .instrument(info_span!("index", batch_size = project_ids.len()))
        .await
        .wrap_err_with(|| {
            format!(
                "failed to build search documents for {} projects",
                project_ids.len()
            )
        })?;

    info!("Fetched all project documents");
    Ok(documents)
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum SearchProjectIndexQueueEvent {
    Current(SearchProjectIndexQueueEventData),
    Legacy { project_id: ProjectId },
}

impl SearchProjectIndexQueueEvent {
    fn into_data(self) -> SearchProjectIndexQueueEventData {
        match self {
            Self::Current(data) => data,
            Self::Legacy { project_id } => {
                SearchProjectIndexQueueEventData::Change { project_id }
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum SearchProjectIndexQueueEventData {
    #[serde(rename = "project_change")]
    Change { project_id: ProjectId },
    #[serde(rename = "project_version_change")]
    VersionChange {
        project_id: ProjectId,
        version_ids: Vec<VersionId>,
    },
    #[serde(rename = "project_removal")]
    Removal { project_id: ProjectId },
}
