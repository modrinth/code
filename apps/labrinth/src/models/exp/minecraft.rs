use chrono::{DateTime, Utc};
use labrinth_derive::Component;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::models::{
    exp::{ProjectComponentKind, component},
    ids::VersionId,
};

/// Listing for a Minecraft server.
#[derive(
    Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema, Component,
)]
pub struct ServerProject {
    /// Maximum number of players allowed on the server.
    pub max_players: Option<u32>,
    /// Country which this server is hosted in.
    #[validate(length(min = 2, max = 2))]
    pub country: Option<String>,
    /// Which version of the listing this server is currently using.
    pub active_version: Option<VersionId>,
}

/// Listing for a Minecraft Java server.
#[derive(
    Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema, Component,
)]
pub struct JavaServerProject {
    /// Address (IP or domain name) of the Java server, excluding port.
    #[validate(length(max = 255))]
    pub address: String,
    /// Port which the server runs on.
    pub port: u16,
    /// What game content this server is using.
    pub content: ServerContent,
    #[component(synthetic)]
    /// Last received ping data from the server.
    pub ping: Option<JavaServerPing>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct JavaServerPing {
    /// When this ping data was gathered.
    pub pinged_at: DateTime<Utc>,
    /// If the ping was successful, contains server ping information.
    pub data: Option<JavaServerPingData>,
}

/// Ping data for a Minecraft Java server.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct JavaServerPingData {
    /// Server description/MOTD as shown in the server list.
    pub description: String,
    /// Number of players currently online.
    pub players_online: u32,
    /// Maximum number of players allowed on the server.
    pub players_max: u32,
}

/// Listing for a Minecraft Bedrock server.
#[derive(
    Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema, Component,
)]
pub struct BedrockServerProject {
    /// Address (IP or domain name) of the Bedrock server, excluding port.
    #[validate(length(max = 255))]
    pub address: String,
    /// Port which the server runs on.
    pub port: u16,
}

/// What game content a [`JavaServerProject`] is using.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ServerContent {
    /// Server runs modded content with a modpack found on the Modrinth platform.
    Modpack {
        /// Version ID of the modpack which the server runs.
        ///
        /// This version may or may not belong to the server project, since
        /// server projects may also be treated as modpacks.
        version_id: VersionId,
    },
    /// Server is a vanilla Minecraft server.
    Vanilla {
        /// List of supported Minecraft Java client versions which can join this
        /// server.
        supported_game_versions: Vec<String>,
        /// Recommended Minecraft Java client version to use to join this server.
        recommended_game_version: Option<String>,
    },
}

impl Default for ServerContent {
    fn default() -> Self {
        ServerContent::Vanilla {
            supported_game_versions: Vec::new(),
            recommended_game_version: None,
        }
    }
}

component::relations! {
    pub(super) static PROJECT_COMPONENT_RELATIONS: ProjectComponentKind = {
        use ProjectComponentKind::*;

        [
            [MinecraftServer, MinecraftJavaServer, MinecraftBedrockServer].only(),
            MinecraftJavaServer.requires(MinecraftServer),
            MinecraftBedrockServer.requires(MinecraftServer),
        ]
    }
}
