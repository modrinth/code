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
    reqwest::Client::builder()
        .tcp_keepalive(Some(time::Duration::from_secs(10)))
        .build()
        .unwrap()
});

pub const MODRINTH_API_URL: &str = "https://api.modrinth.com/v2/";

pub fn sled_config() -> sled::Config {
    sled::Config::default().use_compression(true)
}
