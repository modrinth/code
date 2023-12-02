/// This module is used for the indexing from any source.
pub mod local_import;

use crate::search::{SearchConfig, UploadSearchProject};
use local_import::index_local;
use meilisearch_sdk::client::Client;
use meilisearch_sdk::indexes::Index;
use meilisearch_sdk::settings::{PaginationSetting, Settings};
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

pub async fn index_projects(pool: PgPool, config: &SearchConfig) -> Result<(), IndexingError> {
    let mut docs_to_add: Vec<UploadSearchProject> = vec![];
    let mut additional_fields: Vec<String> = vec![];

    let (mut uploads, mut loader_fields) = index_local(pool.clone()).await?;
    docs_to_add.append(&mut uploads);
    additional_fields.append(&mut loader_fields);

    // Write Indices
    add_projects(docs_to_add, additional_fields, config).await?;

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
            let task = client.create_index(name, Some("version_id")).await?;
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
            .add_documents(chunk, Some("version_id"))
            .await?
            .wait_for_completion(client, None, None)
            .await?;
    }
    Ok(())
}

async fn create_and_add_to_index(
    client: &Client,
    projects: &[UploadSearchProject],
    additional_fields: &[String],
    name: &'static str,
    custom_rules: Option<&'static [&'static str]>,
) -> Result<(), IndexingError> {
    let index = create_index(client, name, custom_rules).await?;

    let mut new_filterable_attributes = index.get_filterable_attributes().await?;
    let mut new_displayed_attributes = index.get_displayed_attributes().await?;

    new_filterable_attributes.extend(additional_fields.iter().map(|s| s.to_string()));
    new_displayed_attributes.extend(additional_fields.iter().map(|s| s.to_string()));
    index
        .set_filterable_attributes(new_filterable_attributes)
        .await?;
    index
        .set_displayed_attributes(new_displayed_attributes)
        .await?;

    add_to_index(client, index, projects).await?;
    Ok(())
}

pub async fn add_projects(
    projects: Vec<UploadSearchProject>,
    additional_fields: Vec<String>,
    config: &SearchConfig,
) -> Result<(), IndexingError> {
    let client = config.make_client();

    create_and_add_to_index(&client, &projects, &additional_fields, "projects", None).await?;

    create_and_add_to_index(
        &client,
        &projects,
        &additional_fields,
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
        .with_distinct_attribute("project_id")
        .with_displayed_attributes(DEFAULT_DISPLAYED_ATTRIBUTES)
        .with_searchable_attributes(DEFAULT_SEARCHABLE_ATTRIBUTES)
        .with_sortable_attributes(DEFAULT_SORTABLE_ATTRIBUTES)
        .with_filterable_attributes(DEFAULT_ATTRIBUTES_FOR_FACETING)
        .with_pagination(PaginationSetting {
            max_total_hits: 2147483647,
        })
}

const DEFAULT_DISPLAYED_ATTRIBUTES: &[&str] = &[
    "project_id",
    "version_id",
    "project_types",
    "slug",
    "author",
    "name",
    "description",
    "categories",
    "display_categories",
    "downloads",
    "follows",
    "icon_url",
    "date_created",
    "date_modified",
    "latest_version",
    "license",
    "gallery",
    "featured_gallery",
    "color",
];

const DEFAULT_SEARCHABLE_ATTRIBUTES: &[&str] = &["name", "description", "author", "slug"];

const DEFAULT_ATTRIBUTES_FOR_FACETING: &[&str] = &[
    "categories",
    "license",
    "project_types",
    "downloads",
    "follows",
    "author",
    "name",
    "date_created",
    "created_timestamp",
    "date_modified",
    "modified_timestamp",
    "project_id",
    "open_source",
    "color",
];

const DEFAULT_SORTABLE_ATTRIBUTES: &[&str] =
    &["downloads", "follows", "date_created", "date_modified"];
