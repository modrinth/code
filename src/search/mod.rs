use crate::models::error::ApiError;
use crate::models::projects::SearchRequest;
use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use chrono::{DateTime, Utc};
use meilisearch_sdk::client::Client;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::cmp::min;
use std::fmt::Write;
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

impl SearchConfig {
    pub fn make_client(&self) -> Client {
        Client::new(self.address.as_str(), self.key.as_str())
    }
}

/// A project document used for uploading projects to MeiliSearch's indices.
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
    pub display_categories: Vec<String>,
    pub versions: Vec<String>,
    pub follows: i32,
    pub downloads: i32,
    pub icon_url: String,
    pub latest_version: String,
    pub license: String,
    pub client_side: String,
    pub server_side: String,
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
    pub display_categories: Vec<String>,
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
    pub gallery: Vec<String>,
    pub featured_gallery: Option<String>,
    pub color: Option<u32>,
}

pub async fn search_for_project(
    info: &SearchRequest,
    config: &SearchConfig,
) -> Result<SearchResults, SearchError> {
    let client = Client::new(&*config.address, &*config.key);

    let offset = info.offset.as_deref().unwrap_or("0").parse()?;
    let index = info.index.as_deref().unwrap_or("relevance");
    let limit = info.limit.as_deref().unwrap_or("10").parse()?;

    let sort = match index {
        "relevance" => ("projects", ["downloads:desc"]),
        "downloads" => ("projects_filtered", ["downloads:desc"]),
        "follows" => ("projects", ["follows:desc"]),
        "updated" => ("projects", ["date_modified:desc"]),
        "newest" => ("projects", ["date_created:desc"]),
        i => return Err(SearchError::InvalidIndex(i.to_string())),
    };

    let meilisearch_index = client.get_index(sort.0).await?;

    let mut filter_string = String::new();

    let results = {
        let mut query = meilisearch_index.search();

        query
            .with_limit(min(100, limit))
            .with_offset(offset)
            .with_query(info.query.as_deref().unwrap_or_default())
            .with_sort(&sort.1);

        if let Some(new_filters) = info.new_filters.as_deref() {
            query.with_filter(new_filters);
        } else {
            let facets = if let Some(facets) = &info.facets {
                Some(serde_json::from_str::<Vec<Vec<&str>>>(facets)?)
            } else {
                None
            };

            let filters: Cow<_> =
                match (info.filters.as_deref(), info.version.as_deref()) {
                    (Some(f), Some(v)) => format!("({f}) AND ({v})").into(),
                    (Some(f), None) => f.into(),
                    (None, Some(v)) => v.into(),
                    (None, None) => "".into(),
                };

            if let Some(facets) = facets {
                filter_string.push('(');
                for (index, facet_list) in facets.iter().enumerate() {
                    filter_string.push('(');

                    for (facet_index, facet) in facet_list.iter().enumerate() {
                        filter_string.push_str(&facet.replace(':', " = "));

                        if facet_index != (facet_list.len() - 1) {
                            filter_string.push_str(" OR ")
                        }
                    }

                    filter_string.push(')');

                    if index != (facets.len() - 1) {
                        filter_string.push_str(" AND ")
                    }
                }
                filter_string.push(')');

                if !filters.is_empty() {
                    write!(filter_string, " AND ({filters})")?;
                }
            } else {
                filter_string.push_str(&filters);
            }

            if !filter_string.is_empty() {
                query.with_filter(&filter_string);
            }
        }

        query.execute::<ResultSearchProject>().await?
    };

    Ok(SearchResults {
        hits: results.hits.into_iter().map(|r| r.result).collect(),
        offset: results.offset.unwrap_or_default(),
        limit: results.limit.unwrap_or_default(),
        total_hits: results.estimated_total_hits.unwrap_or_default(),
    })
}
