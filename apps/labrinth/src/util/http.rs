use std::sync::LazyLock;

use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};

pub static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .default_headers(HeaderMap::from_iter([(
            USER_AGENT,
            HeaderValue::from_static(concat!(
                "Labrinth/",
                env!("COMPILATION_DATE")
            )),
        )]))
        .build()
        .unwrap()
});
