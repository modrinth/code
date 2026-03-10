use crate::database::redis::RedisPool;
use crate::models::exp;
use crate::models::exp::minecraft::JavaServerPing;
use crate::models::ids::{ProjectId, VersionId};
use crate::queue::server_ping;
use crate::routes::ApiError;
use crate::{database::PgPool, env::ENV};
use ariadne::ids::base62_impl::parse_base62;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, str::FromStr};
use thiserror::Error;
use utoipa::ToSchema;

pub mod backend;
pub mod indexing;

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchQuery {
    pub query: Option<String>,
    pub offset: Option<String>,
    pub index: Option<String>,
    pub limit: Option<String>,

    pub new_filters: Option<String>,

    pub facets: Option<String>,
    pub filters: Option<String>,
    pub version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchRequest {
    pub query: Option<String>,
    pub offset: Option<String>,
    pub index: Option<String>,
    pub limit: Option<String>,
    #[serde(default)]
    pub show_metadata: bool,
    #[serde(default)]
    pub elasticsearch_config: backend::elasticsearch::RequestConfig,

    pub new_filters: Option<String>,

    pub facets: Option<String>,
    pub filters: Option<String>,
    pub version: Option<String>,
}

impl From<SearchQuery> for SearchRequest {
    fn from(query: SearchQuery) -> Self {
        Self {
            query: query.query,
            offset: query.offset,
            index: query.index,
            limit: query.limit,
            show_metadata: false,
            elasticsearch_config:
                backend::elasticsearch::RequestConfig::default(),
            new_filters: query.new_filters,
            facets: query.facets,
            filters: query.filters,
            version: query.version,
        }
    }
}

#[async_trait]
pub trait SearchBackend: Send + Sync {
    async fn search_for_project(
        &self,
        info: &SearchRequest,
        redis: &RedisPool,
    ) -> Result<SearchResults, ApiError> {
        let mut results = self.search_for_project_raw(info).await?;
        hydrate_search_results(&mut results.hits, redis)
            .await
            .map_err(ApiError::Internal)?;
        Ok(results)
    }

    async fn search_for_project_raw(
        &self,
        info: &SearchRequest,
    ) -> Result<SearchResults, ApiError>;

    async fn index_projects(
        &self,
        ro_pool: PgPool,
        redis: RedisPool,
    ) -> eyre::Result<()>;

    async fn remove_documents(&self, ids: &[VersionId]) -> eyre::Result<()>;

    async fn tasks(&self) -> eyre::Result<Value>;

    async fn tasks_cancel(
        &self,
        filter: &TasksCancelFilter,
    ) -> eyre::Result<()>;
}

async fn hydrate_search_results(
    hits: &mut [ResultSearchProject],
    redis_pool: &RedisPool,
) -> eyre::Result<()> {
    // Minecraft Java servers should fetch the latest player count that we have
    // from Redis, rather than the (pretty stale) data from search backend
    // TODO: this block should be made generic over the component type,
    // for now we can hardcode MC java servers tho

    let project_ids = hits
        .iter()
        .filter(|hit| hit.components.minecraft_java_server.is_some())
        .filter_map(|hit| parse_base62(&hit.project_id).ok().map(ProjectId))
        .collect::<Vec<_>>();

    let pings_by_project_id = if project_ids.is_empty() {
        HashMap::new()
    } else {
        let mut redis = redis_pool.connect().await?;
        let ping_results = redis
            .get_many_deserialized_from_json::<JavaServerPing>(
                server_ping::REDIS_NAMESPACE,
                &project_ids
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
            )
            .await?;

        ping_results
            .into_iter()
            .enumerate()
            .filter_map(|(idx, ping)| ping.map(|ping| (project_ids[idx], ping)))
            .collect::<HashMap<_, _>>()
    };

    for hit in hits {
        let Some(java_server) = hit.components.minecraft_java_server.as_mut()
        else {
            continue;
        };
        if let Ok(project_id) = parse_base62(&hit.project_id).map(ProjectId) {
            java_server.ping = pings_by_project_id.get(&project_id).cloned();
        } else {
            java_server.ping = None;
        }
    }

    Ok(())
}

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TasksCancelFilter {
    All,
    AllEnqueued,
    Indexes { indexes: Vec<String> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchBackendKind {
    Meilisearch,
    Elasticsearch,
}

#[derive(Debug, Error)]
#[error("invalid search backend kind")]
pub struct InvalidSearchBackendKind;

impl FromStr for SearchBackendKind {
    type Err = InvalidSearchBackendKind;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "meilisearch" => SearchBackendKind::Meilisearch,
            "elasticsearch" => SearchBackendKind::Elasticsearch,
            _ => return Err(InvalidSearchBackendKind),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UploadSearchProject {
    pub version_id: String,
    pub project_id: String,
    //
    pub project_types: Vec<String>,
    pub slug: Option<String>,
    pub author: String,
    pub name: String,
    pub summary: String,
    pub categories: Vec<String>,
    pub display_categories: Vec<String>,
    pub follows: i32,
    pub downloads: i32,
    pub icon_url: Option<String>,
    pub license: String,
    pub gallery: Vec<String>,
    pub featured_gallery: Option<String>,
    /// RFC 3339 formatted creation date of the project
    pub date_created: DateTime<Utc>,
    /// Unix timestamp of the creation date of the project
    pub created_timestamp: i64,
    /// RFC 3339 formatted date/time of last major modification (update)
    pub date_modified: DateTime<Utc>,
    /// Unix timestamp of the last major modification
    pub modified_timestamp: i64,
    /// Unix timestamp of the publication date of the version
    pub version_published_timestamp: i64,
    pub open_source: bool,
    pub color: Option<u32>,

    // Hidden fields to get the Project model out of the search results.
    pub loaders: Vec<String>, // Search uses loaders as categories- this is purely for the Project model.
    pub project_loader_fields: HashMap<String, Vec<serde_json::Value>>, // Aggregation of loader_fields from all versions of the project, allowing for reconstruction of the Project model.

    #[serde(flatten)]
    pub components: exp::ProjectQuery,
    #[serde(flatten)]
    pub loader_fields: HashMap<String, Vec<serde_json::Value>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResults {
    pub hits: Vec<ResultSearchProject>,
    pub page: usize,
    pub hits_per_page: usize,
    pub total_hits: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResultSearchProject {
    pub version_id: String,
    pub project_id: String,
    pub project_types: Vec<String>,
    pub slug: Option<String>,
    pub author: String,
    pub name: String,
    pub summary: String,
    pub categories: Vec<String>,
    pub display_categories: Vec<String>,
    pub downloads: i32,
    pub follows: i32,
    pub icon_url: Option<String>,
    /// RFC 3339 formatted creation date of the project
    pub date_created: String,
    /// RFC 3339 formatted modification date of the project
    pub date_modified: String,
    pub license: String,
    pub gallery: Vec<String>,
    pub featured_gallery: Option<String>,
    pub color: Option<u32>,

    // Hidden fields to get the Project model out of the search results.
    pub loaders: Vec<String>, // Search uses loaders as categories- this is purely for the Project model.
    pub project_loader_fields: HashMap<String, Vec<serde_json::Value>>, // Aggregation of loader_fields from all versions of the project, allowing for reconstruction of the Project model.

    #[serde(flatten)]
    pub components: exp::ProjectQuery,
    #[serde(flatten)]
    pub loader_fields: HashMap<String, Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_metadata: Option<Value>,
}

impl From<UploadSearchProject> for ResultSearchProject {
    fn from(source: UploadSearchProject) -> Self {
        Self {
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
            components: source.components,
            loader_fields: source.loader_fields,
            search_metadata: None,
        }
    }
}

pub fn backend(meta_namespace: Option<String>) -> Box<dyn SearchBackend> {
    match ENV.SEARCH_BACKEND {
        SearchBackendKind::Meilisearch => {
            let config = backend::MeilisearchConfig::new(meta_namespace);
            Box::new(backend::Meilisearch::new(config))
        }
        SearchBackendKind::Elasticsearch => {
            Box::new(backend::Elasticsearch::new(meta_namespace).unwrap())
        }
    }
}
