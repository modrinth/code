use crate::models::error::ApiError;
use crate::models::mods::SearchRequest;
use actix_web::http::StatusCode;
use actix_web::web::HttpResponse;
use meilisearch_sdk::client::Client;
use meilisearch_sdk::document::Document;
use meilisearch_sdk::search::Query;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod indexing;

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("Error while connecting to the MeiliSearch database")]
    IndexDBError(meilisearch_sdk::errors::Error),
    #[error("Error while serializing or deserializing JSON: {0}")]
    SerDeError(#[from] serde_json::Error),
    #[error("Error while parsing an integer: {0}")]
    IntParsingError(#[from] std::num::ParseIntError),
    #[error("Environment Error")]
    EnvError(#[from] dotenv::Error),
}

impl actix_web::ResponseError for SearchError {
    fn status_code(&self) -> StatusCode {
        match self {
            SearchError::EnvError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            SearchError::IndexDBError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            SearchError::SerDeError(..) => StatusCode::BAD_REQUEST,
            SearchError::IntParsingError(..) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ApiError {
            error: match self {
                SearchError::EnvError(..) => "environment_error",
                SearchError::IndexDBError(..) => "indexdb_error",
                SearchError::SerDeError(..) => "invalid_input",
                SearchError::IntParsingError(..) => "invalid_input",
            },
            description: &self.to_string(),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchMod {
    pub mod_id: i64,
    pub author: String,
    pub title: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub versions: Vec<String>,
    pub downloads: i32,
    pub page_url: String,
    pub icon_url: String,
    pub author_url: String,
    pub date_created: String,
    pub created: i64,
    pub date_modified: String,
    pub updated: i64,
    pub latest_version: String,
    pub empty: String,
}

impl Document for SearchMod {
    type UIDType = i64;

    fn get_uid(&self) -> &Self::UIDType {
        &self.mod_id
    }
}

pub fn search_for_mod(info: &SearchRequest) -> Result<Vec<SearchMod>, SearchError> {
    use std::borrow::Cow;
    let address = &*dotenv::var("MEILISEARCH_ADDR")?;
    let client = Client::new(address, "");

    let filters: Cow<_> = match (info.filters.as_deref(), info.version.as_deref()) {
        (Some(f), Some(v)) => format!("({}) AND ({})", f, v).into(),
        (Some(f), None) => f.into(),
        (None, Some(v)) => v.into(),
        (None, None) => "".into(),
    };

    let offset = info.offset.as_deref().unwrap_or("0").parse()?;
    let index = info.index.as_deref().unwrap_or("relevance");
    let search_query: &str = info
        .query
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("{}{}{}");

    let mut query = Query::new(search_query).with_limit(10).with_offset(offset);

    if !filters.is_empty() {
        query = query.with_filters(&filters);
    }

    Ok(client
        .get_index(format!("{}_mods", index).as_ref())
        .map_err(SearchError::IndexDBError)?
        .search::<SearchMod>(&query)
        .map_err(SearchError::IndexDBError)?
        .hits)
}
