use anyhow::Result;

use crate::{
    AppState, export::fabriclike, model::FabriclikeLoader, store::PutStatus,
};

pub const FORMAT_VERSION: u32 = 1;

pub async fn export(
    app: &AppState,
    maven: &super::maven::MavenExports<'_>,
) -> Result<PutStatus> {
    fabriclike::export(app, maven, FabriclikeLoader::Quilt, FORMAT_VERSION)
        .await
}
