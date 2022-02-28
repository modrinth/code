/// This module is used for the indexing from any source.
pub mod local_import;
pub mod queue;

use crate::search::{SearchConfig, UploadSearchProject};
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
    SerdeError(#[from] serde_json::Error),
    #[error("Error while parsing a timestamp: {0}")]
    ParseDateError(#[from] chrono::format::ParseError),
    #[error("Database Error: {0}")]
    SqlxError(#[from] sqlx::error::Error),
    #[error("Database Error: {0}")]
    DatabaseError(#[from] crate::database::models::DatabaseError),
    #[error("Environment Error")]
    EnvError(#[from] dotenv::Error),
}

// The chunk size for adding projects to the indexing database. If the request size
// is too large (>10MiB) then the request fails with an error.  This chunk size
// assumes a max average size of 1KiB per project to avoid this cap.
const MEILISEARCH_CHUNK_SIZE: usize = 10000;

#[derive(Debug)]
pub struct IndexingSettings {
    pub index_local: bool,
}

impl IndexingSettings {
    #[allow(dead_code)]
    pub fn from_env() -> Self {
        //FIXME: what?
        let index_local = true;

        Self { index_local }
    }
}

pub async fn index_projects(
    pool: PgPool,
    settings: IndexingSettings,
    config: &SearchConfig,
) -> Result<(), IndexingError> {
    let mut docs_to_add: Vec<UploadSearchProject> = vec![];

    if settings.index_local {
        docs_to_add.append(&mut index_local(pool.clone()).await?);
    }
    // Write Indices

    add_projects(docs_to_add, config).await?;

    Ok(())
}

pub async fn reset_indices(config: &SearchConfig) -> Result<(), IndexingError> {
    let client = config.make_client();

    client.delete_index("relevance_projects").await?;
    client.delete_index("downloads_projects").await?;
    client.delete_index("follows_projects").await?;
    client.delete_index("updated_projects").await?;
    client.delete_index("newest_projects").await?;
    Ok(())
}

async fn update_index_helper<'a>(
    client: &'a Client<'a>,
    name: &'static str,
    rule: &'static str,
) -> Result<Index<'a>, IndexingError> {
    update_index(client, name, {
        let mut rules = default_rules();
        rules.push_back(rule);
        rules.into()
    })
    .await
}

pub async fn reconfigure_indices(
    config: &SearchConfig,
) -> Result<(), IndexingError> {
    let client = config.make_client();

    // Relevance Index
    update_index_helper(&client, "relevance_projects", "desc(downloads)")
        .await?;
    update_index_helper(&client, "downloads_projects", "desc(downloads)")
        .await?;
    update_index_helper(&client, "follows_projects", "desc(follows)").await?;
    update_index_helper(
        &client,
        "updated_projects",
        "desc(modified_timestamp)",
    )
    .await?;
    update_index_helper(&client, "newest_projects", "desc(created_timestamp)")
        .await?;

    Ok(())
}

async fn update_index<'a>(
    client: &'a Client<'a>,
    name: &'a str,
    rules: Vec<&'static str>,
) -> Result<Index<'a>, IndexingError> {
    let index = match client.get_index(name).await {
        Ok(index) => index,
        Err(meilisearch_sdk::errors::Error::MeiliSearchError {
            error_code: meilisearch_sdk::errors::ErrorCode::IndexNotFound,
            ..
        }) => client.create_index(name, Some("project_id")).await?,
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
    name: &'static str,
    rules: impl FnOnce() -> Vec<&'static str>,
) -> Result<Index<'a>, IndexingError> {
    match client.get_index(name).await {
        // TODO: update index settings on startup (or delete old indices on startup)
        Ok(index) => Ok(index),
        Err(meilisearch_sdk::errors::Error::MeiliSearchError {
            error_code: meilisearch_sdk::errors::ErrorCode::IndexNotFound,
            ..
        }) => {
            // Only create index and set settings if the index doesn't already exist
            let index = client.create_index(name, Some("project_id")).await?;

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

async fn add_to_index(
    index: Index<'_>,
    mods: &[UploadSearchProject],
) -> Result<(), IndexingError> {
    for chunk in mods.chunks(MEILISEARCH_CHUNK_SIZE) {
        index.add_documents(chunk, Some("project_id")).await?;
    }
    Ok(())
}

async fn create_and_add_to_index<'a>(
    client: &'a Client<'a>,
    projects: &'a [UploadSearchProject],
    name: &'static str,
    rule: &'static str,
) -> Result<(), IndexingError> {
    let index = create_index(client, name, || {
        let mut relevance_rules = default_rules();
        relevance_rules.push_back(rule);
        relevance_rules.into()
    })
    .await?;
    add_to_index(index, projects).await?;
    Ok(())
}

pub async fn add_projects(
    projects: Vec<UploadSearchProject>,
    config: &SearchConfig,
) -> Result<(), IndexingError> {
    let client = config.make_client();

    create_and_add_to_index(
        &client,
        &projects,
        "relevance_projects",
        "desc(downloads)",
    )
    .await?;
    create_and_add_to_index(
        &client,
        &projects,
        "downloads_projects",
        "desc(downloads)",
    )
    .await?;
    create_and_add_to_index(
        &client,
        &projects,
        "follows_projects",
        "desc(follows)",
    )
    .await?;
    create_and_add_to_index(
        &client,
        &projects,
        "updated_projects",
        "desc(modified_timestamp)",
    )
    .await?;
    create_and_add_to_index(
        &client,
        &projects,
        "newest_projects",
        "desc(created_timestamp)",
    )
    .await?;

    Ok(())
}

//region Utils
fn default_rules() -> VecDeque<&'static str> {
    vec![
        "typo",
        "words",
        "proximity",
        "attribute",
        "wordsPosition",
        "exactness",
    ]
    .into()
}

fn default_settings() -> Settings {
    Settings::new()
        .with_displayed_attributes(DEFAULT_DISPLAYED_ATTRIBUTES)
        .with_searchable_attributes(DEFAULT_SEARCHABLE_ATTRIBUTES)
        .with_stop_words(Vec::<String>::new())
        .with_synonyms(HashMap::<String, Vec<String>>::new())
        .with_attributes_for_faceting(DEFAULT_ATTRIBUTES_FOR_FACETING)
}

const DEFAULT_DISPLAYED_ATTRIBUTES: &[&str] = &[
    "project_id",
    "project_type",
    "slug",
    "author",
    "title",
    "description",
    "categories",
    "versions",
    "downloads",
    "follows",
    "icon_url",
    "date_created",
    "date_modified",
    "latest_version",
    "license",
    "client_side",
    "server_side",
    "gallery",
];

const DEFAULT_SEARCHABLE_ATTRIBUTES: &[&str] =
    &["title", "description", "categories", "versions", "author"];

const DEFAULT_ATTRIBUTES_FOR_FACETING: &[&str] = &[
    "categories",
    "host",
    "versions",
    "license",
    "client_side",
    "server_side",
    "project_type",
];
//endregion

// This shouldn't be relied on for proper sorting, but it makes an
// attempt at getting proper sorting for Mojang's versions.
// This isn't currently used, but I wrote it and it works, so I'm
// keeping this mess in case someone needs it in the future.
#[allow(dead_code)]
pub fn sort_projects(a: &str, b: &str) -> std::cmp::Ordering {
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
            (true, true) => unreachable!(),
        }
    }
}
