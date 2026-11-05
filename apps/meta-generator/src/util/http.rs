use std::time::Duration;

use anyhow::{Result, anyhow};
use reqwest::{RequestBuilder, Response, StatusCode};
use tracing::warn;

pub trait RequestExt: Sized {
	/// Sends a request, retrying if failing.
	async fn send_retry(self) -> reqwest::Result<Response>;
}

impl RequestExt for RequestBuilder {
	async fn send_retry(self) -> reqwest::Result<Response> {
		let mut request = self;
		for attempt in 0..=3 {
			let next = request.try_clone();
			let result = request.send().await;
			let retryable = match &result {
				Ok(response) => {
					response.status() == StatusCode::TOO_MANY_REQUESTS
						|| response.status().is_server_error()
				}
				Err(error) => {
					error.is_timeout()
						|| error.is_connect()
						|| error.is_request()
				}
			};
			if !retryable || attempt == 3 {
				return result;
			}
			let Some(next) = next else {
				return result;
			};
			let delay = Duration::from_secs(1 << attempt);
			match &result {
				Ok(response) => {
					warn!(url = %response.url(), status = %response.status(), retry = attempt + 1, delay_seconds = delay.as_secs(), "retrying HTTP request")
				}
				Err(error) => {
					warn!(%error, retry = attempt + 1, delay_seconds = delay.as_secs(), "retrying HTTP request")
				}
			}
			drop(result);
			tokio::time::sleep(delay).await;
			request = next;
		}
		unreachable!()
	}
}

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
