use std::sync::LazyLock;

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    database::{PgTransaction, models::DBProjectId},
    models::exp::{
        ComponentKindArrayExt, ComponentKindExt, ComponentRelation,
        ProjectComponent, ProjectComponentEdit, ProjectComponentKind,
    },
};

define! {
    #[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
    pub struct Mod {}

    #[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
    pub struct Server {
        pub max_players: u32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
    pub struct JavaServer {
        #[validate(length(max = 255))]
        pub address: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
    pub struct BedrockServer {
        #[validate(length(max = 255))]
        pub address: String,
    }
}

relations! {
    [MinecraftMod].only(),
    [
        MinecraftServer,
        MinecraftJavaServer,
        MinecraftBedrockServer,
    ]
    .only(),
    MinecraftJavaServer.requires(MinecraftServer),
    MinecraftBedrockServer.requires(MinecraftServer),
}

impl ProjectComponent for Mod {
    type Serial = Self;

    type Edit = ModEdit;

    fn kind() -> ProjectComponentKind {
        ProjectComponentKind::MinecraftMod
    }

    fn into_serial(self) -> Self::Serial {
        self
    }

    fn from_serial(serial: Self::Serial) -> Self {
        serial
    }
}

impl ProjectComponentEdit for ModEdit {
    type Component = Mod;

    async fn apply_to(
        self,
        _txn: &mut PgTransaction<'_>,
        _project_id: DBProjectId,
        _component: &mut Self::Component,
    ) -> Result<(), sqlx::Error> {
        unimplemented!();
    }
}

impl ProjectComponent for Server {
    type Serial = Self;

    type Edit = ServerEdit;

    fn kind() -> ProjectComponentKind {
        ProjectComponentKind::MinecraftServer
    }

    fn into_serial(self) -> Self::Serial {
        self
    }

    fn from_serial(serial: Self::Serial) -> Self {
        serial
    }
}

impl ProjectComponentEdit for ServerEdit {
    type Component = Server;

    async fn apply_to(
        self,
        _txn: &mut PgTransaction<'_>,
        _project_id: DBProjectId,
        component: &mut Self::Component,
    ) -> Result<(), sqlx::Error> {
        if let Some(max_players) = self.max_players {
            component.max_players = max_players;
        }
        Ok(())
    }
}

impl ProjectComponent for JavaServer {
    type Serial = Self;

    type Edit = JavaServerEdit;

    fn kind() -> ProjectComponentKind {
        ProjectComponentKind::MinecraftJavaServer
    }

    fn into_serial(self) -> Self::Serial {
        self
    }

    fn from_serial(serial: Self::Serial) -> Self {
        serial
    }
}

impl ProjectComponentEdit for JavaServerEdit {
    type Component = JavaServer;

    async fn apply_to(
        self,
        _txn: &mut PgTransaction<'_>,
        _project_id: DBProjectId,
        component: &mut Self::Component,
    ) -> Result<(), sqlx::Error> {
        if let Some(address) = self.address {
            component.address = address;
        }
        Ok(())
    }
}

impl ProjectComponent for BedrockServer {
    type Serial = Self;

    type Edit = BedrockServerEdit;

    fn kind() -> ProjectComponentKind {
        ProjectComponentKind::MinecraftBedrockServer
    }

    fn into_serial(self) -> Self::Serial {
        self
    }

    fn from_serial(serial: Self::Serial) -> Self {
        serial
    }
}

impl ProjectComponentEdit for BedrockServerEdit {
    type Component = BedrockServer;

    async fn apply_to(
        self,
        _txn: &mut PgTransaction<'_>,
        _project_id: DBProjectId,
        component: &mut Self::Component,
    ) -> Result<(), sqlx::Error> {
        if let Some(address) = self.address {
            component.address = address;
        }
        Ok(())
    }
}
