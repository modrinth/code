/// This module is used for the indexing from any source.
pub mod local_import;
pub mod queue;

use crate::search::{SearchConfig, UploadSearchMod};
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
    #[error("Error while serializing or deserializing JSON: {0}")]
    SerDeError(#[from] serde_json::Error),
    #[error("Error while parsing a timestamp: {0}")]
    ParseDateError(#[from] chrono::format::ParseError),
    #[error("Database Error: {0}")]
    SqlxError(#[from] sqlx::error::Error),
    #[error("Database Error: {0}")]
    DatabaseError(#[from] crate::database::models::DatabaseError),
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
    #[allow(dead_code)]
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

pub async fn index_mods(
    pool: PgPool,
    settings: IndexingSettings,
    config: &SearchConfig,
) -> Result<(), IndexingError> {
    let mut docs_to_add: Vec<UploadSearchMod> = vec![];

    if settings.index_local {
        docs_to_add.append(&mut index_local(pool.clone()).await?);
    }
    // Write Indices

    add_mods(docs_to_add, config).await?;

    Ok(())
}

pub async fn reset_indices(config: &SearchConfig) -> Result<(), IndexingError> {
    let client = Client::new(&*config.address, &*config.key);

    client.delete_index("relevance_mods").await?;
    client.delete_index("downloads_mods").await?;
    client.delete_index("follows_mods").await?;
    client.delete_index("alphabetically_mods").await?;
    client.delete_index("updated_mods").await?;
    client.delete_index("newest_mods").await?;
    Ok(())
}

pub async fn reconfigure_indices(config: &SearchConfig) -> Result<(), IndexingError> {
    let client = Client::new(&*config.address, &*config.key);

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

    // Follows Index
    update_index(&client, "follows_mods", {
        let mut follows_rules = default_rules();
        follows_rules.push_front("desc(follows)".to_string());
        follows_rules.into()
    })
    .await?;

    // Alphabetically Index
    update_index(&client, "alphabetically_mods", {
        let mut alphabetically_rules = default_rules();
        alphabetically_rules.push_front("desc(title)".to_string());
        alphabetically_rules.into()
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

pub async fn add_mods(
    mods: Vec<UploadSearchMod>,
    config: &SearchConfig,
) -> Result<(), IndexingError> {
    let client = Client::new(&*config.address, &*config.key);

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

    // Follows Index
    let follows_index = create_index(&client, "follows_mods", || {
        let mut follows_rules = default_rules();
        follows_rules.push_front("desc(follows)".to_string());
        follows_rules.into()
    })
    .await?;
    add_to_index(follows_index, &mods).await?;

    // Alphabetically Index
    let alphabetically_index = create_index(&client, "alphabetically_mods", || {
        let mut alphabetically_rules = default_rules();
        alphabetically_rules.push_front("desc(title)".to_string());
        alphabetically_rules.into()
    })
        .await?;
    add_to_index(alphabetically_index, &mods).await?;

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
        "slug".to_string(),
        "author".to_string(),
        "title".to_string(),
        "description".to_string(),
        "categories".to_string(),
        "versions".to_string(),
        "downloads".to_string(),
        "follows".to_string(),
        "page_url".to_string(),
        "icon_url".to_string(),
        "author_url".to_string(),
        "date_created".to_string(),
        "date_modified".to_string(),
        "latest_version".to_string(),
        "license".to_string(),
        "client_side".to_string(),
        "server_side".to_string(),
        "host".to_string(),
    ];

    let searchable_attributes = vec![
        "title".to_string(),
        "description".to_string(),
        "categories".to_string(),
        "versions".to_string(),
        "author".to_string(),
    ];

    let stop_words: Vec<String> = Vec::new();
    let synonyms: HashMap<String, Vec<String>> = HashMap::new();

    Settings::new()
        .with_displayed_attributes(displayed_attributes)
        .with_searchable_attributes(searchable_attributes)
        .with_stop_words(stop_words)
        .with_synonyms(synonyms)
        .with_attributes_for_faceting(vec![
            String::from("categories"),
            String::from("host"),
            String::from("versions"),
            String::from("license"),
            String::from("client_side"),
            String::from("server_side"),
        ])
}

//endregion

// This shouldn't be relied on for proper sorting, but it makes an
// attempt at getting proper sorting for mojang's versions.
// This isn't currenly used, but I wrote it and it works, so I'm
// keeping this mess in case someone needs it in the future.
#[allow(dead_code)]
pub fn sort_mods(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    let cmp = a.contains('.').cmp(&b.contains('.'));
    if cmp != Ordering::Equal {
        return cmp;
    }
    let mut a = a.split(&['.', '-'] as &[char]);
    let mut b = b.split(&['.', '-'] as &[char]);
    let a = (a.next(), a.next(), a.next(), a.next());
    let b = (b.next(), b.next(), b.next(), b.next());
    if a.0 == b.0 {
        let cmp =
            a.1.map(|s| s.chars().all(|c| c.is_ascii_digit()))
                .cmp(&b.1.map(|s| s.chars().all(|c| c.is_ascii_digit())));
        if cmp != Ordering::Equal {
            return cmp;
        }
        if a.1 == b.1 {
            let cmp =
                a.2.map(|s| s.chars().all(|c| c.is_ascii_digit()))
                    .unwrap_or(true)
                    .cmp(
                        &b.2.map(|s| s.chars().all(|c| c.is_ascii_digit()))
                            .unwrap_or(true),
                    );
            if cmp != Ordering::Equal {
                return cmp;
            }
            if a.2 == b.2 {
                match (a.3.is_some(), b.3.is_some()) {
                    (false, false) => Ordering::Equal,
                    (false, true) => Ordering::Greater,
                    (true, false) => Ordering::Less,
                    (true, true) => a.3.cmp(&b.3),
                }
            } else {
                a.2.cmp(&b.2)
            }
        } else {
            a.1.cmp(&b.1)
        }
    } else {
        match (a.0 == Some("1"), b.0 == Some("1")) {
            (false, false) => a.0.cmp(&b.0),
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            (true, true) => Ordering::Equal, // unreachable
        }
    }
}
