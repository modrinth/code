use std::sync::LazyLock;

use ariadne::ids::base62_impl::to_base62;
use async_trait::async_trait;
use eyre::{Result, eyre};
use regex::Regex;
use reqwest::Method;
use serde_json::{Value, json};
use tracing::{info, warn};

use crate::database::PgPool;
use crate::database::redis::RedisPool;
use crate::env::ENV;
use crate::models::ids::VersionId;
use crate::routes::ApiError;
use crate::search::backend::{
    SearchIndex, SearchIndexName, combined_search_filters, parse_search_index,
    parse_search_request,
};
use crate::search::indexing::index_local;
use crate::search::{
    ResultSearchProject, SearchBackend, SearchRequest, SearchResults,
    TasksCancelFilter, UploadSearchProject,
};
use crate::util::error::Context;

#[derive(Debug, Clone)]
pub struct TypesenseConfig {
    pub url: String,
    pub api_key: String,
    pub index_prefix: String,
    pub meta_namespace: String,
    pub index_chunk_size: i64,
}

impl TypesenseConfig {
    pub fn new(meta_namespace: Option<String>) -> Self {
        Self {
            url: ENV.TYPESENSE_URL.clone(),
            api_key: ENV.TYPESENSE_API_KEY.clone(),
            index_prefix: ENV.TYPESENSE_INDEX_PREFIX.clone(),
            meta_namespace: meta_namespace.unwrap_or_default(),
            index_chunk_size: ENV.TYPESENSE_INDEX_CHUNK_SIZE,
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
        let error_count = body
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter(|l| {
                serde_json::from_str::<Value>(l)
                    .ok()
                    .and_then(|v| v["success"].as_bool())
                    .map(|ok| !ok)
                    .unwrap_or(false)
            })
            .count();
        if error_count > 0 {
            warn!(
                "{error_count} document(s) failed to import into `{collection}`"
            );
        }
        Ok(())
    }

    async fn delete_documents_by_filter(
        &self,
        collection: &str,
        filter_by: &str,
    ) -> Result<()> {
        let resp = self
			.request(
				Method::DELETE,
				&format!(
					"/collections/{collection}/documents?filter_by={}&batch_size=1000",
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

impl Typesense {
    pub fn new(config: TypesenseConfig) -> Self {
        let client = TypesenseClient::new(&config.url, &config.api_key);
        Self { config, client }
    }

    fn collection_schema(name: &str) -> Value {
        json!({
            "name": name,
            "enable_nested_fields": true,
            "fields": [
                // Wildcard catches all remaining fields via auto type detection.
                {"name": ".*", "type": "auto", "optional": true},
                // Explicit declarations ensure correct types for search/sort/filter.
                {"name": "version_id", "type": "string"},
                {"name": "project_id", "type": "string", "facet": true},
                {"name": "name", "type": "string", "facet": false},
                {"name": "summary", "type": "string", "facet": false},
                {"name": "author", "type": "string", "facet": true},
                {"name": "slug", "type": "string", "facet": false},
                {"name": "categories", "type": "string[]", "facet": true, "optional": true},
                {"name": "display_categories", "type": "string[]", "facet": true, "optional": true},
                {"name": "project_types", "type": "string[]", "facet": true, "optional": true},
                {"name": "license", "type": "string", "facet": true, "optional": true},
                {"name": "downloads", "type": "int32", "facet": true, "sort": true},
                {"name": "follows", "type": "int32", "facet": true, "sort": true},
                {"name": "created_timestamp", "type": "int64", "sort": true},
                {"name": "modified_timestamp", "type": "int64", "sort": true},
                {
                    "name": "version_published_timestamp",
                    "type": "int64",
                    "sort": true,
                    "optional": true
                },
                {"name": "open_source", "type": "bool", "facet": true, "optional": true},
                {"name": "color", "type": "int64", "optional": true},
                {"name": "date_created", "type": "string", "optional": true},
                {"name": "date_modified", "type": "string", "optional": true},
                {
                    "name": "minecraft_java_server.verified_plays_2w",
                    "type": "int64",
                    "sort": true,
                    "optional": true
                },
                {
                    "name": "minecraft_java_server.verified_plays_4w",
                    "type": "int64",
                    "sort": true,
                    "optional": true
                },
                {
                    "name": "minecraft_java_server.ping.data.players_online",
                    "type": "int32",
                    "sort": true,
                    "optional": true
                }
            ],
            "default_sorting_field": "downloads"
        })
    }

    fn get_sort_fields(&self, index: SearchIndex) -> &'static str {
        match index {
            SearchIndex::Relevance => {
                "_text_match:desc,downloads:desc,version_published_timestamp:desc"
            }
            SearchIndex::Downloads => {
                "downloads:desc,version_published_timestamp:desc"
            }
            SearchIndex::Follows => {
                "follows:desc,version_published_timestamp:desc"
            }
            SearchIndex::Updated => {
                "modified_timestamp:desc,version_published_timestamp:desc"
            }
            SearchIndex::Newest => {
                "created_timestamp:desc,version_published_timestamp:desc"
            }
            SearchIndex::MinecraftJavaServerVerifiedPlays2w => concat!(
                "minecraft_java_server.verified_plays_2w:desc,",
                "minecraft_java_server.ping.data.players_online:desc,",
                "version_published_timestamp:desc"
            ),
            SearchIndex::MinecraftJavaServerPlayersOnline => concat!(
                "minecraft_java_server.ping.data.players_online:desc,",
                "version_published_timestamp:desc"
            ),
        }
    }

    fn get_sort_index(
        &self,
        index: &str,
        new_filters: Option<&str>,
    ) -> Result<(String, &'static str), ApiError> {
        let sort = parse_search_index(index, new_filters)?;
        let alias = match sort.index_name {
            SearchIndexName::Projects => self.config.get_alias_name("projects"),
            SearchIndexName::ProjectsFiltered => {
                self.config.get_alias_name("projects_filtered")
            }
        };
        Ok((alias, self.get_sort_fields(sort.index)))
    }

    /// Builds a Typesense `filter_by` string from the [`SearchRequest`].
    ///
    /// Handles the new-style filter string, legacy facets JSON, and the legacy
    /// `filters`/`version` fields, translating each from Meilisearch filter
    /// syntax to Typesense filter syntax.
    fn build_filter(info: &SearchRequest) -> Result<Option<String>, ApiError> {
        if let Some(new_filters) = info.new_filters.as_deref() {
            return Ok(Some(meili_to_typesense(new_filters)));
        }

        let facet_part = if let Some(facets_json) = info.facets.as_deref() {
            Some(
                facets_to_typesense(facets_json)
                    .wrap_request_err("failed to parse facets")?,
            )
        } else {
            None
        };

        let legacy_part =
            combined_search_filters(info).map(|f| meili_to_typesense(&f));

        Ok(match (facet_part, legacy_part) {
            (Some(f), Some(l)) if !f.is_empty() && !l.is_empty() => {
                Some(format!("({f}) && ({l})"))
            }
            (Some(f), _) if !f.is_empty() => Some(f),
            (_, Some(l)) if !l.is_empty() => Some(l),
            _ => None,
        })
    }

    /// Ensures the alias and its backing collection both exist, creating them
    /// when necessary so reads succeed before the first full index run.
    async fn ensure_collection(&self, alias: &str) -> Result<()> {
        if self.client.get_alias(alias).await?.is_some() {
            return Ok(());
        }
        let name = self.config.get_next_collection_name(alias, false);
        if !self.client.collection_exists(&name).await? {
            self.client
                .create_collection(&Self::collection_schema(&name))
                .await?;
        }
        self.client.upsert_alias(alias, &name).await?;
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
        let (collection_alias, sort_by) =
            self.get_sort_index(parsed.index, info.new_filters.as_deref())?;
        let filter_by = Self::build_filter(info)?;

        let q = if parsed.query.is_empty() {
            "*"
        } else {
            parsed.query
        };

        let mut params: Vec<(&str, String)> = vec![
            ("q", q.to_string()),
            ("query_by", "name,slug,author,summary".to_string()),
            ("sort_by", sort_by.to_string()),
            ("page", parsed.page.to_string()),
            ("per_page", parsed.hits_per_page.to_string()),
            ("group_by", "project_id".to_string()),
            ("group_limit", "1".to_string()),
        ];
        if let Some(filter) = &filter_by {
            params.push(("filter_by", filter.clone()));
        }

        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let url = format!(
            "/collections/{collection_alias}/documents/search?{query_string}"
        );

        let resp = self
            .client
            .request(Method::GET, &url)
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

        let total_hits = body["found"].as_u64().unwrap_or(0) as usize;

        let hits = body["grouped_hits"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|group| {
                let hit = group["hits"].as_array()?.first()?.clone();
                let mut doc = hit.get("document")?.clone();
                if let Some(obj) = doc.as_object_mut() {
                    obj.remove("id");
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

    async fn index_projects(
        &self,
        ro_pool: PgPool,
        redis: RedisPool,
    ) -> eyre::Result<()> {
        info!("starting project indexing");

        let projects_alias = self.config.get_alias_name("projects");
        let filtered_alias = self.config.get_alias_name("projects_filtered");

        // Guarantee current aliases exist so reads keep working during re-index.
        self.ensure_collection(&projects_alias).await?;
        self.ensure_collection(&filtered_alias).await?;

        // Toggle the shadow collection name between __current and __alt.
        let projects_current = self.client.get_alias(&projects_alias).await?;
        let filtered_current = self.client.get_alias(&filtered_alias).await?;

        let projects_use_alt = !projects_current
            .as_deref()
            .is_some_and(|n| n.ends_with("__alt"));
        let filtered_use_alt = !filtered_current
            .as_deref()
            .is_some_and(|n| n.ends_with("__alt"));

        let projects_next = self
            .config
            .get_next_collection_name(&projects_alias, projects_use_alt);
        let filtered_next = self
            .config
            .get_next_collection_name(&filtered_alias, filtered_use_alt);

        info!("shadow collections `{projects_next}` and `{filtered_next}`");

        self.client
            .delete_collection_if_exists(&projects_next)
            .await?;
        self.client
            .delete_collection_if_exists(&filtered_next)
            .await?;
        self.client
            .create_collection(&Self::collection_schema(&projects_next))
            .await?;
        self.client
            .create_collection(&Self::collection_schema(&filtered_next))
            .await?;

        let mut cursor = 0_i64;
        let mut chunk_idx = 0_usize;
        let mut total = 0_usize;

        loop {
            info!("fetching index chunk {chunk_idx}");
            chunk_idx += 1;

            let (uploads, next_cursor) = index_local(
                &ro_pool,
                &redis,
                cursor,
                self.config.index_chunk_size,
            )
            .await
            .wrap_err("failed to fetch projects from local DB")?;

            if uploads.is_empty() {
                info!(
                    "no more documents; indexed {total} in {chunk_idx} chunks"
                );
                break;
            }

            total += uploads.len();
            cursor = next_cursor;

            let jsonl = documents_to_jsonl(&uploads)?;
            self.client
                .import_documents(&projects_next, jsonl.clone())
                .await?;
            self.client.import_documents(&filtered_next, jsonl).await?;
        }

        info!("swapping aliases");
        self.client
            .upsert_alias(&projects_alias, &projects_next)
            .await?;
        self.client
            .upsert_alias(&filtered_alias, &filtered_next)
            .await?;

        info!("cleaning up old collections");
        if let Some(old) = projects_current {
            self.client.delete_collection_if_exists(&old).await?;
        }
        if let Some(old) = filtered_current {
            self.client.delete_collection_if_exists(&old).await?;
        }

        info!("indexing complete");
        Ok(())
    }

    async fn remove_documents(&self, ids: &[VersionId]) -> eyre::Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        let id_list = ids
            .iter()
            .map(|id| to_base62(id.0))
            .collect::<Vec<_>>()
            .join(", ");
        let filter = format!("id:[{id_list}]");

        for alias in [
            self.config.get_alias_name("projects"),
            self.config.get_alias_name("projects_filtered"),
        ] {
            // Delete from both the live collection and any shadow collections.
            let live = self.client.get_alias(&alias).await?;
            let shadow_alt = self.config.get_next_collection_name(&alias, true);
            let shadow_current =
                self.config.get_next_collection_name(&alias, false);

            for collection in
                live.into_iter().chain([shadow_alt, shadow_current])
            {
                if self.client.collection_exists(&collection).await? {
                    self.client
                        .delete_documents_by_filter(&collection, &filter)
                        .await?;
                }
            }
        }
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
/// equal to `version_id` so Typesense can use it as the primary key.
fn documents_to_jsonl(uploads: &[UploadSearchProject]) -> Result<String> {
    let mut out = String::new();
    for upload in uploads {
        let mut doc = serde_json::to_value(upload)
            .wrap_err("failed to serialise UploadSearchProject")?;
        if let Some(obj) = doc.as_object_mut() {
            let id = obj
                .get("version_id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            obj.insert("id".to_string(), Value::String(id));
        }
        out.push_str(&serde_json::to_string(&doc)?);
        out.push('\n');
    }
    Ok(out)
}

/// Translates a Meilisearch filter expression into Typesense `filter_by`
/// syntax.
///
/// Transformations (applied in order):
/// 1. `field (NOT )IN [v1, v2]`  →  `field:[v1, v2]` / `field:!=[v1, v2]`
/// 2. `field op value` for op ∈ {`!=`, `>=`, `<=`, `>`, `<`, `=`}
///    →  `field:op value`
/// 3. `AND` / `OR` (case-insensitive)  →  `&&` / `||`
fn meili_to_typesense(filter: &str) -> String {
    static IN_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"(?i)\b([a-zA-Z_.][a-zA-Z0-9_.]*)\s+(NOT\s+)?IN\s*\[([^\]]*)\]",
        )
        .expect("valid regex")
    });
    static CMP_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"([a-zA-Z_.][a-zA-Z0-9_.]*)\s*(!=|>=|<=|>|<|=)\s*")
            .expect("valid regex")
    });
    static AND_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?i)\bAND\b").expect("valid regex"));
    static OR_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?i)\bOR\b").expect("valid regex"));

    // Step 1 – IN / NOT IN
    let s = IN_RE.replace_all(filter, |caps: &regex::Captures<'_>| {
        let field = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
        let is_not = caps.get(2).is_some();
        let values = caps.get(3).map(|m| m.as_str()).unwrap_or_default();
        if is_not {
            format!("{field}:!=[{values}]")
        } else {
            format!("{field}:[{values}]")
        }
    });

    // Step 2 – comparison operators (field op value → field:op value).
    let s = CMP_RE.replace_all(&s, |caps: &regex::Captures<'_>| {
        let field = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
        let op = caps.get(2).map(|m| m.as_str()).unwrap_or_default();
        format!("{field}:{op} ")
    });

    // Step 3 – logical operators
    let s = AND_RE.replace_all(&s, " && ");
    let s = OR_RE.replace_all(&s, " || ");
    s.into_owned()
}

/// Converts the legacy Meilisearch `facets` JSON array into a Typesense
/// `filter_by` string.  The outer array items are AND-ed together; the inner
/// array items are OR-ed together.
fn facets_to_typesense(facets_json: &str) -> Result<String> {
    let facets = serde_json::from_str::<Vec<Vec<Value>>>(facets_json)
        .wrap_err("failed to parse facets JSON")?;

    let and_parts: Vec<String> = facets
        .into_iter()
        .map(|or_group| {
            let or_parts: Vec<String> = or_group
                .into_iter()
                .map(|facet| {
                    let conditions: Vec<String> = if facet.is_array() {
                        serde_json::from_value::<Vec<String>>(facet)
                            .unwrap_or_default()
                    } else {
                        vec![
                            serde_json::from_value::<String>(facet)
                                .unwrap_or_default(),
                        ]
                    };
                    let and_conds: Vec<String> = conditions
                        .into_iter()
                        .map(|c| condition_to_typesense_filter(&c))
                        .collect();
                    if and_conds.len() == 1 {
                        and_conds.into_iter().next().unwrap_or_default()
                    } else {
                        format!("({})", and_conds.join(" && "))
                    }
                })
                .collect();
            if or_parts.len() == 1 {
                or_parts.into_iter().next().unwrap_or_default()
            } else {
                format!("({})", or_parts.join(" || "))
            }
        })
        .collect();

    Ok(and_parts.join(" && "))
}

/// Converts a single facet condition such as `"categories:mods"`,
/// `"categories=mods"`, or `"downloads!=100"` into a Typesense filter clause.
fn condition_to_typesense_filter(cond: &str) -> String {
    // Handle `!=` before `=` so we don't misfire on the equality arm.
    if let Some((field, value)) = cond.split_once("!=") {
        return format!("{}:!= {}", field.trim(), value.trim());
    }
    if let Some((field, value)) = cond.split_once(':') {
        return format!("{}:= {}", field.trim(), value.trim());
    }
    if let Some((field, value)) = cond.split_once('=') {
        return format!("{}:= {}", field.trim(), value.trim());
    }
    cond.to_string()
}
