pub mod consume;

use std::{
    collections::{HashMap, HashSet},
    mem,
    sync::Arc,
    time::Duration,
};

use rdkafka::{producer::FutureRecord, util::Timeout};
use serde::Serialize;
use tokio::sync::Mutex;

use crate::{
    models::ids::{ProjectId, VersionId},
    util::kafka::{KAFKA_OPERATION_INTERVAL, KafkaClientState, KafkaEvent},
};

pub const SEARCH_PROJECT_INDEX_QUEUE_TOPIC: &str =
    "public.labrinth.search-project-index-queue.v1";
const QUEUE_FLUSH_INTERVAL: Duration = Duration::from_secs(10);

#[derive(Clone)]
pub struct IncrementalSearchQueue {
    operations: Arc<Mutex<PendingSearchIndexOperations>>,
    kafka_client: actix_web::web::Data<KafkaClientState>,
}

impl IncrementalSearchQueue {
    pub fn new(kafka_client: actix_web::web::Data<KafkaClientState>) -> Self {
        Self {
            operations: Arc::new(Mutex::new(
                PendingSearchIndexOperations::default(),
            )),
            kafka_client,
        }
    }

    pub async fn push(
        &self,
        project_id: ProjectId,
        version_ids: impl IntoIterator<Item = VersionId>,
    ) {
        self.operations
            .lock()
            .await
            .push_project_change(project_id, version_ids);
    }

    pub async fn push_project_removal(&self, project_id: ProjectId) {
        self.operations
            .lock()
            .await
            .push_project_removal(project_id);
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
        let operations = {
            let mut operations = self.operations.lock().await;
            mem::take(&mut *operations)
        };

        if operations.is_empty() {
            return Ok(());
        }

        let mut operations = operations.into_events().into_iter();
        while let Some(operation) = operations.next() {
            let event = KafkaEvent::new(
                SEARCH_PROJECT_INDEX_QUEUE_TOPIC,
                operation.clone(),
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
                let mut queued_operations = self.operations.lock().await;
                queued_operations.push_event(operation);
                for operation in operations {
                    queued_operations.push_event(operation);
                }

                return Err(err.into());
            }
        }

        Ok(())
    }
}

#[derive(Default)]
struct PendingSearchIndexOperations {
    changed_projects: HashMap<ProjectId, HashSet<VersionId>>,
    removed_project_ids: HashSet<ProjectId>,
}

impl PendingSearchIndexOperations {
    fn is_empty(&self) -> bool {
        self.changed_projects.is_empty() && self.removed_project_ids.is_empty()
    }

    fn push_project_change(
        &mut self,
        project_id: ProjectId,
        version_ids: impl IntoIterator<Item = VersionId>,
    ) {
        if !self.removed_project_ids.contains(&project_id) {
            self.changed_projects
                .entry(project_id)
                .or_default()
                .extend(version_ids);
        }
    }

    fn push_project_removal(&mut self, project_id: ProjectId) {
        self.changed_projects.remove(&project_id);
        self.removed_project_ids.insert(project_id);
    }

    fn push_event(&mut self, event: SearchProjectIndexQueueEventData) {
        match event {
            SearchProjectIndexQueueEventData::ProjectChange {
                project_id,
                version_ids,
            } => self.push_project_change(project_id, version_ids),
            SearchProjectIndexQueueEventData::ProjectRemoval { project_id } => {
                self.push_project_removal(project_id)
            }
        }
    }

    fn into_events(self) -> Vec<SearchProjectIndexQueueEventData> {
        let mut events = Vec::with_capacity(
            self.changed_projects.len() + self.removed_project_ids.len(),
        );

        events.extend(self.removed_project_ids.into_iter().map(|project_id| {
            SearchProjectIndexQueueEventData::ProjectRemoval { project_id }
        }));
        events.extend(self.changed_projects.into_iter().map(
            |(project_id, version_ids)| {
                SearchProjectIndexQueueEventData::ProjectChange {
                    project_id,
                    version_ids: version_ids.into_iter().collect(),
                }
            },
        ));

        events
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SearchProjectIndexQueueEventData {
    ProjectChange {
        project_id: ProjectId,
        version_ids: Vec<VersionId>,
    },
    ProjectRemoval {
        project_id: ProjectId,
    },
}
