//! Schemas for files we download from our upstreams (e.g. Minecraft version
//! catalog manifest).

pub mod fabric;
pub mod fabriclike;
pub mod forge;
pub mod mojang;
pub mod neoforge;
pub mod quilt;

#[derive(Debug, Clone, Copy)]
pub struct Upstreams {
    pub mojang: bool,
    pub fabric: bool,
    pub forge: bool,
    pub neoforge: bool,
    pub quilt: bool,
}

impl Default for Upstreams {
    fn default() -> Self {
        Self {
            mojang: true,
            fabric: true,
            forge: true,
            neoforge: true,
            quilt: true,
        }
    }
}
