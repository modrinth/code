use anyhow::{Context, Result};
use temporalio_client::{Client, ClientOptions, ConnectionOptions};

use crate::env::ENV;

#[derive(Debug)]
pub struct TemporalClient {
    pub client: Client,
}

impl TemporalClient {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn from_env() -> Result<Self> {
        let client = Client::connect(
            ConnectionOptions::new(ENV.TEMPORAL_ADDRESS.clone()).build(),
            ClientOptions::new(ENV.TEMPORAL_NAMESPACE.clone()).build(),
        )
        .await
        .context("connecting to temporal")?;

        Ok(Self::new(client))
    }
}
