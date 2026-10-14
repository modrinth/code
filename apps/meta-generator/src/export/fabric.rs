use anyhow::Result;

use crate::{
    AppState, export::fabriclike, model::FabriclikeLoader, store::PutStatus,
};

pub const FORMAT_VERSION: u32 = 0;

pub async fn export(app: &AppState) -> Result<PutStatus> {
    fabriclike::export(app, FabriclikeLoader::Fabric, FORMAT_VERSION).await
}
