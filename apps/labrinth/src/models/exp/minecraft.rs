use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::models::{
    exp::{ProjectComponentKind, VersionComponentKind, component},
    ids::VersionId,
};

component::define! {
    #[component(ProjectComponentKind::MinecraftMod)]
    #[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
    pub struct ModProject {}

    #[component(ProjectComponentKind::MinecraftServer)]
    /// Listing for a Minecraft server.
    #[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
    pub struct ServerProject {
        #[base()]
        #[edit(serde(
            default,
            skip_serializing_if = "Option::is_none",
            with = "serde_with::rust::double_option"
        ))]
        /// Maximum number of players allowed on the server.
        pub max_players: Option<u32>,
        #[base()]
        #[edit(serde(
            default,
            skip_serializing_if = "Option::is_none",
            with = "serde_with::rust::double_option"
        ))]
        /// Country which this server is hosted in.
        #[validate(length(min = 2, max = 2))]
        pub country: Option<String>,
        #[base()]
        #[edit(serde(
            default,
            skip_serializing_if = "Option::is_none",
            with = "serde_with::rust::double_option"
        ))]
        /// Which version of the listing this server is currently using.
        pub active_version: Option<VersionId>,
    }

    #[component(ProjectComponentKind::MinecraftJavaServer)]
    /// Listing for a Minecraft Java server.
    #[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
    pub struct JavaServerProject {
        #[base()]
        #[edit(serde(default))]
        /// Address (IP or domain name) of the Java server, excluding port.
        #[validate(length(max = 255))]
        pub address: String,
        #[base()]
        #[edit(serde(default))]
        /// Port which the server runs on.
        pub port: u16,
        #[base(serde(default))]
        #[edit(serde(default))]
        /// What game content this server is using.
        pub content: ServerContent,
    }

    #[component(VersionComponentKind::MinecraftJavaServer)]
    /// Version of a Minecraft Java server listing.
    #[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
    pub struct JavaServerVersion {}

    #[component(ProjectComponentKind::MinecraftBedrockServer)]
    /// Listing for a Minecraft Bedrock server.
    #[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
    pub struct BedrockServerProject {
        #[base()]
        #[edit(serde(default))]
        /// Address (IP or domain name) of the Bedrock server, excluding port.
        #[validate(length(max = 255))]
        pub address: String,
        #[base()]
        #[edit(serde(default))]
        /// Port which the server runs on.
        pub port: u16,
    }
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
            [MinecraftMod].only(),
            [MinecraftServer, MinecraftJavaServer, MinecraftBedrockServer].only(),
            MinecraftJavaServer.requires(MinecraftServer),
            MinecraftBedrockServer.requires(MinecraftServer),
        ]
    }
}
