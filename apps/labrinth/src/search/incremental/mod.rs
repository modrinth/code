pub mod consume;

use std::{mem, sync::Arc};

use kafka::client::{ProduceMessage, RequiredAcks};
use serde::Serialize;
use tokio::sync::Mutex;

use crate::{
    models::ids::ProjectId,
    util::kafka::{KAFKA_OPERATION_INTERVAL, KafkaClientState, KafkaEvent},
};

pub const SEARCH_PROJECT_INDEX_QUEUE_TOPIC: &str =
    "public.labrinth.search_project_index_queue.v1";

#[derive(Debug, Clone)]
pub struct IncrementalSearchQueue {
    operations: Arc<Mutex<Vec<SearchIndexOperation>>>,
    kafka_client: actix_web::web::Data<KafkaClientState>,
}

impl IncrementalSearchQueue {
    pub fn new(kafka_client: actix_web::web::Data<KafkaClientState>) -> Self {
        Self {
            operations: Arc::new(Mutex::new(Vec::new())),
            kafka_client,
        }
    }

    pub async fn push(&self, project_id: ProjectId) {
        self.operations
            .lock()
            .await
            .push(SearchIndexOperation { project_id });
    }

    pub async fn run(self) {
        loop {
            tokio::time::sleep(KAFKA_OPERATION_INTERVAL).await;

            if let Err(err) = self.drain().await {
                tracing::error!(
                    "Failed to drain incremental search queue: {err:?}"
                );
            }
        }
    }

    async fn drain(&self) -> eyre::Result<()> {
        let operations = {
            let mut operations = self.operations.lock().await;
            mem::take(&mut *operations)
        };

        if operations.is_empty() {
            return Ok(());
        }

        let mut client = self.kafka_client.lock().await;

        let mut operations = operations.into_iter();
        while let Some(operation) = operations.next() {
            let event = KafkaEvent::new(
                SEARCH_PROJECT_INDEX_QUEUE_TOPIC,
                SearchProjectIndexQueueEventData {
                    project_id: operation.project_id,
                },
            );
            let event_id = event.event_metadata.event_id;
            let key = event_id.to_string();
            let payload = serde_json::to_vec(&event)?;
            let message = ProduceMessage::new(
                SEARCH_PROJECT_INDEX_QUEUE_TOPIC,
                0,
                Some(key.as_bytes()),
                Some(payload.as_slice()),
            );

            if let Err(err) = client.produce_messages(
                RequiredAcks::One,
                KAFKA_OPERATION_INTERVAL,
                [message],
            ) {
                let mut queued_operations = self.operations.lock().await;
                queued_operations.push(operation);
                queued_operations.extend(operations);

                return Err(err.into());
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SearchIndexOperation {
    pub project_id: ProjectId,
}

#[derive(Debug, Serialize)]
pub struct SearchProjectIndexQueueEventData {
    pub project_id: ProjectId,
}
