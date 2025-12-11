mod error;
pub use error::*;
use {
    reqwest::{IntoUrl, RequestBuilder},
    secrecy::{ExposeSecret, SecretString},
};

#[cfg(feature = "mock")]
mod mock;
#[cfg(feature = "mock")]
pub use mock::MuralPayMock;
use serde::de::DeserializeOwned;

#[derive(Debug, Clone)]
pub struct Client {
    pub http: reqwest::Client,
    pub api_url: String,
    pub api_key: SecretString,
    pub transfer_api_key: SecretString,
    #[cfg(feature = "mock")]
    pub mock: std::sync::Arc<arc_swap::ArcSwapOption<mock::MuralPayMock>>,
}

impl Client {
    pub fn new(
        api_url: impl Into<String>,
        api_key: impl Into<SecretString>,
        transfer_api_key: impl Into<SecretString>,
    ) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_url: api_url.into(),
            api_key: api_key.into(),
            transfer_api_key: transfer_api_key.into(),
            #[cfg(feature = "mock")]
            mock: std::sync::Arc::new(arc_swap::ArcSwapOption::empty()),
        }
    }

    /// Creates a client which mocks responses.
    #[cfg(feature = "mock")]
    #[must_use]
    pub fn from_mock(mock: mock::MuralPayMock) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_url: String::new(),
            api_key: SecretString::from(String::new()),
            transfer_api_key: SecretString::from(String::new()),
            mock: std::sync::Arc::new(arc_swap::ArcSwapOption::from_pointee(mock)),
        }
    }

    fn http_req(&self, make_req: impl FnOnce() -> RequestBuilder) -> RequestBuilder {
        make_req()
            .bearer_auth(self.api_key.expose_secret())
            .header("accept", "application/json")
            .header("content-type", "application/json")
    }

    pub(crate) fn http_get<U: IntoUrl>(&self, make_url: impl FnOnce(&str) -> U) -> RequestBuilder {
        self.http_req(|| self.http.get(make_url(&self.api_url)))
    }

    pub(crate) fn http_post<U: IntoUrl>(&self, make_url: impl FnOnce(&str) -> U) -> RequestBuilder {
        self.http_req(|| self.http.post(make_url(&self.api_url)))
    }

    pub(crate) fn http_put<U: IntoUrl>(&self, make_url: impl FnOnce(&str) -> U) -> RequestBuilder {
        self.http_req(|| self.http.put(make_url(&self.api_url)))
    }

    pub(crate) fn http_delete<U: IntoUrl>(
        &self,
        make_url: impl FnOnce(&str) -> U,
    ) -> RequestBuilder {
        self.http_req(|| self.http.delete(make_url(&self.api_url)))
    }

    pub async fn health(&self) -> reqwest::Result<()> {
        self.http_get(|base| format!("{base}/api/health"))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

pub trait RequestExt: Sized {
    #[must_use]
    fn transfer_auth(self, client: &Client) -> Self;

    fn send_mural<T: DeserializeOwned>(
        self,
    ) -> impl Future<Output = crate::Result<T>> + Send + Sync;
}

const HEADER_TRANSFER_API_KEY: &str = "transfer-api-key";

impl RequestExt for reqwest::RequestBuilder {
    fn transfer_auth(self, client: &Client) -> Self {
        self.header(
            HEADER_TRANSFER_API_KEY,
            client.transfer_api_key.expose_secret(),
        )
    }

    async fn send_mural<T: DeserializeOwned>(self) -> crate::Result<T> {
        let resp = self.send().await?;
        let status = resp.status();
        if status.is_client_error() || status.is_server_error() {
            let json = resp.bytes().await?;
            let err = serde_json::from_slice::<ApiError>(&json)
                .map_err(|source| MuralError::DecodeError { source, json })?;
            Err(MuralError::Api(err))
        } else {
            let json = resp.bytes().await?;
            let t = serde_json::from_slice::<T>(&json)
                .map_err(|source| MuralError::Decode { source, json })?;
            Ok(t)
        }
    }
}
