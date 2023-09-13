use clickhouse::Row;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::net::Ipv6Addr;
use uuid::Uuid;

#[derive(Row, Serialize, Deserialize, Clone)]
pub struct Download {
    #[serde(with = "uuid::serde::compact")]
    pub id: Uuid,
    pub recorded: i64,
    pub domain: String,
    pub site_path: String,

    // Modrinth User ID for logged in users, default 0
    pub user_id: u64,
    // default is 0 if unknown
    pub project_id: u64,
    // default is 0 if unknown
    pub version_id: u64,

    // The below information is used exclusively for data aggregation and fraud detection
    // (ex: download botting).
    pub ip: Ipv6Addr,
    pub country: String,
    pub user_agent: String,
    pub headers: Vec<(String, String)>,
}

impl PartialEq<Self> for Download {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Download {}

impl Hash for Download {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Row, Serialize, Deserialize, Clone)]
pub struct PageView {
    #[serde(with = "uuid::serde::compact")]
    pub id: Uuid,
    pub recorded: i64,
    pub domain: String,
    pub site_path: String,

    // Modrinth User ID for logged in users
    pub user_id: u64,
    // Modrinth Project ID (used for payouts)
    pub project_id: u64,

    // The below information is used exclusively for data aggregation and fraud detection
    // (ex: page view botting).
    pub ip: Ipv6Addr,
    pub country: String,
    pub user_agent: String,
    pub headers: Vec<(String, String)>,
}

impl PartialEq<Self> for PageView {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for PageView {}

impl Hash for PageView {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Row, Serialize, Deserialize, Clone, Debug)]
pub struct Playtime {
    #[serde(with = "uuid::serde::compact")]
    pub id: Uuid,
    pub recorded: i64,
    pub seconds: u64,

    // Modrinth User ID for logged in users (unused atm)
    pub user_id: u64,
    // Modrinth Project ID
    pub project_id: u64,
    // Modrinth Version ID
    pub version_id: u64,

    pub loader: String,
    pub game_version: String,
    /// Parent modpack this playtime was recorded in
    pub parent: u64,
}

impl PartialEq<Self> for Playtime {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Playtime {}

impl Hash for Playtime {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
