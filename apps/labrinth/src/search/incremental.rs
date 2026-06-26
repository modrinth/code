pub mod consume;

use std::{collections::HashSet, mem, sync::Arc, time::Duration};

use rdkafka::{producer::FutureRecord, util::Timeout};
use serde::Serialize;
use tokio::sync::Mutex;

use crate::{
    models::ids::ProjectId,
    util::kafka::{KAFKA_OPERATION_INTERVAL, KafkaClientState, KafkaEvent},
};

pub const SEARCH_PROJECT_INDEX_QUEUE_TOPIC: &str =
    "public.labrinth.search-project-index-queue.v1";
const QUEUE_FLUSH_INTERVAL: Duration = Duration::from_secs(10);

#[derive(Clone)]
pub struct IncrementalSearchQueue {
    project_ids: Arc<Mutex<HashSet<ProjectId>>>,
    kafka_client: actix_web::web::Data<KafkaClientState>,
}

impl IncrementalSearchQueue {
    pub fn new(kafka_client: actix_web::web::Data<KafkaClientState>) -> Self {
        Self {
            project_ids: Arc::new(Mutex::new(HashSet::new())),
            kafka_client,
        }
    }

    pub async fn push(&self, project_id: ProjectId) {
        self.project_ids.lock().await.insert(project_id);
    }

    pub async fn run(self) {
        loop {
            tokio::time::sleep(QUEUE_FLUSH_INTERVAL).await;

            if let Err(err) = self.drain().await {
                tracing::error!(
                    "Failed to drain incremental search queue: {err:?}"
                );
            }
        }
    }

    pub async fn drain(&self) -> eyre::Result<()> {
        let project_ids = {
            let mut project_ids = self.project_ids.lock().await;
            mem::take(&mut *project_ids)
        };

        if project_ids.is_empty() {
            return Ok(());
        }

        let mut project_ids = project_ids.into_iter();
        while let Some(project_id) = project_ids.next() {
            let event = KafkaEvent::new(
                SEARCH_PROJECT_INDEX_QUEUE_TOPIC,
                SearchProjectIndexQueueEventData { project_id },
            );
            let event_id = event.event_metadata.event_id;
            let key = event_id.to_string();
            let payload = serde_json::to_vec(&event)?;
            let record = FutureRecord::to(SEARCH_PROJECT_INDEX_QUEUE_TOPIC)
                .key(&key)
                .payload(&payload);

            if let Err((err, _)) = self
                .kafka_client
                .client
                .send(record, Timeout::After(KAFKA_OPERATION_INTERVAL))
                .await
            {
                let mut queued_project_ids = self.project_ids.lock().await;
                queued_project_ids.insert(project_id);
                queued_project_ids.extend(project_ids);

                return Err(err.into());
            }
        }

        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct SearchProjectIndexQueueEventData {
    pub project_id: ProjectId,
}
