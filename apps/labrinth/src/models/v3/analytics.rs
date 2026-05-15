use clickhouse::Row;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::net::Ipv6Addr;
use uuid::Uuid;

#[derive(Debug, Row, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
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

    // added retroactively - may be empty
    pub reason: String,
    pub game_version: String,
    pub loader: String,
}

/// Why a project was downloaded.
#[derive(
    Debug,
    Display,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[display(rename_all = "snake_case")]
pub enum DownloadReason {
    /// Project was downloaded directly by the user.
    Standalone,
    /// Project was downloaded as a dependency, possibly transitive, of another
    /// project.
    Dependency,
    /// Project was downloaded as part of a modpack.
    Modpack,
    /// Project was re-downloaded due to an update.
    Update,
}

impl std::str::FromStr for DownloadReason {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::Value::String(s.to_string()))
            .map_err(|_| ())
    }
}

#[derive(Debug, Row, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
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

#[derive(Debug, Row, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct AffiliateCodeClick {
    pub recorded: i64,
    pub domain: String,

    // Modrinth User ID for logged in users
    pub user_id: u64,
    pub affiliate_code_id: u64,

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

    // added retroactively - may be empty
    pub country: String,
}

#[derive(Row, Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
pub struct MinecraftServerPlay {
    pub recorded: i64,
    pub user_id: u64,
    pub project_id: u64,
    #[serde(with = "clickhouse::serde::uuid")]
    pub minecraft_uuid: Uuid,
    pub ip: Ipv6Addr,
}
