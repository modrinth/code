use actix_web::web;
use eyre::WrapErr;
use kafka::client::{FetchOffset, FetchPartition, KafkaClient};
use serde::Deserialize;
use std::collections::HashMap;

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
    let mut client = KafkaClient::new(ENV.KAFKA_BOOTSTRAP_SERVERS.0.clone());
    client.set_client_id(ENV.KAFKA_CLIENT_ID.clone());
    client
        .load_metadata(&[SEARCH_PROJECT_INDEX_QUEUE_TOPIC])
        .wrap_err("failed to load Kafka metadata")?;

    let mut offsets = client
        .fetch_topic_offsets(
            SEARCH_PROJECT_INDEX_QUEUE_TOPIC,
            FetchOffset::Earliest,
        )
        .wrap_err("failed to fetch Kafka topic offsets")?
        .into_iter()
        .map(|partition_offset| {
            (partition_offset.partition, partition_offset.offset)
        })
        .collect::<HashMap<_, _>>();

    tracing::info!(
        kafka.bootstrap_servers = ?ENV.KAFKA_BOOTSTRAP_SERVERS.0,
        kafka.client_id = %ENV.KAFKA_CLIENT_ID,
        kafka.topic = SEARCH_PROJECT_INDEX_QUEUE_TOPIC,
        kafka.consumer_group = INCREMENTAL_INDEX_SEARCH_TASK,
        "Started Kafka consumer"
    );

    loop {
        let fetch_partitions = offsets
            .iter()
            .map(|(partition, offset)| {
                FetchPartition::new(
                    SEARCH_PROJECT_INDEX_QUEUE_TOPIC,
                    *partition,
                    *offset,
                )
            })
            .collect::<Vec<_>>();

        let responses = client
            .fetch_messages(fetch_partitions.iter())
            .wrap_err("failed to fetch Kafka messages")?;
        let mut consumed_any = false;

        for response in responses {
            for topic in response.topics() {
                for partition in topic.partitions() {
                    let data = partition.data().wrap_err_with(|| {
                        format!(
                            "failed to fetch Kafka partition {}:{}",
                            topic.topic(),
                            partition.partition()
                        )
                    })?;

                    for message in data.messages() {
                        let event = serde_json::from_slice::<
                            SearchProjectIndexQueueEvent,
                        >(message.value)?;

                        tracing::info!(
                            kafka.topic = topic.topic(),
                            kafka.partition = partition.partition(),
                            kafka.offset = message.offset,
                            project_id = %event.project_id,
                            "Consumed incremental search index event"
                        );

                        reindex_project(
                            ro_pool,
                            redis_pool,
                            search_backend,
                            event.project_id,
                        )
                        .await
                        .wrap_err_with(|| {
                            format!(
                                "failed to reindex project {}",
                                event.project_id
                            )
                        })?;

                        offsets
                            .insert(partition.partition(), message.offset + 1);
                        consumed_any = true;
                    }
                }
            }
        }

        if !consumed_any {
            tokio::time::sleep(KAFKA_OPERATION_INTERVAL).await;
        }
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
