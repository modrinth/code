use crate::model::{
    MinecraftLoaderVersionKind, MinecraftVersion, MinecraftVersionId,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref, Embed)]
pub struct MinecraftForgeVersionId(pub String);

#[derive(Debug, Clone, Model)]
pub struct MinecraftForgeVersion {
    #[key]
    pub id: MinecraftForgeVersionId,
    pub version_kind: MinecraftLoaderVersionKind,
}

#[derive(Debug, Clone, Model)]
pub struct MinecraftForgeGameVersion {
    #[key]
    pub minecraft_version_id: MinecraftVersionId,
    #[belongs_to]
    pub minecraft_version: toasty::Deferred<MinecraftVersion>,
    #[key]
    pub forge_version_id: MinecraftForgeVersionId,
    #[belongs_to]
    pub forge_version: toasty::Deferred<MinecraftForgeVersion>,
}
