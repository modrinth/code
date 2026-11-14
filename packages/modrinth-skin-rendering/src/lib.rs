//! CPU rendering of Minecraft player skins.
#![cfg_attr(
    all(feature = "gltf", feature = "png"),
    doc = include_str!("../README.md")
)]

#[cfg(feature = "cache")]
pub mod cache;
pub mod ears;
mod error;
pub mod math;
mod model;
mod raster;
mod render;
mod scene;
mod texture;

pub use error::{Error, Result};
pub use math::{Mat4, Vec3};
pub use model::{
    BodyPart, Bounds, Mesh, Model, ModelConventions, Node, Primitive,
};
pub use render::{Camera, Lighting, RenderOptions, render, render_face};
pub use scene::{Material, Player, Scene, Triangle};
pub use texture::{
    ArmModel, Rgba, SKIN_SIZE, SkinTexture, Texture, normalize_skin_pixels,
};

/// Bump when rendered output changes. Cache keys include it.
pub const RENDERER_VERSION: u32 = 1;
