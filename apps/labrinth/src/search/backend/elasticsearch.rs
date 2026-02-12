use crate::database::redis::RedisPool;
use crate::models::ids::VersionId;
use crate::search::backend::{
    BackendType, IndexingError, SearchBackend, TaskInfo, TaskStatus,
    TasksCancelFilter,
};
use crate::search::{
    ResultSearchProject, SearchRequest, SearchResults, UploadSearchProject,
};
use ariadne::ids::to_base62;
use async_trait::async_trait;
use elasticsearch::http::transport::{
    SingleNodeConnectionPool, TransportBuilder,
};
use elasticsearch::{Elasticsearch, SearchParts};
use serde_json::json;
use tracing::info;

pub struct ElasticsearchBackend {
    pub client: Elasticsearch,
    pub backend_type: BackendType,
    pub index_prefix: String,
}

impl ElasticsearchBackend {
    pub fn new(url: &str, index_prefix: &str) -> Result<Self, String> {
        let url_p = url.parse::<url::Url>().map_err(|e| e.to_string())?;
        let conn_pool = SingleNodeConnectionPool::new(url_p);
        let transport = TransportBuilder::new(conn_pool)
            .build()
            .map_err(|e| e.to_string())?;
        let client = Elasticsearch::new(transport);

        Ok(Self {
            client,
            backend_type: BackendType::Elasticsearch,
            index_prefix: index_prefix.to_string(),
        })
    }

    fn get_projects_index_name(&self, suffix: Option<&str>) -> String {
        if let Some(s) = suffix {
            format!("{}_projects_{}", self.index_prefix, s)
        } else {
            format!("{}_projects", self.index_prefix)
        }
    }
}

#[async_trait]
impl SearchBackend for ElasticsearchBackend {
    async fn search(
        &self,
        request: &SearchRequest,
    ) -> Result<SearchResults, SearchError> {
        let offset: usize = request.offset.as_deref().unwrap_or("0").parse()?;
        let index = request.index.as_deref().unwrap_or("relevance");
        let limit = request
            .limit
            .as_deref()
            .unwrap_or("10")
            .parse::<usize>()?
            .min(100);

        let index_name = match index {
            "relevance" => self.get_projects_index_name(Some("relevance")),
            "downloads" => self.get_projects_index_name(Some("filtered")),
            "follows" => self.get_projects_index_name(Some("relevance")),
            "updated" => self.get_projects_index_name(Some("relevance")),
            "newest" => self.get_projects_index_name(Some("relevance")),
            i => return Err(SearchError::InvalidIndex(i.to_string())),
        };

        let sort_field = match index {
            "downloads" => "downloads",
            "follows" => "follows",
            "updated" => "date_modified",
            "newest" => "date_created",
            _ => "_score",
        };

        let mut query_body = json!({
            "from": offset,
            "size": limit,
            "sort": [format!("{}:desc", sort_field)],
        });

        if let Some(q) = request.query.as_deref() {
            if !q.is_empty() {
                query_body["query"] = json!({
                    "multi_match": {
                        "query": q,
                        "fields": ["name^3", "summary^2", "author"]
                    }
                });
            }
        }

        if let Some(f) = request.filters.as_deref() {
            query_body["query"] = json!({
                "query_string": {
                    "query": f
                }
            });
        }

        info!("Executing Elasticsearch search on index: {}", index_name);
        let response = self
            .client
            .search(SearchParts::Index(&[&index_name]))
            .body(query_body)
            .send()
            .await
            .map_err(|e| SearchError::Elasticsearch(e.to_string()))?;

        let response_body: serde_json::Value =
            response.json().await.map_err(|e| SearchError::Serde(e))?;

        let hits =
            response_body["hits"]["hits"].as_array().ok_or_else(|| {
                SearchError::Serde("No hits array in response".into())
            })?;

        let total_hits = response_body["hits"]["total"]
            .as_object()
            .and_then(|o| o.get("value"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as usize;

        let results: Vec<ResultSearchProject> = hits
            .iter()
            .filter_map(|hit| hit["_source"].as_object().cloned())
            .map(|source| serde_json::from_value(source.into()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| SearchError::Serde(e))?;

        Ok(SearchResults {
            hits: results,
            page: (offset / limit) + 1,
            hits_per_page: limit,
            total_hits,
        })
    }

    async fn index_projects(
        &self,
        ro_pool: sqlx::PgPool,
        _redis: RedisPool,
    ) -> Result<(), IndexingError> {
        info!("Elasticsearch indexing not yet implemented");
        Ok(())
    }

    async fn remove_documents(
        &self,
        ids: &[VersionId],
    ) -> Result<(), IndexingError> {
        info!("Removing {} documents from Elasticsearch", ids.len());
        Ok(())
    }

    async fn get_tasks(&self) -> Result<Vec<TaskInfo>, IndexingError> {
        info!("Getting Elasticsearch tasks not yet implemented");
        Ok(vec![])
    }

    async fn cancel_tasks(
        &self,
        _filter: TasksCancelFilter,
    ) -> Result<(), IndexingError> {
        info!("Cancelling Elasticsearch tasks not yet implemented");
        Ok(())
    }
}
