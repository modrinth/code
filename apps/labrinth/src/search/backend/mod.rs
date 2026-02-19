use crate::models::projects::SearchRequest;
use crate::search::SearchResults;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    Meilisearch,
    Elasticsearch,
}

#[derive(Debug, Clone)]
pub struct TasksCancelFilter {
    pub index_name: Option<String>,
    pub cancel_all: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub uid: u32,
    pub status: TaskStatus,
    pub duration: Option<String>,
    pub enqueued_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Enqueued,
    Processing,
    Failed,
    Succeeded,
}

#[async_trait]
pub trait SearchBackend: Send + Sync {
    async fn search(
        &self,
        request: &SearchRequest,
    ) -> Result<SearchResults, SearchError>;

    async fn index_projects(
        &self,
        ro_pool: sqlx::PgPool,
        redis: crate::database::redis::RedisPool,
    ) -> Result<(), IndexingError>;

    async fn remove_documents(
        &self,
        ids: &[crate::models::ids::VersionId],
    ) -> Result<(), IndexingError>;

    async fn get_tasks(&self) -> Result<Vec<TaskInfo>, IndexingError>;

    async fn cancel_tasks(
        &self,
        filter: TasksCancelFilter,
    ) -> Result<(), IndexingError>;
}

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("Meilisearch Error: {0}")]
    Meilisearch(#[from] meilisearch_sdk::errors::Error),
    #[error("Elasticsearch Error: {0}")]
    Elasticsearch(String),
    #[error("Error while serializing or deserializing JSON: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Error while parsing an integer: {0}")]
    IntParsing(#[from] std::num::ParseIntError),
    #[error("Error while formatting strings: {0}")]
    FormatError(#[from] std::fmt::Error),
    #[error("Environment Error")]
    Env(#[from] dotenvy::Error),
    #[error("Invalid index to sort by: {0}")]
    InvalidIndex(String),
    #[error("Unknown backend type: {0}")]
    UnknownBackend(String),
}

#[derive(Error, Debug)]
pub enum IndexingError {
    #[error("Meilisearch Indexing Error: {0}")]
    Meilisearch(String),
    #[error("Elasticsearch Indexing Error: {0}")]
    Elasticsearch(String),
    #[error("Database Error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Error while serializing or deserializing JSON: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Error while awaiting index creation task")]
    Task,
    #[error("Environment Error")]
    Env(#[from] dotenvy::Error),
}

pub fn get_backend() -> Result<
    Box<dyn crate::search::backend::SearchBackend + Send + Sync>,
    SearchError,
> {
    use crate::search::backend::SearchBackend;
    use crate::search::backend::{
        BackendKind, ElasticsearchBackend, MeilisearchBackend,
    };

    let backend_type_str = dotenvy::var("SEARCH_BACKEND")
        .unwrap_or_else(|_| "meilisearch".to_string());

    let backend_type = match backend_type_str.as_str() {
        "elasticsearch" => BackendKind::Elasticsearch,
        _ => BackendKind::Meilisearch,
    };

    match backend_type {
        BackendKind::Elasticsearch => {
            let url = dotenvy::var("ELASTICSEARCH_URL")
                .unwrap_or_else(|_| "http://localhost:9200".to_string());
            let index_prefix = dotenvy::var("ELASTICSEARCH_INDEX_PREFIX")
                .unwrap_or_else(|_| "labrinth".to_string());

            info!(
                "Using Elasticsearch backend at {} with prefix {}",
                url, index_prefix
            );
            let backend = ElasticsearchBackend::new(&url, &index_prefix)
                .map_err(|e| SearchError::Elasticsearch(e))?;
            Ok(Box::new(backend))
        }
        BackendKind::Meilisearch => {
            let meta_namespace =
                dotenvy::var("MEILISEARCH_META_NAMESPACE").ok();
            info!("Using Meilisearch backend");
            let config = SearchConfig::new(meta_namespace);
            let backend = MeilisearchBackend::new(config);
            Ok(Box::new(backend))
        }
    }
}
