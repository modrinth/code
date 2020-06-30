use crate::database::DatabaseError;
use crate::models::mods::SearchRequest;
use meilisearch_sdk::client::Client;
use meilisearch_sdk::document::Document;
use meilisearch_sdk::search::Query;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod indexing;

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("Error while connection to the MeiliSearch database")]
    IndexDBError(),
    #[error("Error while connecting to the local server")]
    LocalDatabaseError(#[from] mongodb::error::Error),
    #[error("Error while accessing the data from remote")]
    RemoteWebsiteError(#[from] reqwest::Error),
    #[error("Error while serializing or deserializing JSON")]
    SerDeError(#[from] serde_json::Error),
    #[error("Error while parsing float")]
    FloatParsingError(#[from] std::num::ParseFloatError),
    #[error("Error while parsing float")]
    IntParsingError(#[from] std::num::ParseIntError),
    #[error("Error while parsing BSON")]
    DatabaseError(#[from] DatabaseError),
    #[error("Environment Error")]
    EnvError(#[from] dotenv::Error),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchMod {
    pub mod_id: i32,
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
    type UIDType = i32;

    fn get_uid(&self) -> &Self::UIDType {
        &self.mod_id
    }
}

pub fn search_for_mod(info: &SearchRequest) -> Result<Vec<SearchMod>, SearchError> {
    let address = &*dotenv::var("MEILISEARCH_ADDR")?;
    let client = Client::new(address, "");

    let search_query: &str;
    let mut filters = String::new();
    let mut offset = 0;
    let mut index = "relevance";

    match info.query.as_ref() {
        Some(q) => search_query = q,
        None => search_query = "{}{}{}",
    }

    if let Some(f) = info.filters.as_ref() {
        filters = f.clone();
    }

    if let Some(v) = info.version.as_ref() {
        if filters.is_empty() {
            filters = v.clone();
        } else {
            filters = format!("({}) AND ({})", filters, v);
        }
    }

    if let Some(o) = info.offset.as_ref() {
        offset = o.parse().unwrap();
    }

    if let Some(s) = info.index.as_ref() {
        index = s;
    }

    let mut query = Query::new(search_query).with_limit(10).with_offset(offset);

    if !filters.is_empty() {
        query = query.with_filters(&filters);
    }

    Ok(client
        .get_index(format!("{}_mods", index).as_ref())
        .unwrap()
        .search::<SearchMod>(&query)
        .unwrap()
        .hits)
}
