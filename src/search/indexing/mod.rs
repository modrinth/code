/// This module is used for the indexing from any source.
pub mod curseforge_import;
pub mod local_import;

use crate::search::indexing::curseforge_import::index_curseforge;
use crate::search::indexing::local_import::index_local;
use crate::search::SearchMod;
use meilisearch_sdk::client::Client;
use meilisearch_sdk::settings::Settings;
use std::collections::{HashMap, VecDeque};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IndexingError {
    #[error("Error while connecting to the MeiliSearch database")]
    IndexDBError(meilisearch_sdk::errors::Error),
    #[error("Error while importing mods from CurseForge")]
    CurseforgeImportError(reqwest::Error),
    #[error("Error while serializing or deserializing JSON: {0}")]
    SerDeError(#[from] serde_json::Error),
    #[error("Error while parsing a timestamp: {0}")]
    ParseDateError(#[from] chrono::format::ParseError),
    #[error("Database Error: {0}")]
    DatabaseError(#[from] crate::database::DatabaseError),
    #[error("Environment Error")]
    EnvError(#[from] dotenv::Error),
}

// The chunk size for adding mods to the indexing database. If the request size
// is too large (>10MiB) then the request fails with an error.  This chunk size
// assumes a max average size of 1KiB per mod to avoid this cap.
const MEILISEARCH_CHUNK_SIZE: usize = 10000;

pub async fn index_mods(db: mongodb::Client) -> Result<(), IndexingError> {
    // Check if the index exists
    let address = &*dotenv::var("MEILISEARCH_ADDR")?;
    let client = Client::new(address, "");

    let mut docs_to_add: Vec<SearchMod> = vec![];

    docs_to_add.append(&mut index_local(db.clone()).await?);
    if dotenv::var("INDEX_CURSEFORGE")?
        .parse()
        .expect("`INDEX_CURSEFORGE` is not a boolean.")
    {
        docs_to_add.append(&mut index_curseforge(1, 400_000).await?);
    }
    //Write Indexes
    //Relevance Index

    let mut relevance_index = client
        .get_or_create("relevance_mods")
        .map_err(IndexingError::IndexDBError)?;

    let mut relevance_rules = default_rules();
    relevance_rules.push_back("desc(downloads)".to_string());

    relevance_index
        .set_settings(&default_settings().with_ranking_rules(relevance_rules.into()))
        .map_err(IndexingError::IndexDBError)?;

    for chunk in docs_to_add.chunks(MEILISEARCH_CHUNK_SIZE) {
        // TODO: get meilisearch sdk to not require cloning (ie take a reference to docs_to_add)
        // This may require making our own fork of it.
        relevance_index
            .add_documents(Vec::from(chunk), Some("mod_id"))
            .map_err(IndexingError::IndexDBError)?;
    }

    //Downloads Index
    let mut downloads_index = client
        .get_or_create("downloads_mods")
        .map_err(IndexingError::IndexDBError)?;

    let mut downloads_rules = default_rules();
    downloads_rules.push_front("desc(downloads)".to_string());

    downloads_index
        .set_settings(&default_settings().with_ranking_rules(downloads_rules.into()))
        .map_err(IndexingError::IndexDBError)?;

    for chunk in docs_to_add.chunks(MEILISEARCH_CHUNK_SIZE) {
        downloads_index
            .add_documents(Vec::from(chunk), Some("mod_id"))
            .map_err(IndexingError::IndexDBError)?;
    }

    //Updated Index
    let mut updated_index = client
        .get_or_create("updated_mods")
        .map_err(IndexingError::IndexDBError)?;

    let mut updated_rules = default_rules();
    updated_rules.push_front("desc(updated)".to_string());

    updated_index
        .set_settings(&default_settings().with_ranking_rules(updated_rules.into()))
        .map_err(IndexingError::IndexDBError)?;

    for chunk in docs_to_add.chunks(MEILISEARCH_CHUNK_SIZE) {
        updated_index
            .add_documents(Vec::from(chunk), Some("mod_id"))
            .map_err(IndexingError::IndexDBError)?;
    }

    //Created Index
    let mut newest_index = client
        .get_or_create("newest_mods")
        .map_err(IndexingError::IndexDBError)?;

    let mut newest_rules = default_rules();
    newest_rules.push_back("desc(created)".to_string());

    newest_index
        .set_settings(&default_settings().with_ranking_rules(newest_rules.into()))
        .map_err(IndexingError::IndexDBError)?;

    for chunk in docs_to_add.chunks(MEILISEARCH_CHUNK_SIZE) {
        newest_index
            .add_documents(Vec::from(chunk), Some("mod_id"))
            .map_err(IndexingError::IndexDBError)?;
    }

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
        "keywords".to_string(),
        "versions".to_string(),
        "downloads".to_string(),
        "page_url".to_string(),
        "icon_url".to_string(),
        "author_url".to_string(),
        "date_created".to_string(),
        "created".to_string(),
        "date_modified".to_string(),
        "updated".to_string(),
        "latest_version".to_string(),
        "empty".to_string(),
    ];

    let searchable_attributes = vec![
        "title".to_string(),
        "description".to_string(),
        "keywords".to_string(),
        "versions".to_string(),
        "author".to_string(),
        "empty".to_string(),
    ];

    Settings::new()
        .with_displayed_attributes(displayed_attributes)
        .with_searchable_attributes(searchable_attributes)
        .with_accept_new_fields(true)
        .with_stop_words(vec![])
        .with_synonyms(HashMap::new())
}

//endregion
