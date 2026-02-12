use crate::database::redis::RedisPool;
use crate::models::ids::VersionId;
use crate::models::projects::SearchRequest;
use crate::search::backend::{
    BackendType, IndexingError, SearchBackend, TaskInfo, TaskStatus,
    TasksCancelFilter,
};
use crate::search::indexing::cancel_tasks as meilisearch_cancel_tasks;
use crate::search::indexing::get_tasks as meilisearch_get_tasks;
use crate::search::indexing::index_projects as meilisearch_index_projects;
use crate::search::indexing::remove_documents as meilisearch_remove_documents;
use crate::search::search_for_project as meilisearch_search;
use crate::search::{
    SearchConfig, SearchError, SearchResults, UploadSearchProject,
};
use async_trait::async_trait;

pub struct MeilisearchBackend {
    pub config: SearchConfig,
    pub backend_type: BackendType,
}

impl MeilisearchBackend {
    pub fn new(config: SearchConfig) -> Self {
        Self {
            config,
            backend_type: BackendType::Meilisearch,
        }
    }
}

#[async_trait]
impl SearchBackend for MeilisearchBackend {
    async fn search(
        &self,
        request: &SearchRequest,
    ) -> Result<SearchResults, SearchError> {
        meilisearch_search(request, &self.config).await
    }

    async fn index_projects(
        &self,
        ro_pool: sqlx::PgPool,
        redis: RedisPool,
    ) -> Result<(), IndexingError> {
        meilisearch_index_projects(ro_pool, redis, &self.config)
            .await
            .map_err(|e| IndexingError::Meilisearch(e.to_string()))
    }

    async fn remove_documents(
        &self,
        ids: &[VersionId],
    ) -> Result<(), IndexingError> {
        meilisearch_remove_documents(ids, &self.config)
            .await
            .map_err(|e| IndexingError::Meilisearch(e.to_string()))
    }

    async fn get_tasks(&self) -> Result<Vec<TaskInfo>, IndexingError> {
        meilisearch_get_tasks(&self.config)
            .await
            .map_err(|e| IndexingError::Meilisearch(e.to_string()))
    }

    async fn cancel_tasks(
        &self,
        filter: TasksCancelFilter,
    ) -> Result<(), IndexingError> {
        meilisearch_cancel_tasks(&self.config, filter)
            .await
            .map_err(|e| IndexingError::Meilisearch(e.to_string()))
    }
}
