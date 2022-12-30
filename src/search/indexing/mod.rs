/// This module is used for the indexing from any source.
pub mod local_import;

use crate::search::{SearchConfig, UploadSearchProject};
use local_import::index_local;
use meilisearch_sdk::client::Client;
use meilisearch_sdk::indexes::Index;
use meilisearch_sdk::settings::Settings;
use sqlx::postgres::PgPool;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IndexingError {
    #[error("Error while connecting to the MeiliSearch database")]
    Indexing(#[from] meilisearch_sdk::errors::Error),
    #[error("Error while serializing or deserializing JSON: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Database Error: {0}")]
    Sqlx(#[from] sqlx::error::Error),
    #[error("Database Error: {0}")]
    Database(#[from] crate::database::models::DatabaseError),
    #[error("Environment Error")]
    Env(#[from] dotenvy::Error),
    #[error("Error while awaiting index creation task")]
    Task,
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

async fn create_index(
    client: &Client,
    name: &'static str,
    custom_rules: Option<&'static [&'static str]>,
) -> Result<Index, IndexingError> {
    client
        .delete_index(name)
        .await?
        .wait_for_completion(client, None, None)
        .await?;

    match client.get_index(name).await {
        // TODO: update index settings on startup (or delete old indices on startup)
        Ok(index) => {
            index
                .set_settings(&default_settings())
                .await?
                .wait_for_completion(client, None, None)
                .await?;

            Ok(index)
        }
        Err(meilisearch_sdk::errors::Error::Meilisearch(
            meilisearch_sdk::errors::MeilisearchError {
                error_code: meilisearch_sdk::errors::ErrorCode::IndexNotFound,
                ..
            },
        )) => {
            // Only create index and set settings if the index doesn't already exist
            let task = client.create_index(name, Some("project_id")).await?;
            let task = task.wait_for_completion(client, None, None).await?;
            let index = task
                .try_make_index(client)
                .map_err(|_| IndexingError::Task)?;

            let mut settings = default_settings();

            if let Some(custom_rules) = custom_rules {
                settings = settings.with_ranking_rules(custom_rules);
            }

            index
                .set_settings(&settings)
                .await?
                .wait_for_completion(client, None, None)
                .await?;

            Ok(index)
        }
        Err(e) => {
            log::warn!("Unhandled error while creating index: {}", e);
            Err(IndexingError::Indexing(e))
        }
    }
}

async fn add_to_index(
    client: &Client,
    index: Index,
    mods: &[UploadSearchProject],
) -> Result<(), IndexingError> {
    for chunk in mods.chunks(MEILISEARCH_CHUNK_SIZE) {
        index
            .add_documents(chunk, Some("project_id"))
            .await?
            .wait_for_completion(client, None, None)
            .await?;
    }
    Ok(())
}

async fn create_and_add_to_index(
    client: &Client,
    projects: &[UploadSearchProject],
    name: &'static str,
    custom_rules: Option<&'static [&'static str]>,
) -> Result<(), IndexingError> {
    let index = create_index(client, name, custom_rules).await?;
    add_to_index(client, index, projects).await?;
    Ok(())
}

pub async fn add_projects(
    projects: Vec<UploadSearchProject>,
    config: &SearchConfig,
) -> Result<(), IndexingError> {
    let client = config.make_client();

    create_and_add_to_index(&client, &projects, "projects", None).await?;

    create_and_add_to_index(
        &client,
        &projects,
        "projects_filtered",
        Some(&[
            "sort",
            "words",
            "typo",
            "proximity",
            "attribute",
            "exactness",
        ]),
    )
    .await?;

    Ok(())
}

fn default_settings() -> Settings {
    Settings::new()
        .with_displayed_attributes(DEFAULT_DISPLAYED_ATTRIBUTES)
        .with_searchable_attributes(DEFAULT_SEARCHABLE_ATTRIBUTES)
        .with_sortable_attributes(DEFAULT_SORTABLE_ATTRIBUTES)
        .with_filterable_attributes(DEFAULT_ATTRIBUTES_FOR_FACETING)
}

const DEFAULT_DISPLAYED_ATTRIBUTES: &[&str] = &[
    "project_id",
    "project_type",
    "slug",
    "author",
    "title",
    "description",
    "categories",
    "display_categories",
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
    "featured_gallery",
    "color",
];

const DEFAULT_SEARCHABLE_ATTRIBUTES: &[&str] =
    &["title", "description", "author", "slug"];

const DEFAULT_ATTRIBUTES_FOR_FACETING: &[&str] = &[
    "categories",
    "versions",
    "license",
    "client_side",
    "server_side",
    "project_type",
    "downloads",
    "follows",
    "author",
    "title",
    "date_created",
    "date_modified",
    "project_id",
    "open_source",
    "color",
];

const DEFAULT_SORTABLE_ATTRIBUTES: &[&str] =
    &["downloads", "follows", "date_created", "date_modified"];
