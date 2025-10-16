use reqwest::header::HeaderName;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::routes::ApiError;

const X_MASTER_KEY: HeaderName = HeaderName::from_static("x-master-key");

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Empty {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Specs {
    pub memory_mb: u32,
    pub cpu: u32,
    pub swap_mb: u32,
    pub storage_mb: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateServerRequest {
    pub user_id: String,
    pub name: String,
    pub specs: Specs,
    // Must be included because archon doesn't accept null values, only
    // an empty struct, as a source.
    pub source: Empty,
    pub region: String,
    pub tags: Vec<String>,
}

#[derive(Clone)]
pub struct ArchonClient {
    client: reqwest::Client,
    base_url: String,
    pyro_api_key: String,
}

impl ArchonClient {
    /// Builds an Archon client from environment variables. Returns `None` if the
    /// required environment variables are not set.
    pub fn from_env() -> Result<Self, ApiError> {
        let client = reqwest::Client::new();

        let base_url =
            dotenvy::var("ARCHON_URL")?.trim_end_matches('/').to_owned();

        Ok(Self {
            client,
            base_url,
            pyro_api_key: dotenvy::var("PYRO_API_KEY")?,
        })
    }

    pub async fn create_server(
        &self,
        request: &CreateServerRequest,
    ) -> Result<Uuid, reqwest::Error> {
        #[derive(Deserialize)]
        struct CreateServerResponse {
            uuid: Uuid,
        }

        let response = self
            .client
            .post(format!("{}/modrinth/v0/servers/create", self.base_url))
            .header(X_MASTER_KEY, &self.pyro_api_key)
            .json(request)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json::<CreateServerResponse>().await?.uuid)
    }
}
