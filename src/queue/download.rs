use crate::database::models::{DatabaseError, ProjectId, VersionId};
use sqlx::PgPool;
use tokio::sync::Mutex;

pub struct DownloadQueue {
    queue: Mutex<Vec<(ProjectId, VersionId)>>,
}

// Batches download transactions every thirty seconds
impl DownloadQueue {
    pub fn new() -> Self {
        DownloadQueue {
            queue: Mutex::new(Vec::with_capacity(1000)),
        }
    }
    pub async fn add(&self, project_id: ProjectId, version_id: VersionId) {
        self.queue.lock().await.push((project_id, version_id));
    }

    pub async fn take(&self) -> Vec<(ProjectId, VersionId)> {
        let mut queue = self.queue.lock().await;
        let len = queue.len();

        std::mem::replace(&mut queue, Vec::with_capacity(len))
    }

    pub async fn index(&self, pool: &PgPool) -> Result<(), DatabaseError> {
        let queue = self.take().await;

        if !queue.is_empty() {
            let mut transaction = pool.begin().await?;

            for (project_id, version_id) in queue {
                sqlx::query!(
                    "UPDATE versions
                    SET downloads = downloads + 1
                    WHERE (id = $1)",
                    version_id as VersionId
                )
                .execute(&mut *transaction)
                .await?;

                sqlx::query!(
                    "UPDATE mods
                    SET downloads = downloads + 1
                    WHERE (id = $1)",
                    project_id as ProjectId
                )
                .execute(&mut *transaction)
                .await?;
            }

            transaction.commit().await?;
        }

        Ok(())
    }
}
