use crate::models::error::ApiError;
use crate::models::projects::SearchRequest;
use actix_web::HttpResponse;
use actix_web::http::StatusCode;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use meilisearch_sdk::client::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sqlx::Row;
use thiserror::Error;

pub mod indexing;

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("MeiliSearch Error: {0}")]
    MeiliSearch(#[from] meilisearch_sdk::errors::Error),
    #[error("Error while serializing or deserializing JSON: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Error while parsing an integer: {0}")]
    IntParsing(#[from] std::num::ParseIntError),
    #[error("Error while formatting strings: {0}")]
    FormatError(#[from] std::fmt::Error),
    #[error("Database Error: {0}")]
    Sqlx(#[from] sqlx::error::Error),
    #[error("Database Error: {0}")]
    Database(#[from] crate::database::models::DatabaseError),
    #[error("Environment Error")]
    Env(#[from] dotenvy::Error),
    #[error("Invalid index to sort by: {0}")]
    InvalidIndex(String),
}

impl actix_web::ResponseError for SearchError {
    fn status_code(&self) -> StatusCode {
        match self {
            SearchError::Env(..) => StatusCode::INTERNAL_SERVER_ERROR,
            SearchError::MeiliSearch(..) => StatusCode::BAD_REQUEST,
            SearchError::Serde(..) => StatusCode::BAD_REQUEST,
            SearchError::IntParsing(..) => StatusCode::BAD_REQUEST,
            SearchError::InvalidIndex(..) => StatusCode::BAD_REQUEST,
            SearchError::FormatError(..) => StatusCode::BAD_REQUEST,
            SearchError::Sqlx(..) => StatusCode::INTERNAL_SERVER_ERROR,
            SearchError::Database(..) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ApiError {
            error: match self {
                SearchError::Env(..) => "environment_error",
                SearchError::MeiliSearch(..) => "meilisearch_error",
                SearchError::Serde(..) => "invalid_input",
                SearchError::IntParsing(..) => "invalid_input",
                SearchError::InvalidIndex(..) => "invalid_input",
                SearchError::FormatError(..) => "invalid_input",
                SearchError::Sqlx(..) => "database_error",
                SearchError::Database(..) => "database_error",
            },
            description: self.to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct SearchConfig {
    pub address: String,
    pub key: String,
    pub meta_namespace: String,
}

impl SearchConfig {
    // Panics if the environment variables are not set,
    // but these are already checked for on startup.
    pub fn new(meta_namespace: Option<String>) -> Self {
        let address =
            dotenvy::var("MEILISEARCH_ADDR").expect("MEILISEARCH_ADDR not set");
        let key =
            dotenvy::var("MEILISEARCH_KEY").expect("MEILISEARCH_KEY not set");

        Self {
            address,
            key,
            meta_namespace: meta_namespace.unwrap_or_default(),
        }
    }

    pub fn make_client(
        &self,
    ) -> Result<Client, meilisearch_sdk::errors::Error> {
        Client::new(self.address.as_str(), Some(self.key.as_str()))
    }

    // Next: true if we want the next index (we are preparing the next swap), false if we want the current index (searching)
    pub fn get_index_name(&self, index: &str, next: bool) -> String {
        let alt = if next { "_alt" } else { "" };
        format!("{}_{}_{}", self.meta_namespace, index, alt)
    }
}

/// A project document used for uploading projects to MeiliSearch's indices.
/// This contains some extra data that is not returned by search results.
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
    pub loader_fields: HashMap<String, Vec<serde_json::Value>>,
}

pub fn get_sort_index(
    config: &SearchConfig,
    index: &str,
) -> Result<(String, [&'static str; 1]), SearchError> {
    let projects_name = config.get_index_name("projects", false);
    let projects_filtered_name =
        config.get_index_name("projects_filtered", false);
    Ok(match index {
        "relevance" => (projects_name, ["downloads:desc"]),
        "downloads" => (projects_filtered_name, ["downloads:desc"]),
        "follows" => (projects_name, ["follows:desc"]),
        "updated" => (projects_name, ["date_modified:desc"]),
        "newest" => (projects_name, ["date_created:desc"]),
        i => return Err(SearchError::InvalidIndex(i.to_string())),
    })
}

pub async fn search_for_project(
    info: &SearchRequest,
    config: &SearchConfig,
) -> Result<SearchResults, SearchError> {
    use ariadne::ids::base62_impl::to_base62;
    use crate::database::{connect as db_connect, redis::RedisPool};
    use crate::database::models::{project_item::DBProject, ids::DBProjectId};
    use crate::models::projects::{ProjectStatus, from_duplicate_version_fields};

    let pool = db_connect().await?;
    let redis = RedisPool::new(Some(config.meta_namespace.clone()));

    let offset: usize = info.offset.as_deref().unwrap_or("0").parse()?;
    let index = info.index.as_deref().unwrap_or("relevance");
    let limit = info
        .limit
        .as_deref()
        .unwrap_or("10")
        .parse::<usize>()?
        .min(100);

    let order_by = match index {
        "relevance" | "downloads" => "downloads DESC",
        "follows" => "follows DESC",
        "updated" => "updated DESC",
        "newest" => "COALESCE(approved, published) DESC",
        i => return Err(SearchError::InvalidIndex(i.to_string())),
    };

    let statuses: Vec<String> = ProjectStatus::iterator()
        .filter(|x| x.is_searchable())
        .map(|x| x.to_string())
        .collect();

    let search_pattern = format!("%{}%", info.query.as_deref().unwrap_or("").to_lowercase());
    let sql = format!(
        "SELECT id FROM mods WHERE status = ANY($1) AND (LOWER(name) LIKE $2 OR LOWER(summary) LIKE $2 OR LOWER(COALESCE(slug,'')) LIKE $2) ORDER BY {order_by} OFFSET $3 LIMIT $4"
    );

    let ids: Vec<DBProjectId> = sqlx::query(&sql)
        .bind(&statuses)
        .bind(&search_pattern)
        .bind(offset as i64)
        .bind(limit as i64)
        .fetch_all(&pool)
        .await?
        .iter()
        .map(|row| DBProjectId(row.get::<i64, _>("id")))
        .collect();

    let count_sql = "SELECT COUNT(*) FROM mods WHERE status = ANY($1) AND (LOWER(name) LIKE $2 OR LOWER(summary) LIKE $2 OR LOWER(COALESCE(slug,'')) LIKE $2)";
    let total_hits: i64 = sqlx::query_scalar(count_sql)
        .bind(&statuses)
        .bind(&search_pattern)
        .fetch_one(&pool)
        .await?;

    let projects = DBProject::get_many_ids(&ids, &pool, &redis).await?;

    let ids_i64: Vec<i64> = ids.iter().map(|x| x.0).collect();
    let org_owners = sqlx::query!(
        "SELECT m.id mod_id, u.username FROM mods m INNER JOIN organizations o ON o.id = m.organization_id INNER JOIN team_members tm ON tm.is_owner = TRUE and tm.team_id = o.team_id INNER JOIN users u ON u.id = tm.user_id WHERE m.id = ANY($1)",
        &ids_i64
    )
    .fetch_all(&pool)
    .await?
    .into_iter()
    .map(|r| (DBProjectId(r.mod_id), r.username))
    .collect::<HashMap<_, _>>();

    let team_owners = sqlx::query!(
        "SELECT m.id mod_id, u.username FROM mods m INNER JOIN team_members tm ON tm.is_owner = TRUE and tm.team_id = m.team_id INNER JOIN users u ON u.id = tm.user_id WHERE m.id = ANY($1)",
        &ids_i64
    )
    .fetch_all(&pool)
    .await?
    .into_iter()
    .map(|r| (DBProjectId(r.mod_id), r.username))
    .collect::<HashMap<_, _>>();

    let hits = projects
        .into_iter()
        .map(|p| {
            let author = org_owners
                .get(&p.inner.id)
                .cloned()
                .or_else(|| team_owners.get(&p.inner.id).cloned())
                .unwrap_or_default();

            let version_id = p
                .versions
                .last()
                .map(|v| to_base62(v.0 as u64))
                .unwrap_or_default();

            let project_id = to_base62(p.inner.id.0 as u64);
            let mut categories = p.categories.clone();
            categories.extend(p.additional_categories.clone());
            let display_categories = p.categories.clone();

            let (featured_gallery, gallery) = {
                let mut featured = None;
                let mut gallery = Vec::new();
                for item in &p.gallery_items {
                    if item.featured && featured.is_none() {
                        featured = Some(item.image_url.clone());
                    } else {
                        gallery.push(item.image_url.clone());
                    }
                }
                (featured, gallery)
            };

            let project_loader_fields = from_duplicate_version_fields(p.aggregate_version_fields.clone());

            ResultSearchProject {
                version_id,
                project_id,
                project_types: p.project_types.clone(),
                slug: p.inner.slug.clone(),
                author,
                name: p.inner.name.clone(),
                summary: p.inner.summary.clone(),
                categories,
                display_categories,
                downloads: p.inner.downloads,
                follows: p.inner.follows,
                icon_url: p.inner.icon_url.clone(),
                date_created: p
                    .inner
                    .approved
                    .unwrap_or(p.inner.published)
                    .to_rfc3339(),
                date_modified: p.inner.updated.to_rfc3339(),
                license: p.inner.license.split_whitespace().next().unwrap_or(&p.inner.license).to_string(),
                gallery,
                featured_gallery,
                color: p.inner.color,
                loaders: p.inner.loaders.clone(),
                project_loader_fields: project_loader_fields.clone(),
                loader_fields: project_loader_fields,
            }
        })
        .collect::<Vec<_>>();

    Ok(SearchResults {
        hits,
        page: offset / limit + 1,
        hits_per_page: limit,
        total_hits: total_hits as usize,
    })
}
