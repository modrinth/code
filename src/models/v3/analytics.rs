use clickhouse::Row;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::net::Ipv6Addr;

#[derive(Row, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct Download {
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

#[derive(Row, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct PageView {
    pub recorded: i64,
    pub domain: String,
    pub site_path: String,

    // Modrinth User ID for logged in users
    pub user_id: u64,
    // Modrinth Project ID (used for payouts)
    pub project_id: u64,
    // whether this view will be monetized / counted for payouts
    pub monetized: bool,

    // The below information is used exclusively for data aggregation and fraud detection
    // (ex: page view botting).
    pub ip: Ipv6Addr,
    pub country: String,
    pub user_agent: String,
    pub headers: Vec<(String, String)>,
}

#[derive(Row, Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Playtime {
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
