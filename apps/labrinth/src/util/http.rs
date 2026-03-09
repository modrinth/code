use std::time::Duration;

use derive_more::Deref;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};

/// Generic HTTP client used for anywhere you need to send an HTTP request, and
/// do not care what headers or other parameters are used.
#[derive(Debug, Clone, Deref)]
pub struct HttpClient(pub reqwest::Client);

impl HttpClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .default_headers(HeaderMap::from_iter([(
                USER_AGENT,
                HeaderValue::from_static(concat!(
                    "Labrinth/",
                    env!("COMPILATION_DATE")
                )),
            )]))
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();
        Self(client)
    }
}
