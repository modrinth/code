use anyhow::{Context, Result};
use derive_more::Debug;
use reqwest::{IntoUrl, header::HeaderValue};
use secrecy::{ExposeSecret, SecretString};
use url::Url;

use crate::util::ResponseExt;

#[derive(Debug)]
pub struct Client {
    http: reqwest::Client,
    base: Url,
    fastly_key: HeaderValue,
    service_id: String,
    surrogate_key: String,
}

#[bon::bon]
impl Client {
    #[builder]
    pub fn new(
        http: reqwest::Client,
        base: impl IntoUrl,
        fastly_key: SecretString,
        service_id: String,
        surrogate_key: String,
    ) -> Result<Self> {
        let base = base.into_url().context("invalid base URL")?;
        let mut fastly_key = HeaderValue::from_str(fastly_key.expose_secret())
            .context("invalid Fastly key")?;
        fastly_key.set_sensitive(true);
        Ok(Self {
            http,
            base,
            fastly_key,
            service_id,
            surrogate_key,
        })
    }

    pub async fn soft_purge_all(&self) -> Result<()> {
        let mut endpoint = self.base.join("service/")?;
        endpoint
            .path_segments_mut()
            .expect("URL should support path segments")
            .pop_if_empty()
            .push(&self.service_id)
            .push("purge")
            .push(&self.surrogate_key);
        self.http
            .post(endpoint)
            .header("Fastly-Key", &self.fastly_key)
            .header("Fastly-Soft-Purge", "1")
            .send()
            .await?
            .error_for_status_ext()
            .await?;
        Ok(())
    }
}
