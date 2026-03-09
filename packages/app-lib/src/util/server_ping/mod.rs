use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use url::Url;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ServerStatus {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Box<RawValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub players: Option<ServerPlayers>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<ServerVersion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<Url>,
    #[serde(default)]
    pub enforces_secure_chat: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ping: Option<i64>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ServerPlayers {
    pub max: i32,
    pub online: i32,
    #[serde(default)]
    pub sample: Vec<ServerGameProfile>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ServerGameProfile {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ServerVersion {
    pub name: String,
    pub protocol: i32,
    #[serde(skip_deserializing)]
    pub legacy: bool,
}
