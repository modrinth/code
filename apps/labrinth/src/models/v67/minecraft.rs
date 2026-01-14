use std::sync::LazyLock;

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::models::v67::{
    ComponentKindArrayExt, ComponentKindExt, ComponentRelation,
    ProjectComponent, ProjectComponentKind,
};

pub(super) static RELATIONS: LazyLock<Vec<ComponentRelation>> =
    LazyLock::new(|| {
        use ProjectComponentKind as C;

        vec![
            [C::MinecraftMod].only(),
            [
                C::MinecraftServer,
                C::MinecraftJavaServer,
                C::MinecraftBedrockServer,
            ]
            .only(),
            C::MinecraftJavaServer.requires(C::MinecraftServer),
            C::MinecraftBedrockServer.requires(C::MinecraftServer),
        ]
    });

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModCreate {}

impl ProjectComponent for ModCreate {
    fn kind() -> ProjectComponentKind {
        ProjectComponentKind::MinecraftMod
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ServerCreate {
    pub max_players: Option<u32>,
}

impl ProjectComponent for ServerCreate {
    fn kind() -> ProjectComponentKind {
        ProjectComponentKind::MinecraftServer
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct JavaServerCreate {
    #[validate(length(max = 255))]
    pub address: String,
}

impl ProjectComponent for JavaServerCreate {
    fn kind() -> ProjectComponentKind {
        ProjectComponentKind::MinecraftJavaServer
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct BedrockServerCreate {
    #[validate(length(max = 255))]
    pub address: String,
}

impl ProjectComponent for BedrockServerCreate {
    fn kind() -> ProjectComponentKind {
        ProjectComponentKind::MinecraftBedrockServer
    }
}
