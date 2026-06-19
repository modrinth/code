use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};
use serde::Deserialize;

use crate::{
    env::ENV, models::ids::ProjectId,
    search::incremental::SEARCH_PROJECT_INDEX_QUEUE_TOPIC,
    util::kafka::KAFKA_OPERATION_INTERVAL,
};

pub const INCREMENTAL_INDEX_SEARCH_TASK: &str = "incremental-index-search";

pub async fn run() -> eyre::Result<()> {
    let mut consumer =
        Consumer::from_hosts(ENV.KAFKA_BOOTSTRAP_SERVERS.0.clone())
            .with_topic(SEARCH_PROJECT_INDEX_QUEUE_TOPIC.to_string())
            .with_group(INCREMENTAL_INDEX_SEARCH_TASK.to_string())
            .with_client_id(ENV.KAFKA_CLIENT_ID.clone())
            .with_fallback_offset(FetchOffset::Earliest)
            .with_offset_storage(Some(GroupOffsetStorage::Kafka))
            .create()
            .wrap_err("failed to create kafka consumer")?;

    tracing::info!(
        kafka.bootstrap_servers = ?ENV.KAFKA_BOOTSTRAP_SERVERS.0,
        kafka.client_id = %ENV.KAFKA_CLIENT_ID,
        kafka.topic = SEARCH_PROJECT_INDEX_QUEUE_TOPIC,
        kafka.consumer_group = INCREMENTAL_INDEX_SEARCH_TASK,
        "Started Kafka consumer"
    );

    loop {
        let message_sets = consumer.poll()?;

        if message_sets.is_empty() {
            tokio::time::sleep(KAFKA_OPERATION_INTERVAL).await;
            continue;
        }

        for message_set in message_sets.iter() {
            for message in message_set.messages() {
                let event = serde_json::from_slice::<
                    SearchProjectIndexQueueEvent,
                >(message.value)?;

                tracing::info!(
                    kafka.topic = message_set.topic(),
                    kafka.partition = message_set.partition(),
                    kafka.offset = message.offset,
                    project_id = %event.project_id,
                    "Consumed incremental search index event"
                );

                // TODO: Incrementally index the consumed project.
            }

            consumer.consume_messageset(message_set)?;
        }

        consumer.commit_consumed()?;
    }
}

#[derive(Debug, Deserialize)]
struct SearchProjectIndexQueueEvent {
    project_id: ProjectId,
}
