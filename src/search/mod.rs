use crate::models::error::ApiError;
use crate::models::mods::SearchRequest;
use actix_web::http::StatusCode;
use actix_web::web::HttpResponse;
use chrono::{DateTime, Utc};
use meilisearch_sdk::client::Client;
use meilisearch_sdk::document::Document;
use meilisearch_sdk::search::Query;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::cmp::min;
use thiserror::Error;

pub mod indexing;

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("Error while connecting to the MeiliSearch database: {0}")]
    IndexDBError(#[from] meilisearch_sdk::errors::Error),
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
            SearchError::IndexDBError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            SearchError::SerDeError(..) => StatusCode::BAD_REQUEST,
            SearchError::IntParsingError(..) => StatusCode::BAD_REQUEST,
            SearchError::InvalidIndex(..) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ApiError {
            error: match self {
                SearchError::EnvError(..) => "environment_error",
                SearchError::IndexDBError(..) => "indexdb_error",
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

/// A mod document used for uploading mods to meilisearch's indices.
/// This contains some extra data that is not returned by search results.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UploadSearchMod {
    pub mod_id: String,
    pub author: String,
    pub title: String,
    pub description: String,
    pub categories: Vec<Cow<'static, str>>,
    pub versions: Vec<String>,
    pub downloads: i32,
    pub page_url: String,
    pub icon_url: String,
    pub author_url: String,
    pub latest_version: Cow<'static, str>,

    /// RFC 3339 formatted creation date of the mod
    pub date_created: DateTime<Utc>,
    /// Unix timestamp of the creation date of the mod
    pub created_timestamp: i64,
    /// RFC 3339 formatted date/time of last major modification (update)
    pub date_modified: DateTime<Utc>,
    /// Unix timestamp of the last major modification
    pub modified_timestamp: i64,

    pub host: Cow<'static, str>,

    /// Must be "{}{}{}", a hack until meilisearch supports searches
    /// with empty queries (https://github.com/meilisearch/MeiliSearch/issues/729)
    // This is a Cow to prevent unnecessary allocations for a static
    // string
    pub empty: Cow<'static, str>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResults {
    pub hits: Vec<ResultSearchMod>,
    pub offset: usize,
    pub limit: usize,
    pub total_hits: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResultSearchMod {
    pub mod_id: String,
    pub author: String,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    // TODO: more efficient format for listing versions, without many repetitions
    pub versions: Vec<String>,
    pub downloads: i32,
    pub page_url: String,
    pub icon_url: String,
    pub author_url: String,
    /// RFC 3339 formatted creation date of the mod
    pub date_created: String,
    /// RFC 3339 formatted modification date of the mod
    pub date_modified: String,
    pub latest_version: String,

    /// The host of the mod: Either `modrinth` or `curseforge`
    pub host: String,
}

impl Document for UploadSearchMod {
    type UIDType = String;

    fn get_uid(&self) -> &Self::UIDType {
        &self.mod_id
    }
}

impl Document for ResultSearchMod {
    type UIDType = String;

    fn get_uid(&self) -> &Self::UIDType {
        &self.mod_id
    }
}

pub async fn search_for_mod(
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
    let search_query: &str = info
        .query
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("{}{}{}");

    let mut query = Query::new(search_query)
        .with_limit(min(100, limit))
        .with_offset(offset);

    if !filters.is_empty() {
        query = query.with_filters(&filters);
    }
    if let Some(facets) = &info.facets {
        let facets = serde_json::from_str::<Vec<Vec<&str>>>(facets)?;
        query = query.with_facet_filters(facets);
    }

    let index = match index {
        "relevance" => "relevance_mods",
        "downloads" => "downloads_mods",
        "updated" => "updated_mods",
        "newest" => "newest_mods",
        i => return Err(SearchError::InvalidIndex(i.to_string())),
    };

    let results = client
        .get_index(index)
        .await?
        .search::<ResultSearchMod>(&query)
        .await?;

    Ok(SearchResults {
        hits: results.hits,
        offset: results.offset,
        limit: results.limit,
        total_hits: results.nb_hits,
    })
}
