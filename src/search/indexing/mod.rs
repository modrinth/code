/// This module is used for the indexing from any source.
pub mod curseforge_import;
pub mod local_import;
pub mod queue;

use crate::search::UploadSearchMod;
use curseforge_import::index_curseforge;
use local_import::index_local;
use meilisearch_sdk::client::Client;
use meilisearch_sdk::indexes::Index;
use meilisearch_sdk::settings::Settings;
use sqlx::postgres::PgPool;
use std::collections::{HashMap, VecDeque};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IndexingError {
    #[error("Error while connecting to the MeiliSearch database")]
    IndexDBError(#[from] meilisearch_sdk::errors::Error),
    #[error("Error while importing mods from CurseForge")]
    CurseforgeImportError(reqwest::Error),
    #[error("Error while serializing or deserializing JSON: {0}")]
    SerDeError(#[from] serde_json::Error),
    #[error("Error while parsing a timestamp: {0}")]
    ParseDateError(#[from] chrono::format::ParseError),
    #[error("Database Error: {0}")]
    DatabaseError(#[from] sqlx::error::Error),
    #[error("Environment Error")]
    EnvError(#[from] dotenv::Error),
}

// The chunk size for adding mods to the indexing database. If the request size
// is too large (>10MiB) then the request fails with an error.  This chunk size
// assumes a max average size of 1KiB per mod to avoid this cap.
const MEILISEARCH_CHUNK_SIZE: usize = 10000;

#[derive(Debug)]
pub struct IndexingSettings {
    pub index_external: bool,
    pub index_local: bool,
}

impl IndexingSettings {
    pub fn from_env() -> Self {
        let index_local = true;
        let index_external = dotenv::var("INDEX_CURSEFORGE")
            .ok()
            .and_then(|b| b.parse::<bool>().ok())
            .unwrap_or(false);

        Self {
            index_external,
            index_local,
        }
    }
}

pub async fn index_mods(pool: PgPool, settings: IndexingSettings) -> Result<(), IndexingError> {
    let mut docs_to_add: Vec<UploadSearchMod> = vec![];

    if settings.index_local {
        docs_to_add.append(&mut index_local(pool.clone()).await?);
    }
    if settings.index_external {
        let end_index = dotenv::var("MAX_CURSEFORGE_ID")
                .ok()
                .map(|i| i.parse().unwrap())
                .unwrap_or(450_000);

        docs_to_add.append(&mut index_curseforge(1, end_index).await?);
    }

    // Write Indices

    add_mods(docs_to_add).await?;

    Ok(())
}

pub async fn reset_indices() -> Result<(), IndexingError> {
    let address = &*dotenv::var("MEILISEARCH_ADDR")?;
    let client = Client::new(address, "");

    client.delete_index("relevance_mods").await?;
    client.delete_index("downloads_mods").await?;
    client.delete_index("updated_mods").await?;
    client.delete_index("newest_mods").await?;
    Ok(())
}

pub async fn reconfigure_indices() -> Result<(), IndexingError> {
    let address = &*dotenv::var("MEILISEARCH_ADDR")?;
    let client = Client::new(address, "");

    // Relevance Index
    update_index(&client, "relevance_mods", {
        let mut relevance_rules = default_rules();
        relevance_rules.push_back("desc(downloads)".to_string());
        relevance_rules.into()
    })
    .await?;

    // Downloads Index
    update_index(&client, "downloads_mods", {
        let mut downloads_rules = default_rules();
        downloads_rules.push_front("desc(downloads)".to_string());
        downloads_rules.into()
    })
    .await?;

    // Updated Index
    update_index(&client, "updated_mods", {
        let mut updated_rules = default_rules();
        updated_rules.push_front("desc(modified_timestamp)".to_string());
        updated_rules.into()
    })
    .await?;

    // Created Index
    update_index(&client, "newest_mods", {
        let mut newest_rules = default_rules();
        newest_rules.push_front("desc(created_timestamp)".to_string());
        newest_rules.into()
    })
    .await?;

    Ok(())
}

async fn update_index<'a>(
    client: &'a Client<'a>,
    name: &'a str,
    rules: Vec<String>,
) -> Result<Index<'a>, IndexingError> {
    let index = match client.get_index(name).await {
        Ok(index) => index,
        Err(meilisearch_sdk::errors::Error::MeiliSearchError {
            error_code: meilisearch_sdk::errors::ErrorCode::IndexNotFound,
            ..
        }) => client.create_index(name, Some("mod_id")).await?,
        Err(e) => {
            return Err(IndexingError::IndexDBError(e));
        }
    };
    index
        .set_settings(&default_settings().with_ranking_rules(rules))
        .await?;
    Ok(index)
}

async fn create_index<'a>(
    client: &'a Client<'a>,
    name: &'a str,
    rules: impl FnOnce() -> Vec<String>,
) -> Result<Index<'a>, IndexingError> {
    match client.get_index(name).await {
        // TODO: update index settings on startup (or delete old indices on startup)
        Ok(index) => Ok(index),
        Err(meilisearch_sdk::errors::Error::MeiliSearchError {
            error_code: meilisearch_sdk::errors::ErrorCode::IndexNotFound,
            ..
        }) => {
            // Only create index and set settings if the index doesn't already exist
            let index = client.create_index(name, Some("mod_id")).await?;

            index
                .set_settings(&default_settings().with_ranking_rules(rules()))
                .await?;

            Ok(index)
        }
        Err(e) => {
            log::warn!("Unhandled error while creating index: {}", e);
            Err(IndexingError::IndexDBError(e))
        }
    }
}

async fn add_to_index(index: Index<'_>, mods: &[UploadSearchMod]) -> Result<(), IndexingError> {
    for chunk in mods.chunks(MEILISEARCH_CHUNK_SIZE) {
        index.add_documents(chunk, Some("mod_id")).await?;
    }
    Ok(())
}

pub async fn add_mods(mods: Vec<UploadSearchMod>) -> Result<(), IndexingError> {
    let address = &*dotenv::var("MEILISEARCH_ADDR")?;
    let client = Client::new(address, "");

    // Relevance Index
    let relevance_index = create_index(&client, "relevance_mods", || {
        let mut relevance_rules = default_rules();
        relevance_rules.push_back("desc(downloads)".to_string());
        relevance_rules.into()
    })
    .await?;
    add_to_index(relevance_index, &mods).await?;

    // Downloads Index
    let downloads_index = create_index(&client, "downloads_mods", || {
        let mut downloads_rules = default_rules();
        downloads_rules.push_front("desc(downloads)".to_string());
        downloads_rules.into()
    })
    .await?;
    add_to_index(downloads_index, &mods).await?;

    // Updated Index
    let updated_index = create_index(&client, "updated_mods", || {
        let mut updated_rules = default_rules();
        updated_rules.push_front("desc(modified_timestamp)".to_string());
        updated_rules.into()
    })
    .await?;
    add_to_index(updated_index, &mods).await?;

    // Created Index
    let newest_index = create_index(&client, "newest_mods", || {
        let mut newest_rules = default_rules();
        newest_rules.push_front("desc(created_timestamp)".to_string());
        newest_rules.into()
    })
    .await?;
    add_to_index(newest_index, &mods).await?;

    Ok(())
}

//region Utils
fn default_rules() -> VecDeque<String> {
    vec![
        "typo".to_string(),
        "words".to_string(),
        "proximity".to_string(),
        "attribute".to_string(),
        "wordsPosition".to_string(),
        "exactness".to_string(),
    ]
    .into()
}

fn default_settings() -> Settings {
    let displayed_attributes = vec![
        "mod_id".to_string(),
        "author".to_string(),
        "title".to_string(),
        "description".to_string(),
        "categories".to_string(),
        "versions".to_string(),
        "downloads".to_string(),
        "page_url".to_string(),
        "icon_url".to_string(),
        "author_url".to_string(),
        "date_created".to_string(),
        "date_modified".to_string(),
        "latest_version".to_string(),
        "host".to_string(),
    ];

    let searchable_attributes = vec![
        "title".to_string(),
        "description".to_string(),
        "categories".to_string(),
        "versions".to_string(),
        "author".to_string(),
        "empty".to_string(),
    ];

    Settings::new()
        .with_displayed_attributes(displayed_attributes)
        .with_searchable_attributes(searchable_attributes)
        .with_stop_words(vec![])
        .with_synonyms(HashMap::new())
        .with_attributes_for_faceting(vec![String::from("categories"), String::from("host"), String::from("versions")])
}

//endregion
