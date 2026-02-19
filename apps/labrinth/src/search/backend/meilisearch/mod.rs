use crate::database::PgPool;
use crate::database::redis::RedisPool;
use crate::env::ENV;
use crate::models::ids::VersionId;
use crate::models::projects::SearchRequest;
use crate::routes::ApiError;
use crate::search::{
    ResultSearchProject, SearchBackend, SearchResults, TasksCancelFilter,
};
use crate::util::error::Context;
use async_trait::async_trait;
use eyre::eyre;
use futures::TryStreamExt;
use futures::stream::FuturesOrdered;
use itertools::Itertools;
use meilisearch_sdk::client::Client;
use meilisearch_sdk::tasks::{Task, TasksCancelQuery};
use serde::Serialize;
use serde_json::Value;
use std::borrow::Cow;
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
    ) -> Result<Vec<T>, indexing::IndexingError>
    where
        G: Fn(&'a Client) -> Fut,
        Fut: Future<Output = Result<T, indexing::IndexingError>> + 'a,
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
    ) -> Result<(String, &'static [&'static str]), ApiError> {
        let projects_name = self.config.get_index_name("projects", false);
        let projects_filtered_name =
            self.config.get_index_name("projects_filtered", false);
        Ok(match index {
            "relevance" => (projects_name, &["downloads:desc"]),
            "downloads" => (projects_filtered_name, &["downloads:desc"]),
            "follows" => (projects_name, &["follows:desc"]),
            "updated" => (projects_name, &["date_modified:desc"]),
            "newest" => (projects_name, &["date_created:desc"]),
            _ => {
                return Err(ApiError::Request(eyre!(
                    "invalid index `{index}`"
                )));
            }
        })
    }
}

#[async_trait]
impl SearchBackend for Meilisearch {
    async fn search_for_project(
        &self,
        info: &SearchRequest,
    ) -> Result<SearchResults, ApiError> {
        let offset: usize = info
            .offset
            .as_deref()
            .unwrap_or("0")
            .parse()
            .wrap_request_err("invalid offset")?;
        let index = info.index.as_deref().unwrap_or("relevance");
        let limit = info
            .limit
            .as_deref()
            .unwrap_or("10")
            .parse::<usize>()
            .wrap_request_err("invalid limit")?
            .min(100);

        let (index_name, sort_name) = self.get_sort_index(index)?;
        let client = self
            .config
            .make_loadbalanced_read_client()
            .wrap_internal_err("failed to make load-balanced read client")?;
        let meilisearch_index = client
            .get_index(index_name)
            .await
            .wrap_internal_err("failed to get index")?;

        let mut filter_string = String::new();

        let hits_per_page = if limit == 0 { 1 } else { limit };

        let page = offset / hits_per_page + 1;

        let results = {
            let mut query = meilisearch_index.search();
            query
                .with_page(page)
                .with_hits_per_page(hits_per_page)
                .with_query(info.query.as_deref().unwrap_or_default())
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

                let filters: Cow<_> =
                    match (info.filters.as_deref(), info.version.as_deref()) {
                        (Some(f), Some(v)) => format!("({f}) AND ({v})").into(),
                        (Some(f), None) => f.into(),
                        (None, Some(v)) => v.into(),
                        (None, None) => "".into(),
                    };

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

            query.execute::<ResultSearchProject>().await?
        };

        Ok(SearchResults {
            hits: results.hits.into_iter().map(|r| r.result).collect(),
            page: results.page.unwrap_or_default(),
            hits_per_page: results.hits_per_page.unwrap_or_default(),
            total_hits: results.total_hits.unwrap_or_default(),
        })
    }

    async fn index_projects(
        &self,
        ro_pool: PgPool,
        redis: RedisPool,
    ) -> Result<(), ApiError> {
        indexing::index_projects(ro_pool, redis, &self.config)
            .await
            .map_err(|e| ApiError::Internal(e.into()))
    }

    async fn remove_documents(
        &self,
        ids: &[VersionId],
    ) -> Result<(), ApiError> {
        indexing::remove_documents(ids, &self.config)
            .await
            .map_err(|e| ApiError::Internal(e.into()))
    }

    async fn tasks(&self) -> Result<Value, ApiError> {
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

        serde_json::to_value(TaskList {
            by_instance: response,
        })
        .wrap_internal_err("failed to serialize tasks response")
    }

    async fn tasks_cancel(
        &self,
        filter: &TasksCancelFilter,
    ) -> Result<(), ApiError> {
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
