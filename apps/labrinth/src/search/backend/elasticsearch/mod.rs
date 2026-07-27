//! Search implementation backed by an Elasticsearch cluster.
//!
//! Projects and versions share an index and use an Elasticsearch join field.
//! This keeps version filters correlated without duplicating every version
//! into its project document.

use async_trait::async_trait;
use eyre::{Result, eyre};
use itertools::Itertools;
use reqwest::{Method, Response, StatusCode};
use serde::Serialize;
use serde_json::{Map, Value, json};
use tracing::{debug, info};
use xredis::RedisPool;

use crate::database::PgPool;
use crate::env::ENV;
use crate::routes::ApiError;
use crate::search::backend::{
	SearchIndex, combined_search_filters, parse_search_index,
	parse_search_request,
};
use crate::search::filter::{
	FilterExpr, from_legacy_v2_facets_json, normalize, parse_expression,
};
use crate::search::indexing::index_local;
use crate::search::{
	ResultSearchProject, SearchBackend, SearchIndexUpdate, SearchRequest,
	SearchResults, TasksCancelFilter, UploadSearchProject, UploadSearchVersion,
};
use crate::util::error::Context;

use self::filter::{ElasticsearchFilter, serialize_filter};

mod filter;

const DELETE_FILTER_ID_BATCH_SIZE: usize = 1024;

#[derive(Debug, Clone)]
pub struct ElasticsearchConfig {
	pub url: String,
	pub username: String,
	pub password: String,
	pub index_prefix: String,
	pub meta_namespace: String,
	pub index_chunk_size: i64,
	pub bulk_batch_size: usize,
}

impl ElasticsearchConfig {
	pub fn new(meta_namespace: Option<String>) -> Self {
		Self {
			url: ENV.ELASTICSEARCH_URL.clone(),
			username: ENV.ELASTICSEARCH_USERNAME.clone(),
			password: ENV.ELASTICSEARCH_PASSWORD.clone(),
			index_prefix: ENV.ELASTICSEARCH_INDEX_PREFIX.clone(),
			meta_namespace: meta_namespace.unwrap_or_default(),
			index_chunk_size: ENV.SEARCH_INDEX_CHUNK_SIZE,
			bulk_batch_size: ENV.ELASTICSEARCH_BULK_BATCH_SIZE,
		}
	}

	fn alias_name(&self) -> String {
		if self.meta_namespace.is_empty() {
			format!("{}_projects", self.index_prefix)
		} else {
			format!(
				"{}_{}_projects",
				self.meta_namespace, self.index_prefix
			)
		}
	}

	fn next_index_name(&self, alias: &str, use_alt: bool) -> String {
		if use_alt {
			format!("{alias}__alt")
		} else {
			format!("{alias}__current")
		}
	}
}

struct ElasticsearchClient {
	client: reqwest::Client,
	base_url: String,
	username: String,
	password: String,
}

impl ElasticsearchClient {
	fn new(config: &ElasticsearchConfig) -> Self {
		Self {
			client: reqwest::Client::new(),
			base_url: config.url.trim_end_matches('/').to_string(),
			username: config.username.clone(),
			password: config.password.clone(),
		}
	}

	fn request(&self, method: Method, path: &str) -> reqwest::RequestBuilder {
		let request = self
			.client
			.request(method, format!("{}{}", self.base_url, path));
		if self.username.is_empty() {
			request
		} else {
			request.basic_auth(&self.username, Some(&self.password))
		}
	}

	async fn get_alias_target(&self, alias: &str) -> Result<Option<String>> {
		let response = self
			.request(Method::GET, &format!("/_alias/{alias}"))
			.send()
			.await
			.wrap_err("failed to get Elasticsearch alias")?;
		if response.status() == StatusCode::NOT_FOUND {
			return Ok(None);
		}
		let body = response_json(response, "get Elasticsearch alias").await?;
		Ok(body
			.as_object()
			.and_then(|indices| indices.keys().next())
			.cloned())
	}

	async fn index_exists(&self, index: &str) -> Result<bool> {
		let response = self
			.request(Method::HEAD, &format!("/{index}"))
			.send()
			.await
			.wrap_err("failed to check Elasticsearch index existence")?;
		Ok(response.status().is_success())
	}

	async fn create_index(&self, index: &str, schema: &Value) -> Result<()> {
		let response = self
			.request(Method::PUT, &format!("/{index}"))
			.json(schema)
			.send()
			.await
			.wrap_err("failed to create Elasticsearch index")?;
		response_json(response, "create Elasticsearch index").await?;
		Ok(())
	}

	async fn delete_index_if_exists(&self, index: &str) -> Result<()> {
		let response = self
			.request(Method::DELETE, &format!("/{index}"))
			.send()
			.await
			.wrap_err("failed to delete Elasticsearch index")?;
		if response.status() == StatusCode::NOT_FOUND {
			return Ok(());
		}
		response_json(response, "delete Elasticsearch index").await?;
		Ok(())
	}

	async fn swap_alias(
		&self,
		alias: &str,
		old_index: Option<&str>,
		new_index: &str,
	) -> Result<()> {
		let mut actions = Vec::new();
		if let Some(old_index) = old_index {
			actions.push(json!({
				"remove": {"index": old_index, "alias": alias}
			}));
		}
		actions.push(json!({
			"add": {
				"index": new_index,
				"alias": alias,
				"is_write_index": true
			}
		}));

		let response = self
			.request(Method::POST, "/_aliases")
			.json(&json!({"actions": actions}))
			.send()
			.await
			.wrap_err("failed to swap Elasticsearch alias")?;
		response_json(response, "swap Elasticsearch alias").await?;
		Ok(())
	}

	async fn bulk(&self, index: &str, body: String) -> Result<()> {
		let response = self
			.request(
				Method::POST,
				&format!("/{index}/_bulk?refresh=false"),
			)
			.header("Content-Type", "application/x-ndjson")
			.body(body)
			.send()
			.await
			.wrap_err("failed to execute Elasticsearch bulk request")?;
		let body =
			response_json(response, "execute Elasticsearch bulk request")
				.await?;
		if body["errors"].as_bool() == Some(true) {
			let failures = body["items"]
				.as_array()
				.into_iter()
				.flatten()
				.filter_map(|item| {
					item.as_object()?
						.values()
						.next()?
						.get("error")
						.cloned()
				})
				.unique()
				.take(10)
				.map(|error| error.to_string())
				.join("; ");
			return Err(eyre!(
				"Elasticsearch bulk request contained failures: {failures}"
			));
		}
		Ok(())
	}

	async fn delete_by_query(
		&self,
		index: &str,
		query: &Value,
	) -> Result<()> {
		let response = self
			.request(
				Method::POST,
				&format!(
					"/{index}/_delete_by_query?conflicts=proceed&refresh=true"
				),
			)
			.json(&json!({"query": query}))
			.send()
			.await
			.wrap_err("failed to delete Elasticsearch documents")?;
		let body =
			response_json(response, "delete Elasticsearch documents").await?;
		if body["failures"]
			.as_array()
			.is_some_and(|failures| !failures.is_empty())
		{
			return Err(eyre!(
				"Elasticsearch delete-by-query contained failures: {}",
				body["failures"]
			));
		}
		Ok(())
	}

	async fn refresh(&self, index: &str) -> Result<()> {
		let response = self
			.request(Method::POST, &format!("/{index}/_refresh"))
			.send()
			.await
			.wrap_err("failed to refresh Elasticsearch index")?;
		response_json(response, "refresh Elasticsearch index").await?;
		Ok(())
	}
}

async fn response_json(
	response: Response,
	operation: &str,
) -> Result<Value> {
	let status = response.status();
	let body = response
		.text()
		.await
		.wrap_err_with(|| format!("failed to read response for {operation}"))?;
	let json = serde_json::from_str(&body).unwrap_or_else(|_| {
		json!({
			"unparsed_response": body
		})
	});
	if !status.is_success() {
		return Err(eyre!("{operation} failed ({status}): {json}"));
	}
	Ok(json)
}

pub struct Elasticsearch {
	pub config: ElasticsearchConfig,
	client: ElasticsearchClient,
}

impl Elasticsearch {
	pub fn new(config: ElasticsearchConfig) -> Self {
		let client = ElasticsearchClient::new(&config);
		Self { config, client }
	}

	fn index_schema() -> Value {
		json!({
			"settings": {
				"number_of_shards": 3,
				"number_of_replicas": 1,
				"index.mapping.total_fields.limit": 5000
			},
			"mappings": {
				"dynamic_templates": [
					{
						"strings_as_keywords": {
							"match_mapping_type": "string",
							"mapping": {
								"type": "keyword",
								"ignore_above": 8191
							}
						}
					}
				],
				"properties": {
					"document_type": {
						"type": "join",
						"relations": {"project": "version"}
					},
					"version_id": {"type": "keyword"},
					"project_id": {"type": "keyword"},
					"project_types": {"type": "keyword"},
					"all_project_types": {"type": "keyword"},
					"slug": {
						"type": "text",
						"fields": {
							"keyword": {"type": "keyword", "ignore_above": 8191}
						}
					},
					"author": {
						"type": "text",
						"fields": {
							"keyword": {"type": "keyword", "ignore_above": 8191}
						}
					},
					"indexed_author": {"type": "text"},
					"name": {
						"type": "text",
						"fields": {
							"keyword": {"type": "keyword", "ignore_above": 8191}
						}
					},
					"indexed_name": {"type": "text"},
					"summary": {
						"type": "text",
						"fields": {
							"keyword": {"type": "keyword", "ignore_above": 8191}
						}
					},
					"categories": {"type": "keyword"},
					"project_categories": {"type": "keyword"},
					"display_categories": {"type": "keyword"},
					"license": {"type": "keyword"},
					"open_source": {"type": "boolean"},
					"environment": {"type": "keyword"},
					"game_versions": {"type": "keyword"},
					"client_side": {"type": "keyword"},
					"server_side": {"type": "keyword"},
					"dependency_project_ids": {"type": "keyword"},
					"compatible_dependency_project_ids": {"type": "keyword"},
					"downloads": {"type": "integer"},
					"log_downloads": {"type": "double"},
					"follows": {"type": "integer"},
					"created_timestamp": {"type": "long"},
					"modified_timestamp": {"type": "long"},
					"version_published_timestamp": {"type": "long"},
					"date_created": {"type": "date"},
					"date_modified": {"type": "date"},
					"project_loader_fields": {"type": "object", "enabled": false},
					"minecraft_java_server": {
						"properties": {
							"verified_plays_2w": {"type": "long"},
							"is_online": {"type": "boolean"},
							"ping": {
								"properties": {
									"data": {
										"properties": {
											"players_online": {"type": "integer"}
										}
									}
								}
							}
						}
					}
				}
			}
		})
	}

	fn text_query(query: &str) -> Value {
		if query.is_empty() {
			return json!({"match_all": {}});
		}

		let fields = [
			"name^15",
			"indexed_name^15",
			"slug^10",
			"author^3",
			"indexed_author^3",
			"summary",
		];
		json!({
			"bool": {
				"should": [
					{
						"multi_match": {
							"query": query,
							"fields": fields,
							"type": "best_fields",
							"operator": "and",
							"fuzziness": "AUTO",
							"prefix_length": 1
						}
					},
					{
						"multi_match": {
							"query": query,
							"fields": fields,
							"type": "phrase_prefix"
						}
					}
				],
				"minimum_should_match": 1
			}
		})
	}

	fn sort(index: SearchIndex) -> Vec<Value> {
		let descending = |field: &str| {
			json!({(field): {"order": "desc", "missing": "_last"}})
		};
		let mut sort = match index {
			SearchIndex::Relevance => vec![
				json!({"_score": {"order": "desc"}}),
				descending("log_downloads"),
				descending("version_published_timestamp"),
			],
			SearchIndex::Downloads => vec![
				descending("log_downloads"),
				descending("version_published_timestamp"),
			],
			SearchIndex::Follows => vec![
				descending("follows"),
				descending("version_published_timestamp"),
			],
			SearchIndex::Updated => vec![
				descending("modified_timestamp"),
				descending("version_published_timestamp"),
			],
			SearchIndex::Newest => vec![
				descending("created_timestamp"),
				descending("version_published_timestamp"),
			],
			SearchIndex::MinecraftJavaServerVerifiedPlays2w => vec![
				json!({"_score": {"order": "desc"}}),
				descending("minecraft_java_server.verified_plays_2w"),
				descending("minecraft_java_server.is_online"),
			],
			SearchIndex::MinecraftJavaServerPlayersOnline => vec![
				json!({"_score": {"order": "desc"}}),
				descending("minecraft_java_server.is_online"),
				descending(
					"minecraft_java_server.ping.data.players_online",
				),
			],
		};
		sort.push(json!({"project_id": {"order": "asc"}}));
		sort
	}

	fn build_filter(
		info: &SearchRequest,
	) -> Result<Option<ElasticsearchFilter>, ApiError> {
		let facet_part = if let Some(facets_json) = info.facets.as_deref() {
			from_legacy_v2_facets_json(facets_json)
				.wrap_request_err("failed to parse facets")?
		} else {
			None
		};

		let filter_part = combined_search_filters(info)
			.filter(|filter| !filter.trim().is_empty())
			.map(|filter| parse_expression(&filter))
			.transpose()
			.wrap_request_err("failed to parse filters")?;

		FilterExpr::and([facet_part, filter_part].into_iter().flatten())
			.map(normalize)
			.map(|filter| {
				serialize_filter(&filter)
					.wrap_request_err("failed to build search filter")
			})
			.transpose()
	}

	async fn existing_write_indices(&self) -> Result<Vec<String>> {
		let alias = self.config.alias_name();
		let mut indices = self
			.client
			.get_alias_target(&alias)
			.await?
			.into_iter()
			.collect_vec();

		for index in [
			self.config.next_index_name(&alias, false),
			self.config.next_index_name(&alias, true),
		] {
			if !indices.contains(&index)
				&& self.client.index_exists(&index).await?
			{
				indices.push(index);
			}
		}
		Ok(indices)
	}

	async fn import_projects(
		&self,
		indices: &[String],
		documents: &[UploadSearchProject],
	) -> Result<()> {
		let batch_size = self.config.bulk_batch_size.max(1);
		for documents in documents.chunks(batch_size) {
			let body = projects_to_bulk(documents)?;
			for index in indices {
				info!(
					index,
					document_count = documents.len(),
					content_length_bytes = body.len(),
					"sending Elasticsearch project bulk request"
				);
				self.client.bulk(index, body.clone()).await?;
			}
		}
		Ok(())
	}

	async fn import_versions(
		&self,
		indices: &[String],
		documents: &[UploadSearchVersion],
	) -> Result<()> {
		let batch_size = self.config.bulk_batch_size.max(1);
		for documents in documents.chunks(batch_size) {
			let body = versions_to_bulk(documents)?;
			for index in indices {
				info!(
					index,
					document_count = documents.len(),
					content_length_bytes = body.len(),
					"sending Elasticsearch version bulk request"
				);
				self.client.bulk(index, body.clone()).await?;
			}
		}
		Ok(())
	}

	async fn delete_ids(
		&self,
		field: &str,
		ids: &[String],
	) -> Result<()> {
		let indices = self.existing_write_indices().await?;
		for ids in ids.chunks(DELETE_FILTER_ID_BATCH_SIZE) {
			let query = json!({"terms": {(field): ids}});
			for index in &indices {
				self.client.delete_by_query(index, &query).await?;
			}
		}
		Ok(())
	}

	async fn refresh_write_indices(&self) -> Result<()> {
		for index in self.existing_write_indices().await? {
			self.client.refresh(&index).await?;
		}
		Ok(())
	}
}

#[async_trait]
impl SearchBackend for Elasticsearch {
	async fn search_for_project_raw(
		&self,
		info: &SearchRequest,
	) -> Result<SearchResults, ApiError> {
		let parsed = parse_search_request(info)?;
		let search_sort =
			parse_search_index(parsed.index, info.new_filters.as_deref())?;
		let filter = Self::build_filter(info)?;

		let mut filters =
			vec![json!({"term": {"document_type": "project"}})];
		if let Some(filter) = &filter {
			filters.push(filter.query.clone());
		}
		let query = json!({
			"bool": {
				"must": [Self::text_query(parsed.query)],
				"filter": filters
			}
		});
		let body = json!({
			"from": parsed.offset,
			"size": parsed.hits_per_page,
			"track_total_hits": true,
			"query": query,
			"sort": Self::sort(search_sort.index)
		});

		let alias = self.config.alias_name();
		let response = self
			.client
			.request(Method::POST, &format!("/{alias}/_search"))
			.json(&body)
			.send()
			.await
			.wrap_internal_err("failed to execute Elasticsearch search")?;
		let body = response_json(response, "execute Elasticsearch search")
			.await
			.map_err(ApiError::Internal)?;

		let total_hits = body["hits"]["total"]["value"]
			.as_u64()
			.unwrap_or_default() as usize;
		let hits = body["hits"]["hits"]
			.as_array()
			.into_iter()
			.flatten()
			.filter_map(|hit| {
				let mut document = hit["_source"].clone();
				let object = document.as_object_mut()?;
				object.remove("document_type");
				if filter
					.as_ref()
					.is_some_and(|filter| filter.has_version_filter)
				{
					if let Some(version_id) = matching_version_id(hit) {
						object.insert(
							"version_id".to_string(),
							Value::String(version_id),
						);
					}
				}

				let metadata = info.show_metadata.then(|| {
					json!({
						"score": hit["_score"],
						"sort": hit["sort"]
					})
				});
				let mut result: ResultSearchProject =
					serde_json::from_value::<UploadSearchProject>(document)
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
	) -> Result<()> {
		info!("starting Elasticsearch project indexing");
		let alias = self.config.alias_name();
		let current = self.client.get_alias_target(&alias).await?;
		let use_alt =
			!current.as_deref().is_some_and(|name| name.ends_with("__alt"));
		let next = self.config.next_index_name(&alias, use_alt);

		info!(index = next, "creating Elasticsearch shadow index");
		self.client.delete_index_if_exists(&next).await?;
		self.client
			.create_index(&next, &Self::index_schema())
			.await?;

		let mut cursor = 0_i64;
		let mut chunk_index = 0_usize;
		let mut total_projects = 0_usize;
		let mut total_versions = 0_usize;

		loop {
			info!("fetching index chunk {chunk_index}");
			chunk_index += 1;
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
					"no more documents; indexed {total_projects} projects and {total_versions} versions in {chunk_index} chunks"
				);
				break;
			}

			total_projects += documents.projects.len();
			total_versions += documents.versions.len();
			cursor = next_cursor;
			self.import_projects(
				std::slice::from_ref(&next),
				&documents.projects,
			)
			.await?;
			self.import_versions(
				std::slice::from_ref(&next),
				&documents.versions,
			)
			.await?;
		}

		self.client.refresh(&next).await?;
		info!("swapping Elasticsearch index alias");
		self.client
			.swap_alias(&alias, current.as_deref(), &next)
			.await?;
		if let Some(old) = current {
			self.client.delete_index_if_exists(&old).await?;
		}
		info!("Elasticsearch indexing complete");
		Ok(())
	}

	async fn apply_update(
		&self,
		update: SearchIndexUpdate<'_>,
	) -> Result<()> {
		let removed_project_ids = update
			.removed_projects
			.iter()
			.map(ToString::to_string)
			.collect::<Vec<_>>();
		if !removed_project_ids.is_empty() {
			self.delete_ids("project_id", &removed_project_ids).await?;
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
			self.delete_ids("version_id", &version_ids).await?;
		}

		let indices = self.existing_write_indices().await?;
		if !update.projects.is_empty() {
			debug!(
				?indices,
				num_documents = update.projects.len(),
				"replacing Elasticsearch project documents"
			);
			self.import_projects(&indices, update.projects).await?;
		}
		if !update.versions.is_empty() {
			debug!(
				?indices,
				num_documents = update.versions.len(),
				"replacing Elasticsearch version documents"
			);
			self.import_versions(&indices, update.versions).await?;
		}
		self.refresh_write_indices().await?;
		debug!("done applying Elasticsearch search index update");
		Ok(())
	}

	async fn tasks(&self) -> Result<Value> {
		let response = self
			.client
			.request(Method::GET, "/_tasks?detailed=true")
			.send()
			.await
			.wrap_err("failed to get Elasticsearch tasks")?;
		response_json(response, "get Elasticsearch tasks").await
	}

	async fn tasks_cancel(&self, filter: &TasksCancelFilter) -> Result<()> {
		match filter {
			TasksCancelFilter::All => {
				let response = self
					.client
					.request(Method::POST, "/_tasks/_cancel")
					.send()
					.await
					.wrap_err("failed to cancel Elasticsearch tasks")?;
				response_json(response, "cancel Elasticsearch tasks").await?;
			}
			TasksCancelFilter::AllEnqueued => {
				// Elasticsearch executes operations immediately and does not
				// expose an enqueued-task state.
			}
			TasksCancelFilter::Indexes { indexes } => {
				let tasks = self.tasks().await?;
				for (_node_id, node) in tasks["nodes"]
					.as_object()
					.into_iter()
					.flatten()
				{
					for (task_id, task) in node["tasks"]
						.as_object()
						.into_iter()
						.flatten()
					{
						let description =
							task["description"].as_str().unwrap_or_default();
						if indexes
							.iter()
							.any(|index| description.contains(index))
						{
							let response = self
								.client
								.request(
									Method::POST,
									&format!(
										"/_tasks/{task_id}/_cancel"
									),
								)
								.send()
								.await
								.wrap_err(
									"failed to cancel Elasticsearch task",
								)?;
							response_json(
								response,
								"cancel Elasticsearch task",
							)
							.await?;
						}
					}
				}
			}
		}
		Ok(())
	}
}

fn matching_version_id(hit: &Value) -> Option<String> {
	hit["inner_hits"]
		.as_object()?
		.values()
		.filter_map(|inner_hits| {
			let source = inner_hits["hits"]["hits"]
				.as_array()?
				.first()?
				.get("_source")?;
			Some((
				source["version_published_timestamp"].as_i64()?,
				source["version_id"].as_str()?.to_string(),
			))
		})
		.max_by_key(|(published, _)| *published)
		.map(|(_, version_id)| version_id)
}

fn projects_to_bulk(documents: &[UploadSearchProject]) -> Result<String> {
	let mut output = String::new();
	for document in documents {
		let id = format!("project:{}", document.project_id);
		push_json_line(
			&mut output,
			&json!({
				"index": {
					"_id": id,
					"routing": document.project_id
				}
			}),
		)?;

		let mut source = serde_json::to_value(document)
			.wrap_err("failed to serialize `UploadSearchProject`")?;
		let object = source
			.as_object_mut()
			.ok_or_else(|| eyre!("project search document is not an object"))?;
		object.insert(
			"document_type".to_string(),
			Value::String("project".to_string()),
		);
		add_server_online_field(object);
		push_json_line(&mut output, &source)?;
	}
	Ok(output)
}

fn versions_to_bulk(documents: &[UploadSearchVersion]) -> Result<String> {
	let mut output = String::new();
	for document in documents {
		let id = format!("version:{}", document.version_id);
		push_json_line(
			&mut output,
			&json!({
				"index": {
					"_id": id,
					"routing": document.project_id
				}
			}),
		)?;

		let mut source = serde_json::to_value(document)
			.wrap_err("failed to serialize `UploadSearchVersion`")?;
		source
			.as_object_mut()
			.ok_or_else(|| eyre!("version search document is not an object"))?
			.insert(
				"document_type".to_string(),
				json!({
					"name": "version",
					"parent": format!("project:{}", document.project_id)
				}),
			);
		push_json_line(&mut output, &source)?;
	}
	Ok(output)
}

fn add_server_online_field(object: &mut Map<String, Value>) {
	let Some(server) = object
		.get_mut("minecraft_java_server")
		.and_then(Value::as_object_mut)
	else {
		return;
	};
	let is_online = server
		.get("ping")
		.and_then(Value::as_object)
		.and_then(|ping| ping.get("data"))
		.is_some_and(|data| !data.is_null());
	server.insert("is_online".to_string(), Value::Bool(is_online));
}

fn push_json_line<T: Serialize>(
	output: &mut String,
	value: &T,
) -> Result<()> {
	output.push_str(&serde_json::to_string(value)?);
	output.push('\n');
	Ok(())
}
