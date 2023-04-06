//! Configuration structs

use once_cell::sync::Lazy;
use std::time;

pub static BINCODE_CONFIG: Lazy<bincode::config::Configuration> =
    Lazy::new(|| {
        bincode::config::standard()
            .with_little_endian()
            .with_no_limit()
    });

pub static REQWEST_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    let mut headers = reqwest::header::HeaderMap::new();
    let header = reqwest::header::HeaderValue::from_str(&format!(
        "modrinth/daedalus/{} (support@modrinth.com)",
        env!("CARGO_PKG_VERSION")
    ))
    .unwrap();
    headers.insert(reqwest::header::USER_AGENT, header);
    reqwest::Client::builder()
        .tcp_keepalive(Some(time::Duration::from_secs(10)))
        .default_headers(headers)
        .build()
        .expect("Reqwest Client Building Failed")
});

pub const MODRINTH_API_URL: &str = "https://api.modrinth.com/v2/";

pub fn sled_config() -> sled::Config {
    sled::Config::default().use_compression(true)
}
