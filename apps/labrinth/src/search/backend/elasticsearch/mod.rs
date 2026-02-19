use crate::database::PgPool;
use crate::database::redis::RedisPool;
use crate::env::ENV;
use crate::models::ids::VersionId;
use crate::models::projects::SearchRequest;
use crate::routes::ApiError;
use crate::search::indexing::index_local;
use crate::search::{
    ResultSearchProject, SearchBackend, SearchResults, TasksCancelFilter,
    UploadSearchProject,
};
use crate::util::error::Context;
use ariadne::ids::base62_impl::to_base62;
use async_trait::async_trait;
use elasticsearch::http::Url;
use elasticsearch::http::request::JsonBody;
use elasticsearch::http::response::Response;
use elasticsearch::http::transport::{
    SingleNodeConnectionPool, TransportBuilder,
};
use elasticsearch::indices::{
    IndicesCreateParts, IndicesDeleteParts, IndicesExistsParts,
    IndicesRefreshParts,
};
use elasticsearch::params::Refresh;
use elasticsearch::tasks::TasksCancelParts;
use elasticsearch::{
    BulkParts, DeleteByQueryParts, Elasticsearch as EsClient, SearchParts,
};
use eyre::eyre;
use reqwest::StatusCode;
use serde::Serialize;
use serde_json::{Value, json};
use std::borrow::Cow;
use std::collections::HashMap;
use std::time::Duration;

const INDEX_CHUNK_SIZE: i64 = 10_000;

#[derive(Debug, Clone)]
pub struct ElasticsearchConfig {
    pub url: String,
    pub index_prefix: String,
    pub meta_namespace: String,
}

impl ElasticsearchConfig {
    pub fn new(meta_namespace: Option<String>) -> Self {
        Self {
            url: ENV.ELASTICSEARCH_URL,
            index_prefix: ENV.ELASTICSEARCH_INDEX_PREFIX,
            meta_namespace: meta_namespace.unwrap_or_default(),
        }
    }

    pub fn get_index_name(&self, index: &str) -> String {
        if self.meta_namespace.is_empty() {
            format!("{}_{}", self.index_prefix, index)
        } else {
            format!("{}_{}_{}", self.meta_namespace, self.index_prefix, index)
        }
    }
}

pub struct Elasticsearch {
    pub config: ElasticsearchConfig,
    pub client: EsClient,
}

impl Elasticsearch {
    fn normalize_filter_field(field: &str) -> &str {
        match field {
            "project_type" => "project_types",
            _ => field,
        }
    }

    fn parse_condition_query(condition: &str) -> Value {
        let (field, value, negative) =
            if let Some((f, v)) = condition.split_once("!=") {
                (f.trim(), v.trim(), true)
            } else if let Some((f, v)) = condition.split_once(':') {
                (f.trim(), v.trim(), false)
            } else if let Some((f, v)) = condition.split_once('=') {
                (f.trim(), v.trim(), false)
            } else {
                ("", "", false)
            };

        let field = Self::normalize_filter_field(field);
        let term_query = json!({
            "term": {
                field: value
            }
        });

        if negative {
            json!({
                "bool": {
                    "must_not": [term_query]
                }
            })
        } else {
            term_query
        }
    }

    fn facets_filter_clauses(
        facets_json: Option<&str>,
    ) -> Result<Vec<Value>, ApiError> {
        let Some(raw_facets) = facets_json else {
            return Ok(Vec::new());
        };

        let facets = serde_json::from_str::<Vec<Vec<Value>>>(raw_facets)
            .wrap_request_err("failed to parse facets")?;

        let facets = facets
            .into_iter()
            .map(|facet_group| {
                facet_group
                    .into_iter()
                    .map(|facet| {
                        if facet.is_array() {
                            serde_json::from_value::<Vec<String>>(facet)
                                .unwrap_or_default()
                        } else {
                            vec![
                                serde_json::from_value::<String>(facet)
                                    .unwrap_or_default(),
                            ]
                        }
                    })
                    .collect::<Vec<Vec<String>>>()
            })
            .collect::<Vec<Vec<Vec<String>>>>();

        let mut clauses = Vec::new();
        for or_group in facets {
            let should = or_group
                .into_iter()
                .map(|and_group| {
                    let mut must = Vec::new();
                    let mut must_not = Vec::new();
                    for condition in and_group {
                        let q = Self::parse_condition_query(&condition);
                        if q.get("bool")
                            .and_then(|b| b.get("must_not"))
                            .is_some()
                        {
                            if let Some(parts) =
                                q["bool"]["must_not"].as_array()
                            {
                                must_not.extend(parts.iter().cloned());
                            }
                        } else {
                            must.push(q);
                        }
                    }

                    json!({
                        "bool": {
                            "must": must,
                            "must_not": must_not
                        }
                    })
                })
                .collect::<Vec<_>>();

            clauses.push(json!({
                "bool": {
                    "should": should,
                    "minimum_should_match": 1
                }
            }));
        }

        Ok(clauses)
    }

    pub fn new(meta_namespace: Option<String>) -> eyre::Result<Self> {
        let config = ElasticsearchConfig::new(meta_namespace);
        let url = Url::parse(&config.url)
            .wrap_err("failed to parse Elasticsearch URL")?;
        let transport =
            TransportBuilder::new(SingleNodeConnectionPool::new(url))
                .build()
                .wrap_err("failed to create Elasticsearch transport")?;
        let client = EsClient::new(transport);

        Ok(Self { config, client })
    }

    fn get_sort_index(&self, index: &str) -> Result<(String, Value), ApiError> {
        let projects_name = self.config.get_index_name("projects");
        let projects_filtered_name =
            self.config.get_index_name("projects_filtered");
        Ok(match index {
            "relevance" => (
                projects_name,
                json!([{ "_score": { "order": "desc" } }, { "downloads": { "order": "desc" } }]),
            ),
            "downloads" => (
                projects_filtered_name,
                json!([{ "downloads": { "order": "desc" } }]),
            ),
            "follows" => {
                (projects_name, json!([{ "follows": { "order": "desc" } }]))
            }
            "updated" => (
                projects_name,
                json!([{ "date_modified": { "order": "desc" } }]),
            ),
            "newest" => (
                projects_name,
                json!([{ "date_created": { "order": "desc" } }]),
            ),
            _ => {
                return Err(ApiError::Request(eyre!(
                    "invalid index `{index}`"
                )));
            }
        })
    }

    async fn ensure_index(&self, index_name: &str) -> Result<(), ApiError> {
        let exists = self
            .client
            .indices()
            .exists(IndicesExistsParts::Index(&[index_name]))
            .send()
            .await
            .wrap_internal_err(
                "failed to check Elasticsearch index existence",
            )?;

        if exists.status_code().is_success() {
            return Ok(());
        }

        let response = self
            .client
            .indices()
            .create(IndicesCreateParts::Index(index_name))
            .body(json!({
                "mappings": {
                    "dynamic": true,
                    "properties": {
                        "version_id": { "type": "keyword" },
                        "project_id": { "type": "keyword" },
                        "slug": {
                            "type": "search_as_you_type",
                            "fields": {
                                "keyword": { "type": "keyword" }
                            }
                        },
                        "author": {
                            "type": "search_as_you_type",
                            "fields": {
                                "keyword": { "type": "keyword" }
                            }
                        },
                        "name": { "type": "search_as_you_type" },
                        "summary": { "type": "search_as_you_type" },
                        "categories": { "type": "keyword" },
                        "display_categories": { "type": "keyword" },
                        "downloads": { "type": "integer" },
                        "follows": { "type": "integer" },
                        "date_created": { "type": "date" },
                        "date_modified": { "type": "date" },
                        "license": { "type": "keyword" },
                        "loaders": { "type": "keyword" }
                    }
                }
            }))
            .send()
            .await
            .wrap_internal_err("failed to create Elasticsearch index")?;

        if response.status_code().is_success() {
            Ok(())
        } else {
            let body =
                response.json::<Value>().await.unwrap_or_else(|_| json!({}));
            Err(ApiError::Internal(eyre!(
                "failed to create Elasticsearch index `{index_name}`: {body}"
            )))
        }
    }

    async fn reset_indexes(&self) -> Result<(), ApiError> {
        for index_name in [
            self.config.get_index_name("projects"),
            self.config.get_index_name("projects_filtered"),
        ] {
            let delete = self
                .client
                .indices()
                .delete(IndicesDeleteParts::Index(&[index_name.as_str()]))
                .send()
                .await
                .wrap_internal_err("failed to delete Elasticsearch index")?;

            let success_or_not_found = delete.status_code().is_success()
                || delete.status_code() == StatusCode::NOT_FOUND;

            if !success_or_not_found {
                let body =
                    delete.json::<Value>().await.unwrap_or_else(|_| json!({}));
                return Err(ApiError::Internal(eyre!(
                    "failed to delete Elasticsearch index `{index_name}`: {body}"
                )));
            }
        }

        self.ensure_index(&self.config.get_index_name("projects"))
            .await?;
        self.ensure_index(&self.config.get_index_name("projects_filtered"))
            .await?;
        Ok(())
    }

    async fn bulk_index_documents(
        &self,
        index_name: &str,
        docs: &[crate::search::UploadSearchProject],
    ) -> Result<(), ApiError> {
        if docs.is_empty() {
            return Ok(());
        }

        let mut body: Vec<JsonBody<Value>> = Vec::with_capacity(docs.len() * 2);
        for doc in docs {
            body.push(json!({"index": {"_id": doc.version_id}}).into());
            body.push(
                serde_json::to_value(doc)
                    .wrap_internal_err("failed to serialize document for Elasticsearch bulk index")?
                    .into(),
            );
        }

        let response = self
            .client
            .bulk(BulkParts::Index(index_name))
            .refresh(Refresh::WaitFor)
            .body(body)
            .send()
            .await
            .wrap_internal_err(
                "failed to request bulk index Elasticsearch documents",
            )?
            .error_for_status_code()
            .wrap_internal_err(
                "failed to bulk index Elasticsearch documents",
            )?;

        self.ensure_no_errors(response, "bulk index").await
    }

    async fn ensure_no_errors(
        &self,
        resp: Response,
        action: &str,
    ) -> Result<(), ApiError> {
        let body = resp
            .json::<Value>()
            .await
            .wrap_internal_err("failed to parse Elasticsearch response")?;
        if body.get("errors").and_then(Value::as_bool).unwrap_or(false) {
            return Err(ApiError::Internal(eyre!(
                "Elasticsearch `{action}` reported partial failures: {body}"
            )));
        }

        Ok(())
    }

    fn meili_like_filters(info: &SearchRequest) -> Option<Cow<'_, str>> {
        if let Some(filters) = info.new_filters.as_deref() {
            return Some(Cow::Borrowed(filters));
        }

        match (info.filters.as_deref(), info.version.as_deref()) {
            (Some(f), Some(v)) => Some(Cow::Owned(format!("({f}) AND ({v})"))),
            (Some(f), None) => Some(Cow::Borrowed(f)),
            (None, Some(v)) => Some(Cow::Borrowed(v)),
            (None, None) => None,
        }
    }
}

#[async_trait]
impl SearchBackend for Elasticsearch {
    async fn search_for_project(
        &self,
        info: &SearchRequest,
    ) -> Result<SearchResults, ApiError> {
        let offset = info
            .offset
            .as_deref()
            .unwrap_or("0")
            .parse::<usize>()
            .wrap_request_err("invalid offset")?;
        let limit = info
            .limit
            .as_deref()
            .unwrap_or("10")
            .parse::<usize>()
            .wrap_request_err("invalid limit")?
            .min(100);
        let hits_per_page = if limit == 0 { 1 } else { limit };
        let page = offset / hits_per_page + 1;
        let index = info.index.as_deref().unwrap_or("relevance");
        let (index_name, sort) = self.get_sort_index(index)?;

        let mut must = Vec::new();
        let query_text = info.query.as_deref().unwrap_or_default().trim();
        if query_text.is_empty() {
            must.push(json!({"match_all": {}}));
        } else {
            must.push(json!({
                "multi_match": {
                    "query": query_text,
                    "type": "bool_prefix",
                    "fields": [
                        "name^8",
                        "name._2gram^8",
                        "name._3gram^8",
                        "slug^8",
                        "slug._2gram^8",
                        "slug._3gram^8",
                        "author^2",
                        "author._2gram^2",
                        "author._3gram^2",
                        "summary^3",
                        "summary._2gram^3",
                        "summary._3gram^3"
                    ]
                }
            }));
        }

        let mut filter = Self::facets_filter_clauses(info.facets.as_deref())?;
        if let Some(filter_string) = Self::meili_like_filters(info)
            && !filter_string.trim().is_empty()
        {
            filter.push(json!({
                "query_string": {
                    "query": filter_string,
                    "default_operator": "AND",
                    "lenient": true
                }
            }));
        }

        let response = self
            .client
            .search(SearchParts::Index(&[index_name.as_str()]))
            .from(offset as i64)
            .size(hits_per_page as i64)
            .track_total_hits(true)
            .body(json!({
                "query": {
                    "bool": {
                        "must": must,
                        "filter": filter
                    }
                },
                "sort": sort
            }))
            .send()
            .await
            .wrap_internal_err("failed to execute Elasticsearch search")?;

        let response_body = response.json::<Value>().await.wrap_internal_err(
            "failed to parse Elasticsearch search response",
        )?;

        let hits = response_body["hits"]["hits"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|hit| hit.get("_source").cloned())
            .map(|source| -> Result<ResultSearchProject, ApiError> {
                let source =
                    serde_json::from_value::<UploadSearchProject>(source)
                        .map_err(|e| {
                            ApiError::Internal(eyre!(
                                "failed to deserialize Elasticsearch hit: {e}"
                            ))
                        })?;

                Ok(ResultSearchProject {
                    version_id: source.version_id,
                    project_id: source.project_id,
                    project_types: source.project_types,
                    slug: source.slug,
                    author: source.author,
                    name: source.name,
                    summary: source.summary,
                    categories: source.categories,
                    display_categories: source.display_categories,
                    downloads: source.downloads,
                    follows: source.follows,
                    icon_url: source.icon_url,
                    date_created: source.date_created.to_rfc3339(),
                    date_modified: source.date_modified.to_rfc3339(),
                    license: source.license,
                    gallery: source.gallery,
                    featured_gallery: source.featured_gallery,
                    color: source.color,
                    loaders: source.loaders,
                    project_loader_fields: source.project_loader_fields,
                    loader_fields: source.loader_fields,
                })
            })
            .collect::<Result<Vec<_>, ApiError>>()?;

        let total_hits = match &response_body["hits"]["total"] {
            Value::Number(n) => n.as_u64().unwrap_or_default() as usize,
            Value::Object(map) => {
                map.get("value").and_then(Value::as_u64).unwrap_or_default()
                    as usize
            }
            _ => 0,
        };

        Ok(SearchResults {
            hits,
            page,
            hits_per_page,
            total_hits,
        })
    }

    async fn index_projects(
        &self,
        ro_pool: PgPool,
        _redis: RedisPool,
    ) -> Result<(), ApiError> {
        self.reset_indexes().await?;

        let projects_index = self.config.get_index_name("projects");
        let filtered_index = self.config.get_index_name("projects_filtered");
        let mut cursor = 0_i64;

        loop {
            let (uploads, next_cursor) =
                index_local(&ro_pool, cursor, INDEX_CHUNK_SIZE)
                    .await
                    .wrap_internal_err("failed to index local")?;
            if uploads.is_empty() {
                break;
            }

            self.bulk_index_documents(&projects_index, &uploads).await?;
            self.bulk_index_documents(&filtered_index, &uploads).await?;
            cursor = next_cursor;
        }

        let indices = [projects_index.as_str(), filtered_index.as_str()];
        self.client
            .indices()
            .refresh(IndicesRefreshParts::Index(&indices))
            .send()
            .await
            .wrap_internal_err("failed to refresh Elasticsearch indexes")?;

        Ok(())
    }

    async fn remove_documents(
        &self,
        ids: &[VersionId],
    ) -> Result<(), ApiError> {
        if ids.is_empty() {
            return Ok(());
        }

        let ids_base62 =
            ids.iter().map(|id| to_base62(id.0)).collect::<Vec<_>>();
        for index_name in [
            self.config.get_index_name("projects"),
            self.config.get_index_name("projects_filtered"),
        ] {
            let response = self
                .client
                .delete_by_query(DeleteByQueryParts::Index(&[
                    index_name.as_str()
                ]))
                .refresh(true)
                .body(json!({
                    "query": {
                        "terms": {
                            "version_id": ids_base62
                        }
                    }
                }))
                .send()
                .await
                .wrap_internal_err(
                    "failed to delete Elasticsearch documents by query",
                )?;
            if !response.status_code().is_success() {
                let body = response
                    .json::<Value>()
                    .await
                    .unwrap_or_else(|_| json!({}));
                return Err(ApiError::Internal(eyre!(
                    "failed to delete documents from index `{index_name}`: {body}"
                )));
            }
        }

        Ok(())
    }

    async fn tasks(&self) -> Result<Value, ApiError> {
        #[derive(Serialize)]
        struct ElasticTask {
            uid: u64,
            status: &'static str,
            duration: Option<Duration>,
            enqueued_at: Option<u64>,
        }

        #[derive(Serialize)]
        struct TaskList {
            by_instance: HashMap<String, Vec<ElasticTask>>,
        }

        let response = self
            .client
            .tasks()
            .list()
            .detailed(true)
            .group_by(elasticsearch::params::GroupBy::Nodes)
            .send()
            .await
            .wrap_internal_err("failed to list Elasticsearch tasks")?;

        let body = response
            .json::<Value>()
            .await
            .wrap_internal_err("failed to parse Elasticsearch task response")?;

        let by_instance = body["nodes"]
            .as_object()
            .map(|nodes| {
                nodes
                    .iter()
                    .map(|(node_id, node_value)| {
                        let tasks = node_value["tasks"]
                            .as_object()
                            .map(|tasks| {
                                tasks
                                    .iter()
                                    .map(|(task_id, task)| {
                                        let uid = task_id
                                            .rsplit(':')
                                            .next()
                                            .and_then(|v| v.parse::<u64>().ok())
                                            .unwrap_or_default();
                                        let nanos =
                                            task["running_time_in_nanos"]
                                                .as_u64();
                                        ElasticTask {
                                            uid,
                                            status: "processing",
                                            duration: nanos
                                                .map(Duration::from_nanos),
                                            enqueued_at: task
                                                .get("start_time_in_millis")
                                                .and_then(Value::as_u64),
                                        }
                                    })
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or_default();
                        (node_id.clone(), tasks)
                    })
                    .collect::<HashMap<_, _>>()
            })
            .unwrap_or_default();

        serde_json::to_value(TaskList { by_instance })
            .wrap_internal_err("failed to serialize Elasticsearch tasks")
    }

    async fn tasks_cancel(
        &self,
        filter: &TasksCancelFilter,
    ) -> Result<(), ApiError> {
        match filter {
            TasksCancelFilter::All | TasksCancelFilter::AllEnqueued => {
                let response = self
                    .client
                    .tasks()
                    .cancel(TasksCancelParts::None)
                    .wait_for_completion(true)
                    .send()
                    .await
                    .wrap_internal_err(
                        "failed to cancel Elasticsearch tasks",
                    )?;
                if !response.status_code().is_success() {
                    let body = response
                        .json::<Value>()
                        .await
                        .unwrap_or_else(|_| json!({}));
                    return Err(ApiError::Internal(eyre!(
                        "failed to cancel Elasticsearch tasks: {body}"
                    )));
                }
            }
            TasksCancelFilter::Indexes { indexes } => {
                let response = self
                    .client
                    .tasks()
                    .list()
                    .detailed(true)
                    .group_by(elasticsearch::params::GroupBy::None)
                    .send()
                    .await
                    .wrap_internal_err("failed to list Elasticsearch tasks")?;

                let body = response.json::<Value>().await.wrap_internal_err(
                    "failed to parse Elasticsearch tasks list",
                )?;
                let tasks =
                    body["tasks"].as_object().cloned().unwrap_or_default();

                for (task_id, task) in tasks {
                    let description =
                        task["description"].as_str().unwrap_or_default();
                    if indexes.iter().any(|index| description.contains(index)) {
                        let response = self
                            .client
                            .tasks()
                            .cancel(TasksCancelParts::TaskId(&task_id))
                            .wait_for_completion(true)
                            .send()
                            .await
                            .wrap_internal_err(
                                "failed to cancel Elasticsearch task by id",
                            )?;
                        if !response.status_code().is_success() {
                            let body = response
                                .json::<Value>()
                                .await
                                .unwrap_or_else(|_| json!({}));
                            return Err(ApiError::Internal(eyre!(
                                "failed to cancel Elasticsearch task `{task_id}`: {body}"
                            )));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
