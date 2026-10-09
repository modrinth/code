use anyhow::Result;

use crate::{AppState, upstream::Upstreams};

mod download;
pub mod fabric;
pub mod forge;
pub mod forgelike;
pub mod mojang;
pub mod neoforge;
pub mod quilt;

pub async fn run(app: AppState, upstreams: Upstreams) -> Result<()> {
    Ok(())
}
