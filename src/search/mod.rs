use crate::models::error::ApiError;
use crate::models::projects::SearchRequest;
use actix_web::http::StatusCode;
use actix_web::web::HttpResponse;
use chrono::{DateTime, Utc};
use meilisearch_sdk::client::Client;
use meilisearch_sdk::document::Document;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::cmp::min;
use thiserror::Error;

pub mod indexing;

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("MeiliSearch Error: {0}")]
    MeiliSearchError(#[from] meilisearch_sdk::errors::Error),
    #[error("Error while serializing or deserializing JSON: {0}")]
    SerDeError(#[from] serde_json::Error),
    #[error("Error while parsing an integer: {0}")]
    IntParsingError(#[from] std::num::ParseIntError),
    #[error("Environment Error")]
    EnvError(#[from] dotenv::Error),
    #[error("Invalid index to sort by: {0}")]
    InvalidIndex(String),
}

impl actix_web::ResponseError for SearchError {
    fn status_code(&self) -> StatusCode {
        match self {
            SearchError::EnvError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            SearchError::MeiliSearchError(..) => StatusCode::BAD_REQUEST,
            SearchError::SerDeError(..) => StatusCode::BAD_REQUEST,
            SearchError::IntParsingError(..) => StatusCode::BAD_REQUEST,
            SearchError::InvalidIndex(..) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ApiError {
            error: match self {
                SearchError::EnvError(..) => "environment_error",
                SearchError::MeiliSearchError(..) => "meilisearch_error",
                SearchError::SerDeError(..) => "invalid_input",
                SearchError::IntParsingError(..) => "invalid_input",
                SearchError::InvalidIndex(..) => "invalid_input",
            },
            description: &self.to_string(),
        })
    }
}

#[derive(Clone)]
pub struct SearchConfig {
    pub address: String,
    pub key: String,
}

/// A project document used for uploading projects to meilisearch's indices.
/// This contains some extra data that is not returned by search results.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UploadSearchProject {
    pub project_id: String,
    pub project_type: String,
    pub slug: Option<String>,
    pub author: String,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    pub versions: Vec<String>,
    pub follows: i32,
    pub downloads: i32,
    pub icon_url: String,
    pub latest_version: String,
    pub license: String,
    pub client_side: String,
    pub server_side: String,

    /// RFC 3339 formatted creation date of the project
    pub date_created: DateTime<Utc>,
    /// Unix timestamp of the creation date of the project
    pub created_timestamp: i64,
    /// RFC 3339 formatted date/time of last major modification (update)
    pub date_modified: DateTime<Utc>,
    /// Unix timestamp of the last major modification
    pub modified_timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResults {
    pub hits: Vec<ResultSearchProject>,
    pub offset: usize,
    pub limit: usize,
    pub total_hits: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResultSearchProject {
    pub project_id: String,
    pub project_type: String,
    pub slug: Option<String>,
    pub author: String,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    // TODO: more efficient format for listing versions, without many repetitions
    pub versions: Vec<String>,
    pub downloads: i32,
    pub follows: i32,
    pub icon_url: String,
    /// RFC 3339 formatted creation date of the project
    pub date_created: String,
    /// RFC 3339 formatted modification date of the project
    pub date_modified: String,
    pub latest_version: String,
    pub license: String,
    pub client_side: String,
    pub server_side: String,
}

impl Document for UploadSearchProject {
    type UIDType = String;

    fn get_uid(&self) -> &Self::UIDType {
        &self.project_id
    }
}

impl Document for ResultSearchProject {
    type UIDType = String;

    fn get_uid(&self) -> &Self::UIDType {
        &self.project_id
    }
}

pub async fn search_for_project(
    info: &SearchRequest,
    config: &SearchConfig,
) -> Result<SearchResults, SearchError> {
    let client = Client::new(&*config.address, &*config.key);

    let filters: Cow<_> = match (info.filters.as_deref(), info.version.as_deref()) {
        (Some(f), Some(v)) => format!("({}) AND ({})", f, v).into(),
        (Some(f), None) => f.into(),
        (None, Some(v)) => v.into(),
        (None, None) => "".into(),
    };

    let offset = info.offset.as_deref().unwrap_or("0").parse()?;
    let index = info.index.as_deref().unwrap_or("relevance");
    let limit = info.limit.as_deref().unwrap_or("10").parse()?;

    let index = match index {
        "relevance" => "relevance_projects",
        "downloads" => "downloads_projects",
        "follows" => "follows_projects",
        "updated" => "updated_projects",
        "newest" => "newest_projects",
        "alphabetically" => "alphabetically_projects",
        i => return Err(SearchError::InvalidIndex(i.to_string())),
    };

    let meilisearch_index = client.get_index(index).await?;
    let mut query = meilisearch_index.search();

    query.with_limit(min(100, limit)).with_offset(offset);

    if let Some(search) = info.query.as_deref() {
        if !search.is_empty() {
            query.with_query(search);
        }
    }

    if !filters.is_empty() {
        query.with_filters(&filters);
    }

    // So the meilisearch sdk's lifetimes are... broken, to say the least
    // They are overspecified and almost always wrong, and would generally
    // just be better if they didn't specify them at all.

    // They also decided to have this take a &[&[&str]], which is impossible
    // to construct efficiently.  Instead it should take impl Iterator<Item=&[&str]>,
    // &[impl AsRef<[&str]>], or one of many other proper solutions to that issue.

    let why_meilisearch;
    let why_must_you_do_this;
    if let Some(facets) = &info.facets {
        why_meilisearch = serde_json::from_str::<Vec<Vec<&str>>>(facets)?;
        why_must_you_do_this = why_meilisearch
            .iter()
            .map(|v| v as &[_])
            .collect::<Vec<&[_]>>();
        query.with_facet_filters(&why_must_you_do_this);
    }

    let results = query.execute::<ResultSearchProject>().await?;

    Ok(SearchResults {
        hits: results.hits.into_iter().map(|r| r.result).collect(),
        offset: results.offset,
        limit: results.limit,
        total_hits: results.nb_hits,
    })
}
