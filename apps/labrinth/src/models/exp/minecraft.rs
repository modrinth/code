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
        /// List of supported Minecraft Java client versions which can join this
        /// server.
        pub supported_game_versions: Vec<String>,
        #[base()]
        #[edit(serde(
            default,
            skip_serializing_if = "Option::is_none",
            with = "serde_with::rust::double_option"
        ))]
        /// Recommended Minecraft Java client version to use to join this server.
        pub recommended_game_version: Option<String>,
    }

    #[component(VersionComponentKind::MinecraftJavaServer)]
    /// Listing for a Minecraft Java server.
    #[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
    pub struct JavaServerVersion {
        #[base()]
        #[edit(serde(
            default,
            skip_serializing_if = "Option::is_none",
            with = "serde_with::rust::double_option"
        ))]
        /// What modpack version this server is using.
        ///
        /// If the server is vanilla, this is [`None`].
        pub modpack: Option<VersionId>,
    }

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
