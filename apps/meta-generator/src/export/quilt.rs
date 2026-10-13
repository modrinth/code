use anyhow::Result;

use crate::{AppState, export::fabriclike, model::FabriclikeLoader};

pub const FORMAT_VERSION: u32 = 1;

pub async fn export(app: &AppState) -> Result<()> {
    fabriclike::export(app, FabriclikeLoader::Quilt, FORMAT_VERSION).await
}
