use crate::models::projects::SearchRequest;
use crate::routes::ApiError;
use crate::search::backend::{SearchBackend, SearchResults};
use crate::util::error::Context;
use async_trait::async_trait;
use elasticsearch::{
    http::Url,
    http::transport::{SingleNodeConnectionPool, TransportBuilder},
};
use eyre::eyre;
use tracing::{error, info};

pub struct ElasticsearchConfig {
    pub url: String,
    pub index_prefix: String,
}

impl ElasticsearchConfig {
    pub fn from_env() -> eyre::Result<Self> {
        Ok(Self {
            url: dotenvy::var("ELASTICSEARCH_URL")
                .unwrap_or_else(|_| "http://localhost:9200".to_string()),
            index_prefix: dotenvy::var("ELASTICSEARCH_INDEX_PREFIX")
                .unwrap_or_else(|_| "labrinth".to_string()),
        })
    }
}

pub struct Elasticsearch {
    pub config: ElasticsearchConfig,
    pub client: elasticsearch::Elasticsearch,
}

impl Elasticsearch {
    pub fn new(config: ElasticsearchConfig) -> eyre::Result<Self> {
        info!("Creating Elasticsearch backend with URL: {}", config.url);

        let url = Url::parse(&config.url)
            .wrap_err("failed to parse Elasticsearch URL")?;

        let transport =
            TransportBuilder::new(SingleNodeConnectionPool::new(url))
                .build()
                .wrap_err("failed to create Elasticsearch transport")?;

        let client = elasticsearch::Elasticsearch::new(transport);
        info!("Elasticsearch client created successfully");
        Ok(Self { config, client })
    }

    async fn test_connection(&self) -> Result<(), ApiError> {
        info!("Testing Elasticsearch connection...");

        let response = self.client.ping().send().await.map_err(|e| {
            error!("Elasticsearch ping failed: {:?}", e);
            ApiError::Internal(eyre!("failed to ping Elasticsearch: {}", e))
        })?;

        let status = response.status_code();
        let status_code = status.as_u16();
        if (200..300).contains(&status_code) {
            info!("Elasticsearch connection test successful!");
            Ok(())
        } else {
            error!(
                "Elasticsearch connection test failed. Status: {:?}",
                status
            );
            Err(ApiError::Internal(eyre!(
                "failed to connect to Elasticsearch. Status: {:?}",
                status
            )))
        }
    }

    #[allow(dead_code)]
    fn get_index_name(&self, index: &str) -> String {
        format!("{}_{}", self.config.index_prefix, index)
    }
}

#[async_trait]
impl SearchBackend for Elasticsearch {
    async fn search_for_project(
        &self,
        _info: &SearchRequest,
    ) -> Result<SearchResults, ApiError> {
        Err(ApiError::Internal(eyre!("failed to search Elasticsearch")))
    }

    async fn index_projects(
        &self,
        _ro_pool: crate::database::PgPool,
        _redis: crate::database::redis::RedisPool,
    ) -> Result<(), ApiError> {
        Err(ApiError::Internal(eyre!("failed to index Elasticsearch")))
    }

    async fn remove_documents(
        &self,
        _ids: &[crate::models::ids::VersionId],
    ) -> Result<(), ApiError> {
        Err(ApiError::Internal(eyre!(
            "failed to remove Elasticsearch documents"
        )))
    }
}
