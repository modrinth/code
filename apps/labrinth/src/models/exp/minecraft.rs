use std::time::Duration;

use chrono::{DateTime, Utc};
use eyre::Result;
use minecraft_meta_model::model::MinecraftVersionName;
use serde::{Deserialize, Serialize};
use serde_binhum::serde_binhum;
use tracing::warn;
use validator::Validate;

use crate::{
    models::{
        exp::{
            ProjectComponentKind,
            component::{self, Component, ComponentEdit, ComponentQuery},
            project::{
                ProjectComponent, ProjectQueryContext, ProjectQueryRequirements,
            },
            version::{VersionComponent, VersionComponentKind},
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
    Af,
    Am,
    Az,
    Be,
    Bs,
    Ca,
    Eo,
    Eu,
    Fa,
    Ga,
    Gl,
    Hy,
    Is,
    Ka,
    Kk,
    Km,
    Kn,
    Lo,
    Mk,
    Ml,
    Mn,
    Mr,
    My,
    Ne,
    Pa,
    Si,
    Sl,
    Sq,
    Sw,
    Ta,
    Te,
    Uz,
    Yo,
    Zu,
}

/// Supported Minecraft version environments, including the explicit legacy unknown value.
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
    strum::IntoStaticStr,
    strum::EnumString,
    strum::EnumIter,
)]
#[serde(rename_all = "snake_case")]
#[strum(
    serialize_all = "snake_case",
    parse_err_ty = eyre::Report,
    parse_err_fn = invalid_minecraft_metadata_value
)]
pub enum Environment {
    ClientAndServer,
    ClientOnly,
    ClientOnlyServerOptional,
    SingleplayerOnly,
    ServerOnly,
    ServerOnlyClientOptional,
    DedicatedServerOnly,
    ClientOrServer,
    ClientOrServerPrefersBoth,
    Unknown,
}

/// Loader field values for mrpack versions, not mrpack manifest dependency keys.
/// The legacy field was populated from every non-mrpack loader.
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
    strum::IntoStaticStr,
    strum::EnumString,
    strum::EnumIter,
)]
#[serde(rename_all = "lowercase")]
#[strum(
    serialize_all = "lowercase",
    parse_err_ty = eyre::Report,
    parse_err_fn = invalid_minecraft_metadata_value
)]
pub enum MrpackLoader {
    Forge,
    Fabric,
    Quilt,
    Modloader,
    Rift,
    Liteloader,
    Minecraft,
    Bukkit,
    Spigot,
    Paper,
    Purpur,
    Bungeecord,
    Waterfall,
    Velocity,
    Sponge,
    Optifine,
    Iris,
    Canvas,
    Vanilla,
    Datapack,
    Folia,
    Neoforge,
    Babric,
    #[serde(rename = "bta-babric")]
    #[strum(serialize = "bta-babric")]
    BtaBabric,
    #[serde(rename = "java-agent")]
    #[strum(serialize = "java-agent")]
    JavaAgent,
    #[serde(rename = "legacy-fabric")]
    #[strum(serialize = "legacy-fabric")]
    LegacyFabric,
    Nilloader,
    Ornithe,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        self.into()
    }
}

impl MrpackLoader {
    pub fn as_str(&self) -> &'static str {
        self.into()
    }
}

fn invalid_minecraft_metadata_value(value: &str) -> eyre::Report {
    eyre::eyre!("invalid Minecraft metadata value `{value}`")
}

component::define! {
    #[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
    /// Umbrella for legacy Minecraft mods, modpacks, plugins, data packs,
    /// resource packs, and shaders; not restricted to the legacy mod type.
    pub struct ModProject {}

    /// Shared version metadata for Minecraft mods, modpacks, plugins, data packs,
    /// resource packs, and shaders.
    #[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
    pub struct ModVersion: version {
        #[base(schema(value_type = Vec<String>))]
        #[edit(serde(default), schema(value_type = Option<Vec<String>>))]
        #[create(required)]
        pub game_versions: Vec<MinecraftVersionName>,
        #[base(serde(default))]
        #[edit(serde(
            default,
            skip_serializing_if = "Option::is_none",
            with = "serde_with::rust::double_option"
        ))]
        #[create(optional)]
        pub environment: Option<Environment>,
        #[base(serde(default))]
        #[edit(serde(default))]
        #[create(default)]
        pub mrpack_loaders: Vec<MrpackLoader>,
    }

    /// Listing for a Minecraft server.
    #[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
    pub struct ServerProject {
        #[base(serde(default))]
        #[edit(serde(
            default,
            skip_serializing_if = "Option::is_none",
            with = "serde_with::rust::double_option"
        ))]
        #[create(optional)]
        /// Maximum number of players allowed on the server.
        pub max_players: Option<u32>,
        #[base(serde(default))]
        #[edit(serde(
            default,
            skip_serializing_if = "Option::is_none",
            with = "serde_with::rust::double_option"
        ))]
        #[create(optional)]
        /// Country which this server is hosted in.
        #[validate(length(min = 2, max = 2))]
        pub country: Option<String>,
        #[base(serde(default))]
        #[edit(serde(
            default,
            skip_serializing_if = "Option::is_none",
            with = "serde_with::rust::double_option"
        ))]
        #[create(optional)]
        /// Geographical region which this server is hosted in.
        pub region: Option<ServerRegion>,
        #[base(serde(default))]
        #[edit(serde(default))]
        #[create(default)]
        /// Languages which the owners of this server prefer.
        pub languages: Vec<Language>,
        #[base(serde(default))]
        #[edit(serde(
            default,
            skip_serializing_if = "Option::is_none",
            with = "serde_with::rust::double_option"
        ))]
        #[create(optional)]
        /// Which version of the listing this server is currently using.
        pub active_version: Option<VersionId>,
    }

    /// Version of a Minecraft Java server listing.
    #[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
    pub struct JavaServerVersion: version {}

    /// Listing for a Minecraft Bedrock server.
    #[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
    pub struct BedrockServerProject {
        #[base()]
        #[edit(serde(default))]
        #[create(required)]
        /// Address (IP or domain name) of the Bedrock server, excluding port.
        #[validate(length(max = 255))]
        pub address: String,
    }
}

impl VersionComponent for ModVersion {
    fn kind() -> VersionComponentKind {
        VersionComponentKind::MinecraftMod
    }
}

impl VersionComponent for JavaServerVersion {
    fn kind() -> VersionComponentKind {
        VersionComponentKind::MinecraftJavaServer
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
    /// What game content this server is using.
    #[serde(default)]
    pub content: ServerContent,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct JavaServerProjectEdit {
    #[validate(length(max = 255))]
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub content: Option<ServerContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct JavaServerProjectQuery {
    pub address: String,
    pub content: ServerContentQuery,
    pub ping: Option<JavaServerPing>,
    pub verified_plays_2w: Option<u64>,
    pub verified_plays_4w: Option<u64>,
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
        requirements.minecraft_server_analytics.insert(project_id);
    }

    fn populate(
        serial: Self::Component,
        project_id: ProjectId,
        context: &ProjectQueryContext,
    ) -> Result<Self> {
        let analytics = context.minecraft_server_analytics.get(&project_id);
        Ok(Self {
            address: serial.address,
            content: match serial.content {
                ServerContent::Vanilla {
                    supported_game_versions,
                    recommended_game_version,
                } => ServerContentQuery::Vanilla {
                    supported_game_versions,
                    recommended_game_version,
                },
                ServerContent::Modpack { version_id } => {
                    match context.partial_versions.get(&version_id) {
                        Some(version) => ServerContentQuery::Modpack {
                            version_id,
                            project_id: version.project_id,
                            project_name: version.project_name.clone(),
                            project_icon: version.project_icon.clone(),
                        },
                        None => {
                            // TODO: should be upgraded to an error,
                            // but it's too easy to fall into this illegal state right now
                            warn!("no modpack info for version {version_id:?}");
                            ServerContentQuery::Modpack {
                                version_id,
                                project_id: ProjectId(0),
                                project_name: String::new(),
                                project_icon: String::new(),
                            }
                        }
                    }
                }
            },
            ping: context
                .minecraft_java_server_pings
                .get(&project_id)
                .cloned(),
            verified_plays_2w: analytics.map(|a| a.verified_plays_2w),
            verified_plays_4w: analytics.map(|a| a.verified_plays_4w),
        })
    }
}

impl ComponentEdit for JavaServerProjectEdit {
    type Component = JavaServerProject;

    fn create(self) -> Result<Self::Component> {
        Ok(JavaServerProject {
            address: self.address.wrap_err("missing `address`")?,
            content: self.content.unwrap_or_default(),
        })
    }

    async fn apply_to(self, component: &mut Self::Component) -> Result<()> {
        if let Some(address) = self.address {
            component.address = address;
        }
        if let Some(content) = self.content {
            component.content = content;
        }
        Ok(())
    }
}

/// What game content a [`JavaServerProject`] is using.
#[derive(Debug, Clone)]
#[serde_binhum(schema)]
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
#[derive(Debug, Clone)]
#[serde_binhum(schema)]
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
pub enum ServerRegion {
    UsEast,
    UsWest,
    Europe,
    Asia,
    Australia,
    SouthAmerica,
    MiddleEast,
    Russia,
}

/// Recorded ping attempt that Labrinth made to a Minecraft Java server project.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct JavaServerPing {
    /// When the ping was performed.
    pub when: DateTime<Utc>,
    /// Address of the server at the time of the ping.
    pub address: String,
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
    pub version_protocol: i32,
    /// Description/MOTD of the server as shown in the server list.
    pub description: String,
    /// Number of players online at the time.
    pub players_online: i32,
    /// Maximum number of players allowed on the server.
    pub players_max: i32,
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
