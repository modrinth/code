use anyhow::Result;

use crate::{AppState, export::fabriclike, model::FabriclikeLoader};

pub const FORMAT_VERSION: u32 = 0;

pub async fn export(app: &AppState) -> Result<()> {
    fabriclike::export(app, FabriclikeLoader::Ornithe, FORMAT_VERSION).await
}
