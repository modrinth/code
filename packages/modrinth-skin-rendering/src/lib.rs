//! CPU rendering of Minecraft player skins.
#![doc = include_str!("../README.md")]

pub mod ears;
mod error;
mod model;
mod raster;
mod render;
mod scene;
mod texture;

pub use error::{Error, Result};
pub use glam::{self, Mat4, Vec3};
pub use model::{BodyPart, Bounds, Mesh, Model, Node, Primitive};
pub use render::{Camera, Lighting, RenderOptions, render, render_face};
pub use scene::{Material, Scene, Triangle};
pub use texture::{ArmModel, SkinTexture, Texture, normalize_skin_pixels};

/// Bump when rendered output changes. Cache keys include it.
pub const RENDERER_VERSION: u32 = 1;
