use anyhow::{Result, anyhow};
use reqwest::Response;

pub trait ResponseExt: Sized {
    async fn error_for_status_ext(self) -> Result<Self>;
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
}
