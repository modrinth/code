use std::time::Duration;

use chrono::{DateTime, Utc};
use eyre::{Result, eyre};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    models::{
        exp::{
            ProjectComponentKind,
            component::{self, Component, ComponentEdit, ComponentQuery},
            project::{
                ProjectComponent, ProjectQueryContext, ProjectQueryRequirements,
            },
        },
        ids::{ProjectId, VersionId},
    },
    util::error::Context,
};

#[derive(
    Debug,
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
pub enum Language {
    En,
    Es,
    Pt,
    Fr,
    De,
    It,
    Nl,
    Ru,
    Uk,
    Pl,
    Cs,
    Sk,
    Hu,
    Ro,
    Bg,
    Hr,
    Sr,
    El,
    Tr,
    Ar,
    He,
    Hi,
    Bn,
    Ur,
    Zh,
    Ja,
    Ko,
    Th,
    Vi,
    Id,
    Ms,
    Tl,
    Sv,
    No,
    Da,
    Fi,
    Lt,
    Lv,
    Et,
}

component::define! {
    #[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
    pub struct ModProject {}

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
        #[base(serde(default))]
        #[edit(serde(default))]
        /// Languages which the owners of this server prefer.
        pub languages: Vec<Language>,
        #[base()]
        #[edit(serde(
            default,
            skip_serializing_if = "Option::is_none",
            with = "serde_with::rust::double_option"
        ))]
        /// Which version of the listing this server is currently using.
        pub active_version: Option<VersionId>,
    }

    /// Version of a Minecraft Java server listing.
    #[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
    pub struct JavaServerVersion {}

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

impl ProjectComponent for ModProject {
    fn kind() -> ProjectComponentKind {
        ProjectComponentKind::MinecraftMod
    }
}

impl ProjectComponent for ServerProject {
    fn kind() -> ProjectComponentKind {
        ProjectComponentKind::MinecraftServer
    }
}

impl ProjectComponent for JavaServerProject {
    fn kind() -> ProjectComponentKind {
        ProjectComponentKind::MinecraftJavaServer
    }
}

impl ProjectComponent for BedrockServerProject {
    fn kind() -> ProjectComponentKind {
        ProjectComponentKind::MinecraftBedrockServer
    }
}

/// Listing for a Minecraft Java server.
#[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct JavaServerProject {
    /// Address (IP or domain name) of the Java server, excluding port.
    #[validate(length(max = 255))]
    pub address: String,
    /// Port which the server runs on.
    pub port: u16,
    /// What game content this server is using.
    pub content: ServerContent,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct JavaServerProjectEdit {
    #[validate(length(max = 255))]
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub content: Option<ServerContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct JavaServerProjectQuery {
    pub address: String,
    pub port: u16,
    pub content: ServerContentQuery,
    pub ping: Option<JavaServerPing>,
}

impl Component for JavaServerProject {
    type EntityId = ProjectId;
    type Query = JavaServerProjectQuery;
    type Edit = JavaServerProjectEdit;
}

impl ComponentQuery for JavaServerProjectQuery {
    type Component = JavaServerProject;
    type Requirements = ProjectQueryRequirements;
    type Context = ProjectQueryContext;

    fn collect_requirements(
        serial: &Self::Component,
        project_id: ProjectId,
        requirements: &mut ProjectQueryRequirements,
    ) {
        match serial.content {
            ServerContent::Vanilla { .. } => {}
            ServerContent::Modpack { version_id } => {
                requirements.partial_versions.insert(version_id);
            }
        }
        requirements.minecraft_java_server_pings.insert(project_id);
    }

    fn populate(
        serial: Self::Component,
        project_id: ProjectId,
        context: &ProjectQueryContext,
    ) -> Result<Self> {
        Ok(Self {
            address: serial.address,
            port: serial.port,
            content: match serial.content {
                ServerContent::Vanilla {
                    supported_game_versions,
                    recommended_game_version,
                } => ServerContentQuery::Vanilla {
                    supported_game_versions,
                    recommended_game_version,
                },
                ServerContent::Modpack { version_id } => {
                    let version = context
                        .partial_versions
                        .get(&version_id)
                        .ok_or_else(|| {
                            eyre!("no modpack info for version {version_id:?}")
                        })?;
                    ServerContentQuery::Modpack {
                        version_id,
                        project_id: version.project_id,
                        project_name: version.project_name.clone(),
                        project_icon: version.project_icon.clone(),
                    }
                }
            },
            ping: context
                .minecraft_java_server_pings
                .get(&project_id)
                .cloned(),
        })
    }
}

impl ComponentEdit for JavaServerProjectEdit {
    type Component = JavaServerProject;

    fn create(self) -> Result<Self::Component> {
        Ok(JavaServerProject {
            address: self.address.wrap_err("missing `address`")?,
            port: self.port.wrap_err("missing `port`")?,
            content: self.content.wrap_err("missing `content`")?,
        })
    }

    async fn apply_to(self, component: &mut Self::Component) -> Result<()> {
        if let Some(address) = self.address {
            component.address = address;
        }
        if let Some(port) = self.port {
            component.port = port;
        }
        if let Some(content) = self.content {
            component.content = content;
        }
        Ok(())
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

/// What game content a [`JavaServerProject`] is using.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ServerContentQuery {
    /// Server runs modded content with a modpack found on the Modrinth platform.
    Modpack {
        version_id: VersionId,
        project_id: ProjectId,
        project_name: String,
        project_icon: String,
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

/// Recorded ping attempt that Labrinth made to a Minecraft Java server project.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct JavaServerPing {
    /// When the ping was performed.
    pub when: DateTime<Utc>,
    /// Address of the server at the time of the ping.
    pub address: String,
    /// Port of the server at the time of the ping.
    pub port: u16,
    /// If the ping was successful, info on the ping response.
    pub data: Option<JavaServerPingData>,
}

/// Ping response data for a Minecraft Java server.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct JavaServerPingData {
    /// How long it took for the Labrinth worker to ping the server.
    ///
    /// Note: this is explicitly *not* a client-side ping time, so this should
    /// not be used to display to a client how much latency they have to a
    /// specific server. This is purely for internal metrics.
    pub latency: Duration,
    /// Reported version name of the server.
    pub version_name: String,
    /// Reported version protocol number of the server.
    pub version_protocol: u32,
    /// Description/MOTD of the server as shown in the server list.
    pub description: String,
    /// Number of players online at the time.
    pub players_online: u32,
    /// Maximum number of players allowed on the server.
    pub players_max: u32,
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
