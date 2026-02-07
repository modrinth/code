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
        /// Maximum number of players allowed on the server.
        pub max_players: u32,
        /// Country which this server is hosted in.
        #[validate(length(min = 2, max = 2))]
        pub country: String,
        /// Which version of the listing this server is currently using.
        pub active_version: Option<VersionId>,
    }

    #[component(ProjectComponentKind::MinecraftJavaServer)]
    /// Listing for a Minecraft Java server.
    #[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
    pub struct JavaServerProject {
        /// Address (IP or domain name) of the Java server, excluding port.
        #[validate(length(max = 255))]
        pub address: String,
        /// Port which the server runs on.
        pub port: u16,
    }

    #[component(VersionComponentKind::MinecraftJavaServer)]
    /// Listing for a Minecraft Java server.
    #[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
    pub struct JavaServerVersion {
        /// What modpack version this server is using.
        ///
        /// If the server is vanilla, this is [`None`].
        pub modpack: Option<VersionId>,
    }

    #[component(ProjectComponentKind::MinecraftBedrockServer)]
    /// Listing for a Minecraft Bedrock server.
    #[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
    pub struct BedrockServerProject {
        /// Address (IP or domain name) of the Bedrock server, excluding port.
        #[validate(length(max = 255))]
        pub address: String,
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
