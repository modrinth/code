use crate::database::redis::RedisPool;
use crate::models::exp;
use crate::models::ids::VersionId;
use crate::models::projects::SearchRequest;
use crate::{database::PgPool, env::ENV};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, str::FromStr};
use thiserror::Error;
use utoipa::ToSchema;

pub mod backend;
pub mod indexing;

#[async_trait]
pub trait SearchBackend: Send + Sync {
    async fn search_for_project(
        &self,
        info: &SearchRequest,
    ) -> eyre::Result<SearchResults>;

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

// todo: indexes
/*
"relevance" => (
    projects_name,
    &[
        "minecraft_java_server.verified_plays_2w:desc",
        "minecraft_java_server.ping.data.players_online:desc",
        "downloads:desc",
    ],
),
"downloads" => (projects_filtered_name, &["downloads:desc"]),
"follows" => (projects_name, &["follows:desc"]),
"updated" | "date_modified" => (projects_name, &["date_modified:desc"]),
"newest" | "date_created" => (projects_name, &["date_created:desc"]),
"minecraft_java_server.verified_plays_2w" => (
    projects_name,
    &["minecraft_java_server.verified_plays_2w:desc"],
),
"minecraft_java_server.ping.data.players_online" => (
    projects_name,
    &["minecraft_java_server.ping.data.players_online:desc"],
),
i => return Err(SearchError::InvalidIndex(i.to_string())),
 */

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

#[cfg(test)]
mod tests {
    use super::normalize_filter_aliases;

    #[test]
    fn normalizes_component_filter_aliases() {
        assert_eq!(
            normalize_filter_aliases(
                "components.minecraft_java_server.content = vanilla AND components.minecraft_server.country = US"
            ),
            "minecraft_java_server.content.kind = vanilla AND minecraft_server.country = US"
        );
    }
}
