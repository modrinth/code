//! Search implementation backed by a Typesense cluster.
//!
//! # Search behavior
//!
//! Users expect very specific behavior when searching for projects on the
//! platform. When making changes to search, you must ensure that the query
//! behavior is roughly the same as before your changes - that means test with
//! strings like `sodi`, `sodiu`, `sodium`, `sodium `, etc. and make sure the
//! same results still appear.
//!
//! Before submitting a document into Typesense, we normalize the project name
//! and author using [`normalize_for_search`]. This turns a name like `My Cool
//! Mod` into `my-cool-mod`, which lets us exactly define what we consider to
//! be word separators, and fine-tune search behavior. The normalized/indexed
//! name is never shown to users; it's just used for search ops.
//!
//! # Collections
//!
//! The approach we take is to have two collections in TS: a project and a
//! version collection, which we index separately, and then join together when
//! we query. Project documents contain project metadata like name, downloads,
//! summary, etc.; version documents contain game versions, compatible loaders,
//! etc. This lets us deduplicate large chunks of data (project descriptions),
//! but keep each version doc separate for accurate filtering.
//!
//! If you're wondering why we can't just put all the info required for
//! searching into the project document, it's because collating version info
//! into the project doc will break filtering on multiple fields. Let's say for
//! example we have a project with 2 versions:
//! - NeoForge 1.21.1
//! - Fabric 1.20.0
//!
//! If we put both of these versions into the project object, we'd end up with
//! a document like:
//! ```json
//! {
//!   "name": "My Project",
//!   "game_versions": ["1.21.1", "1.20.0"],
//!   "loaders": ["neoforge", "fabric"]
//! }
//! ```
//!
//! If a user now searches for a mod for NeoForge 1.20.0, this project will
//! match - when in reality it shouldn't, because it has no versions compatible
//! with NeoForge 1.20.0! Only either Neoforge 1.21.1, or Fabric 1.20.0.
//! Keeping the projects/versions document split retains this behavior.
//!
//! What if we put a `versions` object into the project document? This is
//! terrible because:
//! - it makes project documents massive
//! - it forces an entire project document to be rebuilt when one change to a
//!   version is made
//!
//! # Performance
//!
//! Our collections are quite large for a platform of our size. To ensure
//! indexing and query performance, we do a few things.
//!
//! ## Indexing
//!
//! We use incremental indexing to update projects and version documents when
//! a user makes a change ASAP[^1]. When a project or version is updated, we
//! push a message "update this project ID/this version ID" into a Kafka topic;
//! the `incremental-index-search` task then consumes all the Kafka messages
//! and applies the actual indexing updates to the Typesense cluster, and
//! commits the Kafka messages.
//!
//! We do also have the `index-search` task which does a full reindex, but this
//! shouldn't be required in normal operation.
//!
//! Project categories are deliberately denormalized into every version
//! document to avoid a second join when categories are combined with other
//! version filters. Changing a project's categories must therefore reindex all
//! of that project's version documents.
//!
//! When batching Typesense update operations, we batch by both project count
//! and document count. Since a project can have an unbounded number of
//! versions, we can (and will) have projects with thousands of versions. If we
//! were to send updates for just this project alone, we will be sending a lot
//! more data in a single HTTP request, which could timeout the operation.
//!
//! [^1]: Okay, not exactly ASAP. Once the Kafka consumer receives the first
//! message, it waits for a small configurable time to batch a bunch of changes
//! together, and updates Typesense in bulk with those.
//!
//! ## Querying
//!
//! Having two collections is cool and all, but joins over such a large dataset
//! causes queries to be a lot slower than if we're just querying one
//! collection. To speed this up, we use some tricks.
//!
//! ### Filtering on multiple version fields
//!
//! If you search for `categories = fabric AND game_versions = 1.21`, then
//! you're searching for *one* project with *both* these conditions set.
//! Project categories are inherited by every version document, while loaders
//! remain specific to each version. This preserves the legacy behavior where
//! `categories` can match either a project category or a loader, and allows
//! both conditions to use one version join.
//!
//! Therefore, this uses *one* join over versions, rather than two joins
//! each matching different versions of the same project. Using one join here
//! is faster than two. It's even faster if you search for more facets like
//! category, game version, loader, and environment.
//!
//! ### Whole-field tokenization
//!
//! For fields like a version's `categories`, `environment`, `game_versions`
//! etc. it makes no sense to do full-text search on this (unlike name and
//! author), since the full set of categories, environments, etc. is known
//! and finite. All of these values can be represented as whitespace-free,
//! atomic, single tokens in Typesense, if we set
//! `symbols_to_index = ["-", ".", "_"]`. This means that `fabric-api` is
//! treated as one token `fabric-api`, not as `fabric` and `api`. So if we use
//! the filter `categories:fabric-api`, it will match for that *one token*
//! `fabric-api` instead of full text search.
//!
//! For this, we use the `:` operator instead of `:=` to tell
//! Typesense to treat this as an exact token match.

use std::sync::LazyLock;

use async_trait::async_trait;
use eyre::{Result, eyre};
use itertools::Itertools;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tracing::{debug, info};

use crate::database::PgPool;
use crate::database::redis::RedisPool;
use crate::env::ENV;
use crate::routes::ApiError;
use crate::search::backend::{
    SearchIndex, combined_search_filters, parse_search_index,
    parse_search_request,
};
use crate::search::indexing::index_local;
use crate::search::{
    ResultSearchProject, SearchBackend, SearchField, SearchIndexUpdate,
    SearchRequest, SearchResults, TasksCancelFilter, UploadSearchProject,
    UploadSearchVersion,
};
use crate::util::error::Context;

use self::filter_rewrite::{
    facets_to_typesense, meili_to_typesense, rewrite_filter_for_join,
};

mod filter_rewrite;

const DELETE_FILTER_ID_BATCH_SIZE: usize = 256;

#[derive(Debug, Clone)]
pub struct TypesenseConfig {
    pub url: String,
    pub api_key: String,
    pub index_prefix: String,
    pub meta_namespace: String,
    pub index_chunk_size: i64,
    pub import_batch_size: usize,
    pub delete_batch_size: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Bucketing {
    Buckets(u64),
    BucketSize(u64),
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum TextMatchType {
    MaxScore,
    #[default]
    MaxWeight,
    SumScore,
}

impl TextMatchType {
    const fn as_str(&self) -> &'static str {
        match self {
            Self::MaxScore => "max_score",
            Self::MaxWeight => "max_weight",
            Self::SumScore => "sum_score",
        }
    }
}

impl Default for Bucketing {
    fn default() -> Self {
        Self::Buckets(5)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestConfig {
    #[serde(default = "default_query_by")]
    pub query_by: Vec<String>,
    #[serde(default = "default_query_by_weights")]
    pub query_by_weights: Vec<u8>,
    #[serde(default = "default_prefix")]
    pub prefix: Vec<bool>,
    #[serde(default = "default_prioritize_exact_match")]
    pub prioritize_exact_match: bool,
    #[serde(default = "default_prioritize_num_matching_fields")]
    pub prioritize_num_matching_fields: bool,
    #[serde(default = "default_prioritize_token_positions")]
    pub prioritize_token_positions: bool,
    #[serde(default = "default_drop_tokens_threshold")]
    pub drop_tokens_threshold: usize,
    #[serde(default)]
    pub text_match_type: TextMatchType,
    #[serde(default)]
    pub bucketing: Bucketing,
    #[serde(default = "default_max_candidates")]
    pub max_candidates: usize,
}

impl Default for RequestConfig {
    fn default() -> Self {
        Self {
            query_by: default_query_by(),
            query_by_weights: default_query_by_weights(),
            prefix: default_prefix(),
            prioritize_exact_match: default_prioritize_exact_match(),
            prioritize_num_matching_fields:
                default_prioritize_num_matching_fields(),
            prioritize_token_positions: default_prioritize_token_positions(),
            drop_tokens_threshold: default_drop_tokens_threshold(),
            text_match_type: TextMatchType::default(),
            bucketing: Bucketing::default(),
            max_candidates: default_max_candidates(),
        }
    }
}

fn default_query_by() -> Vec<String> {
    [
        "name",
        "indexed_name",
        "slug",
        "author",
        "indexed_author",
        "summary",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn default_query_by_weights() -> Vec<u8> {
    vec![15, 15, 10, 3, 3, 1]
}

fn default_prefix() -> Vec<bool> {
    vec![true, true, true, true, true, true]
}

const fn default_prioritize_exact_match() -> bool {
    true
}

const fn default_prioritize_num_matching_fields() -> bool {
    false
}

const fn default_prioritize_token_positions() -> bool {
    true
    // false
}

const fn default_drop_tokens_threshold() -> usize {
    0
    // 1
}

const fn default_max_candidates() -> usize {
    8
}

impl TypesenseConfig {
    pub fn new(meta_namespace: Option<String>) -> Self {
        Self {
            url: ENV.TYPESENSE_URL.clone(),
            api_key: ENV.TYPESENSE_API_KEY.clone(),
            index_prefix: ENV.TYPESENSE_INDEX_PREFIX.clone(),
            meta_namespace: meta_namespace.unwrap_or_default(),
            index_chunk_size: ENV.SEARCH_INDEX_CHUNK_SIZE,
            import_batch_size: ENV.TYPESENSE_IMPORT_BATCH_SIZE,
            delete_batch_size: ENV.TYPESENSE_DELETE_BATCH_SIZE,
        }
    }

    pub fn get_alias_name(&self, index: &str) -> String {
        if self.meta_namespace.is_empty() {
            format!("{}_{}", self.index_prefix, index)
        } else {
            format!("{}_{}_{}", self.meta_namespace, self.index_prefix, index)
        }
    }

    fn get_next_collection_name(
        &self,
        alias_name: &str,
        use_alt: bool,
    ) -> String {
        if use_alt {
            format!("{alias_name}__alt")
        } else {
            format!("{alias_name}__current")
        }
    }
}

struct TypesenseClient {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl TypesenseClient {
    fn new(url: &str, api_key: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
        }
    }

    fn request(&self, method: Method, path: &str) -> reqwest::RequestBuilder {
        self.client
            .request(method, format!("{}{}", self.base_url, path))
            .header("X-TYPESENSE-API-KEY", &self.api_key)
    }

    async fn get_alias(&self, name: &str) -> Result<Option<String>> {
        let resp = self
            .request(Method::GET, &format!("/aliases/{name}"))
            .send()
            .await
            .wrap_err("failed to GET Typesense alias")?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        let body = resp
            .json::<Value>()
            .await
            .wrap_err("failed to parse alias response")?;
        Ok(body["collection_name"].as_str().map(ToString::to_string))
    }

    async fn upsert_alias(&self, alias: &str, collection: &str) -> Result<()> {
        let resp = self
            .request(Method::PUT, &format!("/aliases/{alias}"))
            .json(&json!({"collection_name": collection}))
            .send()
            .await
            .wrap_err("failed to PUT Typesense alias")?;
        if !resp.status().is_success() {
            let body = resp.json::<Value>().await.unwrap_or_default();
            return Err(eyre!("failed to upsert alias `{alias}`: {body}"));
        }
        Ok(())
    }

    async fn delete_alias_if_exists(&self, alias: &str) -> Result<()> {
        let resp = self
            .request(Method::DELETE, &format!("/aliases/{alias}"))
            .send()
            .await
            .wrap_err("failed to DELETE Typesense alias")?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(());
        }
        if !resp.status().is_success() {
            let body = resp.json::<Value>().await.unwrap_or_default();
            return Err(eyre!("failed to delete alias `{alias}`: {body}"));
        }
        Ok(())
    }

    async fn collection_exists(&self, name: &str) -> Result<bool> {
        let resp = self
            .request(Method::GET, &format!("/collections/{name}"))
            .send()
            .await
            .wrap_err("failed to check Typesense collection existence")?;
        Ok(resp.status().is_success())
    }

    async fn create_collection(&self, schema: &Value) -> Result<()> {
        let resp = self
            .request(Method::POST, "/collections")
            .json(schema)
            .send()
            .await
            .wrap_err("failed to POST Typesense collection")?;
        if !resp.status().is_success() {
            let body = resp.json::<Value>().await.unwrap_or_default();
            return Err(eyre!("failed to create collection: {body}"));
        }
        Ok(())
    }

    async fn delete_collection_if_exists(&self, name: &str) -> Result<()> {
        let resp = self
            .request(Method::DELETE, &format!("/collections/{name}"))
            .send()
            .await
            .wrap_err("failed to DELETE Typesense collection")?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(());
        }
        if !resp.status().is_success() {
            let body = resp.json::<Value>().await.unwrap_or_default();
            return Err(eyre!("failed to delete collection `{name}`: {body}"));
        }
        Ok(())
    }

    async fn import_documents(
        &self,
        collection: &str,
        jsonl: String,
    ) -> Result<()> {
        let url = format!(
            "/collections/{collection}/documents/import?action=upsert&dirty_values=coerce_or_drop"
        );
        let resp = self
            .request(Method::POST, &url)
            .header("Content-Type", "text/plain")
            .body(jsonl)
            .send()
            .await
            .wrap_err("failed to POST Typesense document import")?;
        let status = resp.status();
        let body = resp
            .text()
            .await
            .wrap_err("failed to read import response body")?;
        if !status.is_success() {
            return Err(eyre!(
                "document import into `{collection}` failed ({status}): {body}"
            ));
        }
        // Typesense always returns HTTP 200; individual lines signal per-doc success.
        let failures = body
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| match serde_json::from_str::<Value>(line) {
                Ok(result) if result["success"].as_bool() == Some(true) => None,
                Ok(result) => Some(
                    result["error"]
                        .as_str()
                        .unwrap_or(
                            "Typesense returned an unsuccessful import result",
                        )
                        .to_string(),
                ),
                Err(err) => Some(format!(
                    "failed to parse Typesense import result: {err}"
                )),
            })
            .collect::<Vec<_>>();
        if !failures.is_empty() {
            let failure_count = failures.len();
            let errors = failures
                .into_iter()
                .unique()
                .take(10)
                .collect::<Vec<_>>()
                .join("; ");
            return Err(eyre!(
                "{failure_count} document(s) failed to import into `{collection}`: {errors}"
            ));
        }
        Ok(())
    }

    async fn delete_documents_by_filter(
        &self,
        collection: &str,
        filter_by: &str,
        batch_size: usize,
    ) -> Result<()> {
        let resp = self
            .request(
                Method::DELETE,
                &format!(
					"/collections/{collection}/documents?filter_by={}&batch_size={batch_size}",
					urlencoding::encode(filter_by)
				),
            )
            .send()
            .await
            .wrap_err("failed to DELETE Typesense documents by filter")?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(());
        }
        if !resp.status().is_success() {
            let body = resp.json::<Value>().await.unwrap_or_default();
            return Err(eyre!(
                "failed to delete documents from `{collection}` (filter={filter_by}): {body}"
            ));
        }
        Ok(())
    }
}

pub struct Typesense {
    pub config: TypesenseConfig,
    client: TypesenseClient,
}

pub struct TypesenseFieldSpec {
    pub path: &'static str,
    pub ty: &'static str,
    pub facet: bool,
    pub sort: bool,
    pub optional: bool,
    pub token_separators: &'static [&'static str],
}

impl SearchField {
    const fn is_version_field(self) -> bool {
        matches!(
            self,
            Self::Categories
                | Self::ProjectTypes
                | Self::Environment
                | Self::GameVersions
                | Self::ClientSide
                | Self::ServerSide
        )
    }

    pub const fn typesense_spec(self) -> TypesenseFieldSpec {
        match self {
            SearchField::Categories => TypesenseFieldSpec {
                path: "categories",
                ty: "string[]",
                facet: true,
                sort: false,
                optional: true,
                token_separators: &["-"],
            },
            SearchField::Name => TypesenseFieldSpec {
                path: "name",
                ty: "string",
                facet: true,
                sort: false,
                optional: false,
                token_separators: &["-"],
            },
            SearchField::Author => TypesenseFieldSpec {
                path: "author",
                ty: "string",
                facet: true,
                sort: false,
                optional: false,
                token_separators: &["-"],
            },
            SearchField::License => TypesenseFieldSpec {
                path: "license",
                ty: "string",
                facet: true,
                sort: false,
                optional: true,
                token_separators: &["-"],
            },
            SearchField::ProjectTypes => TypesenseFieldSpec {
                path: "project_types",
                ty: "string[]",
                facet: true,
                sort: false,
                optional: true,
                token_separators: &["-"],
            },
            SearchField::AllProjectTypes => TypesenseFieldSpec {
                path: "all_project_types",
                ty: "string[]",
                facet: true,
                sort: false,
                optional: true,
                token_separators: &["-"],
            },
            SearchField::ProjectId => TypesenseFieldSpec {
                path: "project_id",
                ty: "string",
                facet: true,
                sort: false,
                optional: false,
                token_separators: &["-"],
            },
            SearchField::OpenSource => TypesenseFieldSpec {
                path: "open_source",
                ty: "bool",
                facet: true,
                sort: false,
                optional: true,
                token_separators: &["-"],
            },
            SearchField::Environment => TypesenseFieldSpec {
                path: "environment",
                ty: "string[]",
                facet: true,
                sort: false,
                optional: true,
                token_separators: &["-"],
            },
            SearchField::GameVersions => TypesenseFieldSpec {
                path: "game_versions",
                ty: "string[]",
                facet: true,
                sort: false,
                optional: true,
                token_separators: &["-", "."],
            },
            SearchField::ClientSide => TypesenseFieldSpec {
                path: "client_side",
                ty: "string[]",
                facet: true,
                sort: false,
                optional: true,
                token_separators: &["-"],
            },
            SearchField::ServerSide => TypesenseFieldSpec {
                path: "server_side",
                ty: "string[]",
                facet: true,
                sort: false,
                optional: true,
                token_separators: &["-"],
            },
            SearchField::MinecraftServerRegion => TypesenseFieldSpec {
                path: "minecraft_server.region",
                ty: "string",
                facet: true,
                sort: false,
                optional: true,
                token_separators: &["-"],
            },
            SearchField::MinecraftServerLanguages => TypesenseFieldSpec {
                path: "minecraft_server.languages",
                ty: "string[]",
                facet: true,
                sort: false,
                optional: true,
                token_separators: &["-"],
            },
            SearchField::MinecraftJavaServerContentKind => TypesenseFieldSpec {
                path: "minecraft_java_server.content.kind",
                ty: "string",
                facet: true,
                sort: false,
                optional: true,
                token_separators: &["-"],
            },
            SearchField::MinecraftJavaServerContentSupportedGameVersions => {
                TypesenseFieldSpec {
                    path: "minecraft_java_server.content.supported_game_versions",
                    ty: "string[]",
                    facet: true,
                    sort: false,
                    optional: true,
                    token_separators: &["-", "."],
                }
            }
            SearchField::MinecraftJavaServerPingData => TypesenseFieldSpec {
                path: "minecraft_java_server.ping.data",
                ty: "object",
                facet: true,
                sort: false,
                optional: true,
                token_separators: &["-"],
            },
            SearchField::DependencyProjectIds => TypesenseFieldSpec {
                path: "dependency_project_ids",
                ty: "string[]",
                facet: true,
                sort: false,
                optional: true,
                token_separators: &["-"],
            },
            SearchField::CompatibleDependencyProjectIds => TypesenseFieldSpec {
                path: "compatible_dependency_project_ids",
                ty: "string[]",
                facet: true,
                sort: false,
                optional: true,
                token_separators: &["-"],
            },
        }
    }
}

const VERSION_FILTER_SYMBOLS: &[&str] = &["-", ".", "_"];

static TYPESENSE_SEARCH_FIELDS: LazyLock<Vec<Value>> = LazyLock::new(|| {
    use strum::IntoEnumIterator;

    SearchField::iter()
        .map(|field| {
            let spec = field.typesense_spec();

            let token_separators = spec
                .token_separators
                .iter()
                .map(|sep| Value::String((*sep).to_string()))
                .collect::<Vec<_>>();

            let mut obj = serde_json::Map::from_iter([
                ("name".to_string(), Value::String(spec.path.to_string())),
                ("type".to_string(), Value::String(spec.ty.to_string())),
                (
                    "token_separators".to_string(),
                    Value::Array(token_separators),
                ),
            ]);
            if spec.facet {
                obj.insert("facet".to_string(), Value::Bool(true));
            }
            if spec.sort {
                obj.insert("sort".to_string(), Value::Bool(true));
            }
            if spec.optional {
                obj.insert("optional".to_string(), Value::Bool(true));
            }
            Value::Object(obj)
        })
        .collect()
});

impl Typesense {
    pub fn new(config: TypesenseConfig) -> Self {
        let client = TypesenseClient::new(&config.url, &config.api_key);
        Self { config, client }
    }

    fn project_collection_schema(name: &str) -> Value {
        let mut fields = vec![
            json!({"name": "summary", "type": "string", "facet": false}),
            json!({"name": "slug", "type": "string", "facet": false}),
            json!({"name": "indexed_name", "type": "string", "facet": false, "stem": true}),
            json!({"name": "indexed_author", "type": "string", "facet": false}),
            json!({"name": "log_downloads", "type": "float", "sort": true}),
            json!({"name": "downloads", "type": "int32", "sort": true}),
            json!({"name": "follows", "type": "int32", "facet": true, "sort": true}),
            json!({"name": "created_timestamp", "type": "int64", "sort": true}),
            json!({"name": "modified_timestamp", "type": "int64", "sort": true}),
            json!({"name": "version_published_timestamp", "type": "int64", "sort": true, "optional": true}),
            json!({"name": "minecraft_java_server.verified_plays_2w", "type": "int64", "sort": true, "optional": true}),
            json!({"name": "minecraft_java_server.is_online", "type": "bool", "sort": true, "optional": true}),
            json!({"name": "minecraft_java_server.ping.data.players_online", "type": "int32", "sort": true, "optional": true}),
            json!({"name": "dependencies", "type": "object[]", "optional": true}),
            json!({"name": "project_categories", "type": "string[]", "facet": true, "optional": true}),
        ];
        fields.extend(TYPESENSE_SEARCH_FIELDS.iter().cloned());

        json!({
            "name": name,
            "enable_nested_fields": true,
            "fields": fields,
            "default_sorting_field": "log_downloads"
        })
    }

    fn version_collection_schema(
        name: &str,
        projects_collection: &str,
    ) -> Value {
        use strum::IntoEnumIterator;

        let mut fields = SearchField::iter()
            .filter(|field| field.is_version_field())
            .map(|field| {
                let spec = field.typesense_spec();
                json!({
                    "name": spec.path,
                    "type": spec.ty,
                    "facet": spec.facet,
                    "optional": spec.optional,
                    "symbols_to_index": VERSION_FILTER_SYMBOLS,
                })
            })
            .collect::<Vec<_>>();
        fields.extend([
            json!({
                "name": "version_id",
                "type": "string",
            }),
            json!({
                "name": "project_id",
                "type": "string",
                "reference": format!("{projects_collection}.id"),
                "async_reference": true,
                "cascade_delete": false,
            }),
            json!({
                "name": "version_published_timestamp",
                "type": "int64",
                "sort": true,
            }),
        ]);

        json!({
            "name": name,
            "fields": fields,
        })
    }

    fn text_match_sort_field(request_config: &RequestConfig) -> String {
        match request_config.bucketing {
            Bucketing::Buckets(count) => {
                format!("_text_match(buckets:{count}):desc")
            }
            Bucketing::BucketSize(size) => {
                format!("_text_match(bucket_size:{size}):desc")
            }
        }
    }

    fn get_sort_fields(
        &self,
        index: SearchIndex,
        request_config: &RequestConfig,
    ) -> String {
        // NOTE: we can only sort by max 3 fields here - typesense will not let us sort by more
        let text_match = Self::text_match_sort_field(request_config);
        match index {
            SearchIndex::Relevance => format!(
                "{text_match},log_downloads:desc,version_published_timestamp:desc"
            ),
            SearchIndex::Downloads => {
                "log_downloads:desc,version_published_timestamp:desc"
                    .to_string()
            }
            SearchIndex::Follows => {
                "follows:desc,version_published_timestamp:desc".to_string()
            }
            SearchIndex::Updated => {
                "modified_timestamp:desc,version_published_timestamp:desc"
                    .to_string()
            }
            SearchIndex::Newest => {
                "created_timestamp:desc,version_published_timestamp:desc"
                    .to_string()
            }
            SearchIndex::MinecraftJavaServerVerifiedPlays2w => format!(
                "{text_match},minecraft_java_server.verified_plays_2w:desc,minecraft_java_server.is_online:desc"
            ),
            SearchIndex::MinecraftJavaServerPlayersOnline => format!(
                "{text_match},minecraft_java_server.is_online:desc,minecraft_java_server.ping.data.players_online:desc"
            ),
        }
    }

    fn query_by(request_config: &RequestConfig) -> String {
        request_config.query_by.join(",")
    }

    fn query_by_weights(request_config: &RequestConfig) -> Option<String> {
        (!request_config.query_by_weights.is_empty()).then(|| {
            request_config
                .query_by_weights
                .iter()
                .map(u8::to_string)
                .collect::<Vec<_>>()
                .join(",")
        })
    }

    fn prefix(request_config: &RequestConfig) -> Option<String> {
        (!request_config.prefix.is_empty()).then(|| {
            request_config
                .prefix
                .iter()
                .map(bool::to_string)
                .collect::<Vec<_>>()
                .join(",")
        })
    }

    fn get_sort_index(
        &self,
        index: &str,
        new_filters: Option<&str>,
        request_config: &RequestConfig,
    ) -> Result<(String, String), ApiError> {
        let sort = parse_search_index(index, new_filters)?;
        let alias = self.config.get_alias_name("projects");
        Ok((alias, self.get_sort_fields(sort.index, request_config)))
    }

    /// Builds a Typesense `filter_by` string from the [`SearchRequest`].
    ///
    /// Handles the new-style filter string, legacy facets JSON, and the legacy
    /// `filters`/`version` fields, translating each from Meilisearch filter
    /// syntax to Typesense filter syntax.
    fn build_filter(
        info: &SearchRequest,
        versions_collection: &str,
    ) -> Result<Option<String>, ApiError> {
        let facet_part = if let Some(facets_json) = info.facets.as_deref() {
            Some(
                facets_to_typesense(facets_json)
                    .wrap_request_err("failed to parse facets")?,
            )
        } else {
            None
        };

        let new_filters_part =
            info.new_filters.as_deref().map(meili_to_typesense);

        let legacy_part = if info.new_filters.is_none() {
            combined_search_filters(info).map(|f| meili_to_typesense(&f))
        } else {
            None
        };

        let filter_part = new_filters_part.or(legacy_part);

        let filter = match (facet_part, filter_part) {
            (Some(f), Some(l)) if !f.is_empty() && !l.is_empty() => {
                Some(format!("({f}) && ({l})"))
            }
            (Some(f), _) if !f.is_empty() => Some(f),
            (_, Some(l)) if !l.is_empty() => Some(l),
            _ => None,
        };

        filter
            .map(|filter| {
                rewrite_filter_for_join(&filter, versions_collection)
                    .wrap_request_err("failed to rewrite search filter")
            })
            .transpose()
    }

    async fn import_document_batches<T>(
        &self,
        collections: &[String],
        documents: &[T],
        serialize: fn(&[T]) -> Result<String>,
    ) -> Result<()> {
        let batch_size = self.config.import_batch_size.max(1);

        for batch in documents.chunks(batch_size) {
            let jsonl = serialize(batch)?;

            for collection in collections {
                info!(
                    collection,
                    document_count = batch.len(),
                    content_length_bytes = jsonl.len(),
                    "sending Typesense document import"
                );
                self.client
                    .import_documents(collection, jsonl.clone())
                    .await?;
            }
        }

        Ok(())
    }

    async fn existing_write_collections(
        &self,
        alias: &str,
    ) -> Result<Vec<String>> {
        let mut collections = self
            .client
            .get_alias(alias)
            .await?
            .into_iter()
            .collect_vec();

        for collection in [
            self.config.get_next_collection_name(alias, true),
            self.config.get_next_collection_name(alias, false),
        ] {
            if !collections.contains(&collection)
                && self.client.collection_exists(&collection).await?
            {
                collections.push(collection);
            }
        }

        Ok(collections)
    }

    async fn delete_ids_from_write_collections(
        &self,
        alias: &str,
        field: &str,
        ids: &[String],
    ) -> Result<()> {
        let collections = self.existing_write_collections(alias).await?;
        for ids in ids.chunks(DELETE_FILTER_ID_BATCH_SIZE) {
            let filter = format!("{field}:[{}]", ids.iter().join(", "));
            for collection in &collections {
                self.client
                    .delete_documents_by_filter(
                        collection,
                        &filter,
                        self.config.delete_batch_size,
                    )
                    .await?;
            }
        }
        Ok(())
    }

    async fn delete_legacy_filtered_collections(&self) -> Result<()> {
        let alias = self.config.get_alias_name("projects_filtered");
        let live = self.client.get_alias(&alias).await?;
        let shadow_alt = self.config.get_next_collection_name(&alias, true);
        let shadow_current =
            self.config.get_next_collection_name(&alias, false);

        self.client.delete_alias_if_exists(&alias).await?;
        for collection in live
            .into_iter()
            .chain([shadow_alt, shadow_current])
            .unique()
        {
            self.client.delete_collection_if_exists(&collection).await?;
        }

        Ok(())
    }
}

#[async_trait]
impl SearchBackend for Typesense {
    async fn search_for_project_raw(
        &self,
        info: &SearchRequest,
    ) -> Result<SearchResults, ApiError> {
        let parsed = parse_search_request(info)?;
        let (collection_alias, sort_by) = self.get_sort_index(
            parsed.index,
            info.new_filters.as_deref(),
            &info.typesense_config,
        )?;
        let versions_alias = self.config.get_alias_name("versions");
        let filter_by = Self::build_filter(info, &versions_alias)?;

        let q = if parsed.query.is_empty() {
            "*"
        } else {
            parsed.query
        };

        let query_by = Self::query_by(&info.typesense_config);

        let mut params: Vec<(&str, String)> = vec![
            ("q", q.to_string()),
            ("query_by", query_by),
            (
                "prioritize_exact_match",
                info.typesense_config.prioritize_exact_match.to_string(),
            ),
            (
                "prioritize_num_matching_fields",
                info.typesense_config
                    .prioritize_num_matching_fields
                    .to_string(),
            ),
            (
                "prioritize_token_positions",
                info.typesense_config.prioritize_token_positions.to_string(),
            ),
            (
                "drop_tokens_threshold",
                info.typesense_config.drop_tokens_threshold.to_string(),
            ),
            (
                "text_match_type",
                info.typesense_config.text_match_type.as_str().to_string(),
            ),
            ("sort_by", sort_by.to_string()),
            ("page", parsed.page.to_string()),
            ("per_page", parsed.hits_per_page.to_string()),
            (
                "max_candidates",
                info.typesense_config.max_candidates.to_string(),
            ),
        ];
        if let Some(query_by_weights) =
            Self::query_by_weights(&info.typesense_config)
        {
            params.push(("query_by_weights", query_by_weights));
        }
        if let Some(prefix) = Self::prefix(&info.typesense_config) {
            params.push(("prefix", prefix));
        }
        if let Some(filter) = &filter_by {
            params.push(("filter_by", filter.clone()));
            if filter.contains(&format!("${versions_alias}(")) {
                params.push((
                    "include_fields",
                    format!(
                        "${versions_alias}(version_id, sort_by: version_published_timestamp:desc, limit:1, strategy: nest_array) as matching_versions"
                    ),
                ));
            }
        }

        let resp = self
            .client
            .request(Method::POST, "/multi_search")
            .json(&json!({
                "searches": [
                    serde_json::Map::from_iter(
                        params.iter().map(|(k, v)| ((*k).to_string(), Value::String(v.clone())))
                    )
                    .into_iter()
                    .chain([(
                        "collection".to_string(),
                        Value::String(collection_alias.clone())
                    )])
                    .collect::<serde_json::Map<String, Value>>()
                ]
            }))
            .send()
            .await
            .wrap_internal_err("failed to execute Typesense search")?;

        if !resp.status().is_success() {
            let body = resp.json::<Value>().await.unwrap_or_default();
            return Err(ApiError::Internal(eyre!(
                "Typesense search failed: {body}"
            )));
        }

        let body = resp
            .json::<Value>()
            .await
            .wrap_internal_err("failed to parse Typesense search response")?;

        let body = body["results"]
            .as_array()
            .and_then(|results| results.first())
            .cloned()
            .unwrap_or(body);

        let total_hits = body["found"].as_u64().unwrap_or(0) as usize;

        let hits = body["hits"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|hit| {
                let mut doc = hit.get("document")?.clone();
                if let Some(obj) = doc.as_object_mut() {
                    obj.remove("id");
                    let matching_version_id =
                        obj.remove("matching_versions").and_then(|versions| {
                            versions
                                .as_array()
                                .and_then(|versions| versions.first())
                                .or_else(|| {
                                    versions.as_object().map(|_| &versions)
                                })
                                .and_then(|version| version.get("version_id"))
                                .and_then(Value::as_str)
                                .map(ToString::to_string)
                        });
                    if let Some(version_id) = matching_version_id {
                        obj.insert(
                            "version_id".to_string(),
                            Value::String(version_id),
                        );
                    }
                }

                let metadata = info.show_metadata.then(|| {
                    let mut m = serde_json::Map::new();
                    if let Some(score) = hit.get("text_match") {
                        m.insert("text_match".to_string(), score.clone());
                    }
                    if let Some(match_info) = hit.get("text_match_info") {
                        m.insert(
                            "text_match_info".to_string(),
                            match_info.clone(),
                        );
                    }
                    Value::Object(m)
                });

                let mut result: ResultSearchProject =
                    serde_json::from_value::<UploadSearchProject>(doc)
                        .ok()?
                        .into();
                result.search_metadata = metadata;
                Some(result)
            })
            .collect();

        Ok(SearchResults {
            hits,
            page: parsed.page,
            hits_per_page: parsed.hits_per_page,
            total_hits,
        })
    }

    async fn rebuild_index(
        &self,
        ro_pool: PgPool,
        redis: RedisPool,
    ) -> eyre::Result<()> {
        info!("starting project indexing");

        let projects_alias = self.config.get_alias_name("projects");
        let versions_alias = self.config.get_alias_name("versions");

        let projects_current = self.client.get_alias(&projects_alias).await?;
        let versions_current = self.client.get_alias(&versions_alias).await?;

        let projects_use_alt = !projects_current
            .as_deref()
            .is_some_and(|n| n.ends_with("__alt"));
        let versions_use_alt = !versions_current
            .as_deref()
            .is_some_and(|n| n.ends_with("__alt"));

        let projects_next = self
            .config
            .get_next_collection_name(&projects_alias, projects_use_alt);
        let versions_next = self
            .config
            .get_next_collection_name(&versions_alias, versions_use_alt);

        info!("shadow collections `{projects_next}` and `{versions_next}`");

        self.client
            .delete_collection_if_exists(&versions_next)
            .await?;
        self.client
            .delete_collection_if_exists(&projects_next)
            .await?;
        self.client
            .create_collection(&Self::project_collection_schema(&projects_next))
            .await?;
        self.client
            .create_collection(&Self::version_collection_schema(
                &versions_next,
                &projects_next,
            ))
            .await?;

        let mut cursor = 0_i64;
        let mut chunk_idx = 0_usize;
        let mut total_projects = 0_usize;
        let mut total_versions = 0_usize;

        loop {
            info!("fetching index chunk {chunk_idx}");
            chunk_idx += 1;

            let (documents, next_cursor) = index_local(
                &ro_pool,
                &redis,
                cursor,
                self.config.index_chunk_size,
            )
            .await
            .wrap_err("failed to fetch projects from local DB")?;

            if documents.projects.is_empty() {
                info!(
                    "no more documents; indexed {total_projects} projects and {total_versions} versions in {chunk_idx} chunks"
                );
                break;
            }

            total_projects += documents.projects.len();
            total_versions += documents.versions.len();
            cursor = next_cursor;

            self.import_document_batches(
                std::slice::from_ref(&projects_next),
                &documents.projects,
                documents_to_jsonl,
            )
            .await?;
            self.import_document_batches(
                std::slice::from_ref(&versions_next),
                &documents.versions,
                version_documents_to_jsonl,
            )
            .await?;
        }

        info!("swapping aliases");
        self.client
            .upsert_alias(&projects_alias, &projects_next)
            .await?;
        self.client
            .upsert_alias(&versions_alias, &versions_next)
            .await?;

        info!("cleaning up old collections");
        if let Some(old) = versions_current {
            self.client.delete_collection_if_exists(&old).await?;
        }
        if let Some(old) = projects_current {
            self.client.delete_collection_if_exists(&old).await?;
        }

        self.delete_legacy_filtered_collections().await?;

        info!("indexing complete");
        Ok(())
    }

    async fn apply_update(
        &self,
        update: SearchIndexUpdate<'_>,
    ) -> eyre::Result<()> {
        let projects_alias = self.config.get_alias_name("projects");
        let versions_alias = self.config.get_alias_name("versions");

        let removed_project_ids = update
            .removed_projects
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        if !removed_project_ids.is_empty() {
            self.delete_ids_from_write_collections(
                &versions_alias,
                "project_id",
                &removed_project_ids,
            )
            .await?;
            self.delete_ids_from_write_collections(
                &projects_alias,
                "project_id",
                &removed_project_ids,
            )
            .await?;
        }

        let version_ids = update
            .removed_versions
            .iter()
            .map(ToString::to_string)
            .chain(
                update
                    .versions
                    .iter()
                    .map(|document| document.version_id.clone()),
            )
            .unique()
            .collect::<Vec<_>>();
        if !version_ids.is_empty() {
            self.delete_ids_from_write_collections(
                &versions_alias,
                "id",
                &version_ids,
            )
            .await?;
        }

        let project_ids = update
            .projects
            .iter()
            .map(|document| document.project_id.clone())
            .unique()
            .collect::<Vec<_>>();
        if !project_ids.is_empty() {
            self.delete_ids_from_write_collections(
                &projects_alias,
                "project_id",
                &project_ids,
            )
            .await?;
        }

        if !update.projects.is_empty() {
            let collections =
                self.existing_write_collections(&projects_alias).await?;
            debug!(
                ?collections,
                num_documents = update.projects.len(),
                "Replacing project documents in collections",
            );
            self.import_document_batches(
                &collections,
                update.projects,
                documents_to_jsonl,
            )
            .await?;
        }

        if !update.versions.is_empty() {
            let collections =
                self.existing_write_collections(&versions_alias).await?;
            debug!(
                ?collections,
                num_documents = update.versions.len(),
                "Replacing version documents in collections",
            );
            self.import_document_batches(
                &collections,
                update.versions,
                version_documents_to_jsonl,
            )
            .await?;
        }

        debug!("Done applying search index update");
        Ok(())
    }

    async fn tasks(&self) -> eyre::Result<Value> {
        // Typesense operations are synchronous; there is no async task queue.
        Ok(json!({"typesense": "no async tasks"}))
    }

    async fn tasks_cancel(
        &self,
        _filter: &TasksCancelFilter,
    ) -> eyre::Result<()> {
        Ok(())
    }
}

/// Serialises a batch of [`UploadSearchProject`]s to a JSONL string suitable
/// for the Typesense bulk-import endpoint.  Each document gets an `id` field
/// equal to `project_id` so Typesense can use it as the primary key.
fn documents_to_jsonl(uploads: &[UploadSearchProject]) -> Result<String> {
    let mut out = String::new();
    for upload in uploads {
        let mut doc = serde_json::to_value(upload)
            .wrap_err("failed to serialise UploadSearchProject")?;
        if let Some(obj) = doc.as_object_mut() {
            let id = obj
                .get("project_id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            obj.insert("id".to_string(), Value::String(id));

            if let Some(server) = obj
                .get_mut("minecraft_java_server")
                .and_then(Value::as_object_mut)
            {
                let is_online = server
                    .get("ping")
                    .and_then(Value::as_object)
                    .and_then(|ping| ping.get("data"))
                    .is_some_and(|data| !data.is_null());
                server.insert("is_online".to_string(), Value::Bool(is_online));
            }
        }
        out.push_str(&serde_json::to_string(&doc)?);
        out.push('\n');
    }
    Ok(out)
}

fn version_documents_to_jsonl(
    uploads: &[UploadSearchVersion],
) -> Result<String> {
    let mut out = String::new();
    for upload in uploads {
        let mut document = serde_json::to_value(upload)
            .wrap_err("failed to serialise UploadSearchVersion")?;
        if let Some(object) = document.as_object_mut() {
            object.insert(
                "id".to_string(),
                Value::String(upload.version_id.clone()),
            );
        }
        out.push_str(&serde_json::to_string(&document)?);
        out.push('\n');
    }
    Ok(out)
}
