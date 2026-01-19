use std::sync::LazyLock;

use serde::{Deserialize, Serialize};
use sqlx::PgTransaction;
use validator::Validate;

use crate::{
    database::models::DBProjectId,
    models::v67::{
        ComponentKindArrayExt, ComponentKindExt, ComponentRelation,
        ProjectComponent, ProjectComponentKind,
    },
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

#[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct Mod {}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct Server {
    pub max_players: Option<u32>,
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

// impl

impl ProjectComponent for Mod {
    fn kind() -> ProjectComponentKind {
        ProjectComponentKind::MinecraftMod
    }

    async fn upsert(
        &self,
        _txn: &mut PgTransaction<'_>,
        _project_id: DBProjectId,
    ) -> Result<(), sqlx::Error> {
        unimplemented!();
    }
}

impl ProjectComponent for Server {
    fn kind() -> ProjectComponentKind {
        ProjectComponentKind::MinecraftServer
    }

    async fn upsert(
        &self,
        txn: &mut PgTransaction<'_>,
        project_id: DBProjectId,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            INSERT INTO minecraft_server_projects (id, max_players)
            VALUES ($1, $2)
            ON CONFLICT (id) DO UPDATE SET max_players = $2
            ",
            project_id as _,
            self.max_players.map(|n| n.cast_signed()),
        )
        .execute(&mut **txn)
        .await?;
        Ok(())
    }
}

impl ProjectComponent for JavaServer {
    fn kind() -> ProjectComponentKind {
        ProjectComponentKind::MinecraftJavaServer
    }

    async fn upsert(
        &self,
        txn: &mut PgTransaction<'_>,
        project_id: DBProjectId,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            INSERT INTO minecraft_java_server_projects (id, address)
            VALUES ($1, $2)
            ON CONFLICT (id) DO UPDATE SET address = $2
            ",
            project_id as _,
            self.address,
        )
        .execute(&mut **txn)
        .await?;
        Ok(())
    }
}

impl ProjectComponent for BedrockServer {
    fn kind() -> ProjectComponentKind {
        ProjectComponentKind::MinecraftBedrockServer
    }

    async fn upsert(
        &self,
        txn: &mut PgTransaction<'_>,
        project_id: DBProjectId,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            INSERT INTO minecraft_bedrock_server_projects (id, address)
            VALUES ($1, $2)
            ON CONFLICT (id) DO UPDATE SET address = $2
            ",
            project_id as _,
            self.address,
        )
        .execute(&mut **txn)
        .await?;
        Ok(())
    }
}
