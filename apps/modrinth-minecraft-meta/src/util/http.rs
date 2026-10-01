use anyhow::{Context, Result, anyhow};
use reqwest::Response;
use serde::de::DeserializeOwned;

use crate::util::json_from_value;

pub trait ResponseExt: Sized {
    async fn error_for_status_ext(self) -> Result<Self>;

    async fn json_ext<T: DeserializeOwned>(self) -> Result<T>;
}

impl ResponseExt for Response {
    async fn error_for_status_ext(self) -> Result<Self> {
        let status = self.status();
        if status.is_client_error() || status.is_server_error() {
            let text = self
                .text()
                .await
                .unwrap_or_else(|_| "(non-UTF-8 response)".to_string());
            Err(anyhow!(
                "HTTP status client error ({}): {text}",
                status.as_u16()
            ))
        } else {
            Ok(self)
        }
    }

    async fn json_ext<T: DeserializeOwned>(self) -> Result<T> {
        let data = self.bytes().await.context("reading response bytes")?;
        let text = str::from_utf8(&data)
            .context("response bytes are not valid UTF-8")?;
        let value = serde_json::from_str::<serde_json::Value>(text)
            .with_context(|| {
                anyhow!("response text is not valid JSON\n\n{text}")
            })?;

        json_from_value(&value).with_context(|| {
            let json = serde_json::to_string_pretty(&value)
                .expect("should be able to serialize a JSON value");
            anyhow!("response does not match schema\n\n{json}")
        })
    }
}
