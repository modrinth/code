use derive_more::Deref;
use toasty::{Embed, Model};

/// Minecraft game client mod loader kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Embed)]
pub enum MinecraftLoader {
    Fabric,
    Forge,
    NeoForge,
    Quilt,
}

/// ID of a [`LoaderVersion`] row.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref, Embed)]
pub struct MinecraftLoaderVersionId(pub String);

/// Release kind of a [`LoaderVersion`] row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Embed)]
pub enum MinecraftLoaderVersionKind {
    Stable,
    Beta,
}

/// Single version of a Minecraft mod loader release.
///
/// On its own, a mod loader version can't specify how to launch any given
/// Minecraft instance with that loader.
#[derive(Debug, Clone, Model)]
pub struct MinecraftLoaderVersion {
    #[key]
    pub loader: MinecraftLoader,
    #[key]
    pub id: MinecraftLoaderVersionId,
    pub version_kind: MinecraftLoaderVersionKind,
}
