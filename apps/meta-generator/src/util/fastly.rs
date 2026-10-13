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
}

impl Client {
    pub fn new(
        http: reqwest::Client,
        base: impl IntoUrl,
        fastly_key: SecretString,
    ) -> Result<Self> {
        let base = base.into_url().context("invalid base URL")?;
        let mut fastly_key = HeaderValue::from_str(fastly_key.expose_secret())
            .context("invalid Fastly key")?;
        fastly_key.set_sensitive(true);
        Ok(Self {
            http,
            base,
            fastly_key,
        })
    }

    pub async fn purge_url(&self, cached_url: &Url) -> Result<()> {
        let mut endpoint = self.base.join("purge/")?;
        endpoint
            .path_segments_mut()
            .expect("API URL supports path segments")
            .pop_if_empty()
            .push(cached_url.as_str());

        self.http
            .post(endpoint)
            .header("Fastly-Key", &self.fastly_key)
            .send()
            .await?
            .error_for_status_ext()
            .await?;
        Ok(())
    }
}
