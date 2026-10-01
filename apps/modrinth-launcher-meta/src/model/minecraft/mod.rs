mod forge;
mod loader;

pub use forge::*;
pub use loader::*;

use derive_more::Deref;
use toasty::{Embed, Model};

/// ID of a [`MinecraftVersion`] row.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref, Embed)]
pub struct MinecraftVersionId(pub String);

/// Single version of a Minecraft client release, as provided by Mojang.
///
/// A [`MinecraftVersion`] is not inherently associated to any loader
/// information.
#[derive(Debug, Clone, Model)]
pub struct MinecraftVersion {
    #[key]
    pub id: MinecraftVersionId,
    pub version_kind: MinecraftVersionKind,
    /// When this version was released by Mojang.
    pub released_at: Timestamp,
}

/// Release kind of a [`MinecraftVersion`] row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Embed)]
pub enum MinecraftVersionKind {
    Release,
    Snapshot,
    OldAlpha,
    OldBeta,
}
