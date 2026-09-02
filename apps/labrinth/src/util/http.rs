use std::sync::LazyLock;

use derive_more::Deref;

pub static HTTP_CLIENT: LazyLock<HttpClient> = LazyLock::new(HttpClient::new);

#[derive(Debug, Clone, Deref)]
pub struct HttpClient(reqwest::Client);

impl HttpClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .default_headers(reqwest::header::HeaderMap::from_iter([(
                reqwest::header::USER_AGENT,
                reqwest::header::HeaderValue::from_static(concat!(
                    "Labrinth/",
                    env!("COMPILATION_DATE")
                )),
            )]))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap();
        Self(client)
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}
