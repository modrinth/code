use modrinth_maxmind::{MaxMind, geoip2};
use std::net::Ipv6Addr;

pub struct MaxMindIndexer {
    pub maxmind: MaxMind,
}

impl MaxMindIndexer {
    pub async fn new() -> Self {
        Self {
            maxmind: MaxMind::new().await,
        }
    }

    pub async fn query(&self, ip: Ipv6Addr) -> Option<String> {
        let reader = self.maxmind.reader.as_ref()?;
        reader
            .lookup::<geoip2::Country>(ip.into())
            .ok()?
            .and_then(|c| c.country)
            .and_then(|c| c.iso_code.map(|s| s.to_string()))
    }
}
