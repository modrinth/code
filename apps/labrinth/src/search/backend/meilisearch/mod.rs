use crate::database::PgPool;
use crate::database::redis::RedisPool;
use crate::env::ENV;
use crate::models::ids::VersionId;
use crate::routes::ApiError;
use crate::search::backend::{
    SearchIndex, SearchIndexName, combined_search_filters, parse_search_index,
    parse_search_request,
};
use crate::search::{
    ResultSearchProject, SearchBackend, SearchRequest, SearchResults,
    TasksCancelFilter,
};
use crate::util::error::Context;
use async_trait::async_trait;
use eyre::Result;
use futures::TryStreamExt;
use futures::stream::FuturesOrdered;
use itertools::Itertools;
use meilisearch_sdk::client::Client;
use meilisearch_sdk::tasks::{Task, TasksCancelQuery};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Write;
use std::time::Duration;
use tracing::{Instrument, info_span};

pub mod indexing;

#[derive(Debug, Clone)]
pub struct MeilisearchReadClient {
    pub client: Client,
}

impl std::ops::Deref for MeilisearchReadClient {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

pub struct BatchClient {
    pub clients: Vec<Client>,
}

impl BatchClient {
    pub fn new(clients: Vec<Client>) -> Self {
        Self { clients }
    }

    pub async fn with_all_clients<'a, T, G, Fut>(
        &'a self,
        task_name: &str,
        generator: G,
    ) -> Result<Vec<T>>
    where
        G: Fn(&'a Client) -> Fut,
        Fut: Future<Output = Result<T>> + 'a,
    {
        let mut tasks = FuturesOrdered::new();
        for (idx, client) in self.clients.iter().enumerate() {
            tasks.push_back(generator(client).instrument(info_span!(
                "client_task",
                task.name = task_name,
                client.idx = idx,
            )));
        }

        let results = tasks.try_collect::<Vec<T>>().await?;
        Ok(results)
    }

    pub fn across_all<T, F, R>(&self, data: Vec<T>, mut predicate: F) -> Vec<R>
    where
        F: FnMut(T, &Client) -> R,
    {
        assert_eq!(
            data.len(),
            self.clients.len(),
            "mismatch between data len and meilisearch client count"
        );
        self.clients
            .iter()
            .zip(data)
            .map(|(client, item)| predicate(item, client))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct MeilisearchConfig {
    pub addresses: Vec<String>,
    pub read_lb_address: String,
    pub key: String,
    pub meta_namespace: String,
}

impl MeilisearchConfig {
    pub fn new(meta_namespace: Option<String>) -> Self {
        Self {
            addresses: ENV.MEILISEARCH_WRITE_ADDRS.0.clone(),
            key: ENV.MEILISEARCH_KEY.clone(),
            meta_namespace: meta_namespace.unwrap_or_default(),
            read_lb_address: ENV.MEILISEARCH_READ_ADDR.clone(),
        }
    }

    pub fn make_loadbalanced_read_client(
        &self,
    ) -> Result<MeilisearchReadClient, meilisearch_sdk::errors::Error> {
        Ok(MeilisearchReadClient {
            client: Client::new(&self.read_lb_address, Some(&self.key))?,
        })
    }

    pub fn make_batch_client(
        &self,
    ) -> Result<BatchClient, meilisearch_sdk::errors::Error> {
        Ok(BatchClient::new(
            self.addresses
                .iter()
                .map(|address| {
                    Client::new(address.as_str(), Some(self.key.as_str()))
                })
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }

    pub fn get_index_name(&self, index: &str, next: bool) -> String {
        let alt = if next { "_alt" } else { "" };
        format!("{}_{}_{}", self.meta_namespace, index, alt)
    }
}

pub struct Meilisearch {
    pub config: MeilisearchConfig,
}

impl Meilisearch {
    pub fn new(config: MeilisearchConfig) -> Self {
        Self { config }
    }

    fn get_sort_index(
        &self,
        index: &str,
        new_filters: Option<&str>,
    ) -> Result<(String, &'static [&'static str]), ApiError> {
        let sort = parse_search_index(index, new_filters)?;
        let index_name = match sort.index_name {
            SearchIndexName::Projects => {
                self.config.get_index_name("projects", false)
            }
            SearchIndexName::ProjectsFiltered => {
                self.config.get_index_name("projects_filtered", false)
            }
        };

        Ok(match sort.index {
            SearchIndex::Relevance => (
                index_name,
                &["downloads:desc", "version_published_timestamp:desc"],
            ),
            SearchIndex::Downloads => (
                index_name,
                &["downloads:desc", "version_published_timestamp:desc"],
            ),
            SearchIndex::Follows => (
                index_name,
                &["follows:desc", "version_published_timestamp:desc"],
            ),
            SearchIndex::Updated => (
                index_name,
                &["date_modified:desc", "version_published_timestamp:desc"],
            ),
            SearchIndex::Newest => (
                index_name,
                &["date_created:desc", "version_published_timestamp:desc"],
            ),
            SearchIndex::MinecraftJavaServerVerifiedPlays2w => (
                index_name,
                &[
                    "minecraft_java_server.verified_plays_2w:desc",
                    "minecraft_java_server.ping.data.players_online:desc",
                    "version_published_timestamp:desc",
                ],
            ),
            SearchIndex::MinecraftJavaServerPlayersOnline => (
                index_name,
                &[
                    "minecraft_java_server.ping.data.players_online:desc",
                    "version_published_timestamp:desc",
                ],
            ),
        })
    }
}

#[async_trait]
impl SearchBackend for Meilisearch {
    async fn search_for_project_raw(
        &self,
        info: &SearchRequest,
    ) -> Result<SearchResults, ApiError> {
        let parsed = parse_search_request(info)?;

        let (index_name, sort_name) =
            self.get_sort_index(parsed.index, info.new_filters.as_deref())?;
        let client = self
            .config
            .make_loadbalanced_read_client()
            .wrap_internal_err("failed to make load-balanced read client")?;
        let meilisearch_index = client
            .get_index(index_name)
            .await
            .wrap_internal_err("failed to get index")?;

        let mut filter_string = String::new();

        let results = {
            let mut query = meilisearch_index.search();
            query
                .with_page(parsed.page)
                .with_hits_per_page(parsed.hits_per_page)
                .with_query(parsed.query)
                .with_sort(sort_name);

            if let Some(new_filters) = info.new_filters.as_deref() {
                query.with_filter(new_filters);
            } else {
                let facets = if let Some(facets) = &info.facets {
                    let facets =
                        serde_json::from_str::<Vec<Vec<Value>>>(facets)
                            .wrap_request_err("failed to parse facets")?;
                    Some(facets)
                } else {
                    None
                };

                let filters =
                    combined_search_filters(info).unwrap_or_else(|| "".into());

                if let Some(facets) = facets {
                    let facets: Vec<Vec<Vec<String>>> =
                        facets
                            .into_iter()
                            .map(|facets| {
                                facets
								.into_iter()
								.map(|facet| {
									if facet.is_array() {
										serde_json::from_value::<Vec<String>>(facet)
											.unwrap_or_default()
									} else {
										vec![
											serde_json::from_value::<String>(facet)
												.unwrap_or_default(),
										]
									}
								})
								.collect_vec()
                            })
                            .collect_vec();

                    filter_string.push('(');
                    for (index, facet_outer_list) in facets.iter().enumerate() {
                        filter_string.push('(');

                        for (facet_outer_index, facet_inner_list) in
                            facet_outer_list.iter().enumerate()
                        {
                            filter_string.push('(');
                            for (facet_inner_index, facet) in
                                facet_inner_list.iter().enumerate()
                            {
                                filter_string
                                    .push_str(&facet.replace(':', " = "));
                                if facet_inner_index
                                    != (facet_inner_list.len() - 1)
                                {
                                    filter_string.push_str(" AND ")
                                }
                            }
                            filter_string.push(')');

                            if facet_outer_index != (facet_outer_list.len() - 1)
                            {
                                filter_string.push_str(" OR ")
                            }
                        }

                        filter_string.push(')');

                        if index != (facets.len() - 1) {
                            filter_string.push_str(" AND ")
                        }
                    }
                    filter_string.push(')');

                    if !filters.is_empty() {
                        write!(filter_string, " AND ({filters})")
                            .expect("write should not fail");
                    }
                } else {
                    filter_string.push_str(&filters);
                }

                if !filter_string.is_empty() {
                    query.with_filter(&filter_string);
                }
            }

            if info.show_metadata {
                query.with_show_ranking_score(true);
                query.with_show_ranking_score_details(true);
                query.execute().await?
            } else {
                query.execute::<ResultSearchProject>().await?
            }
        };

        if info.show_metadata {
            let hits = results
                .hits
                .into_iter()
                .map(|hit| {
                    let metadata = serde_json::to_value(&hit)
                        .ok()
                        .and_then(|value| value.as_object().cloned())
                        .map(|mut value| {
                            value.remove("_formatted");
                            value.remove("_matchesPosition");
                            value.remove("_federation");
                            let result = value.remove("result");
                            let metadata = Value::Object(value);
                            (result, metadata)
                        });

                    let (result, metadata) =
                        metadata.unwrap_or((None, Value::Null));
                    let mut result = result
                        .and_then(|value| {
                            serde_json::from_value::<ResultSearchProject>(value)
                                .ok()
                        })
                        .unwrap_or(hit.result);

                    if !metadata.is_null() {
                        result.search_metadata = Some(metadata);
                    }

                    result
                })
                .collect();

            Ok(SearchResults {
                hits,
                page: results.page.unwrap_or_default(),
                hits_per_page: results.hits_per_page.unwrap_or_default(),
                total_hits: results.total_hits.unwrap_or_default(),
            })
        } else {
            Ok(SearchResults {
                hits: results.hits.into_iter().map(|r| r.result).collect(),
                page: results.page.unwrap_or_default(),
                hits_per_page: results.hits_per_page.unwrap_or_default(),
                total_hits: results.total_hits.unwrap_or_default(),
            })
        }
    }

    async fn index_projects(
        &self,
        ro_pool: PgPool,
        redis: RedisPool,
    ) -> eyre::Result<()> {
        indexing::index_projects(ro_pool, redis, &self.config).await?;
        Ok(())
    }

    async fn remove_documents(&self, ids: &[VersionId]) -> eyre::Result<()> {
        indexing::remove_documents(ids, &self.config).await?;
        Ok(())
    }

    async fn tasks(&self) -> eyre::Result<Value> {
        let client = self
            .config
            .make_batch_client()
            .wrap_internal_err("failed to make batch client")?;
        let tasks = client
            .with_all_clients("get_tasks", async |client| {
                let tasks = client.get_tasks().await?;
                Ok(tasks.results)
            })
            .await
            .wrap_internal_err("failed to get tasks")?;

        #[derive(Serialize)]
        struct MeiliTask<Time> {
            uid: u32,
            status: &'static str,
            duration: Option<Duration>,
            enqueued_at: Option<Time>,
        }

        #[derive(Serialize)]
        struct TaskList<Time> {
            by_instance: HashMap<String, Vec<MeiliTask<Time>>>,
        }

        let response = tasks
            .into_iter()
            .enumerate()
            .map(|(idx, instance_tasks)| {
                let tasks = instance_tasks
                    .into_iter()
                    .filter_map(|task| {
                        Some(match task {
                            Task::Enqueued { content } => MeiliTask {
                                uid: content.uid,
                                status: "enqueued",
                                duration: None,
                                enqueued_at: Some(content.enqueued_at),
                            },
                            Task::Processing { content } => MeiliTask {
                                uid: content.uid,
                                status: "processing",
                                duration: None,
                                enqueued_at: Some(content.enqueued_at),
                            },
                            Task::Failed { content } => MeiliTask {
                                uid: content.task.uid,
                                status: "failed",
                                duration: Some(content.task.duration),
                                enqueued_at: Some(content.task.enqueued_at),
                            },
                            Task::Succeeded { .. } => return None,
                        })
                    })
                    .collect();

                (idx.to_string(), tasks)
            })
            .collect::<HashMap<String, Vec<MeiliTask<_>>>>();

        let response = serde_json::to_value(TaskList {
            by_instance: response,
        })
        .wrap_internal_err("failed to serialize tasks response")?;
        Ok(response)
    }

    async fn tasks_cancel(
        &self,
        filter: &TasksCancelFilter,
    ) -> eyre::Result<()> {
        let client = self
            .config
            .make_batch_client()
            .wrap_internal_err("failed to make batch client")?;
        let all_results = client
            .with_all_clients("cancel_tasks", async |client| {
                let mut q = TasksCancelQuery::new(client);
                match filter {
                    TasksCancelFilter::All => {}
                    TasksCancelFilter::Indexes { indexes } => {
                        q.with_index_uids(indexes.iter().map(|s| s.as_str()));
                    }
                    TasksCancelFilter::AllEnqueued => {
                        q.with_statuses(["enqueued"]);
                    }
                };

                let result = client.cancel_tasks_with(&q).await;
                Ok(result)
            })
            .await
            .wrap_internal_err("failed to cancel tasks")?;

        for r in all_results {
            r.wrap_internal_err("failed to cancel tasks")?;
        }

        Ok(())
    }
}
