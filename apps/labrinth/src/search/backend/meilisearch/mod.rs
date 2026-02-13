use crate::models::projects::SearchRequest;
use crate::routes::ApiError;
use crate::search::backend::{
    ResultSearchProject, SearchBackend, SearchResults,
};
use crate::util::error::Context;
use async_trait::async_trait;
use eyre::eyre;
use futures::TryStreamExt;
use futures::stream::FuturesOrdered;
use itertools::Itertools;
use meilisearch_sdk::client::Client;
use serde_json::Value;
use std::borrow::Cow;
use std::fmt::Write;
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
pub struct SearchConfig {
    pub addresses: Vec<String>,
    pub read_lb_address: String,
    pub key: String,
    pub meta_namespace: String,
}

impl SearchConfig {
    pub fn new(meta_namespace: Option<String>) -> Self {
        let address_many = dotenvy::var("MEILISEARCH_WRITE_ADDRS")
            .expect("MEILISEARCH_WRITE_ADDRS not set");

        let read_lb_address = dotenvy::var("MEILISEARCH_READ_ADDR")
            .expect("MEILISEARCH_READ_ADDR not set");

        let addresses = address_many
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let key =
            dotenvy::var("MEILISEARCH_KEY").expect("MEILISEARCH_KEY not set");

        Self {
            addresses,
            key,
            meta_namespace: meta_namespace.unwrap_or_default(),
            read_lb_address,
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

pub struct MeilisearchBackend {
    pub config: SearchConfig,
}

impl MeilisearchBackend {
    pub fn new(config: SearchConfig) -> Self {
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
impl SearchBackend for MeilisearchBackend {
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
}
