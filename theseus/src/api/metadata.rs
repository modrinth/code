use crate::State;
pub use daedalus::minecraft::VersionManifest;
pub use daedalus::modded::Manifest;

#[tracing::instrument]
pub async fn get_minecraft_versions() -> crate::Result<VersionManifest> {
    let state = State::get().await?;
    let tags = state.metadata.read().await.minecraft.clone();

    Ok(tags)
}

#[tracing::instrument]
pub async fn get_fabric_versions() -> crate::Result<Manifest> {
    let state = State::get().await?;
    let tags = state.metadata.read().await.fabric.clone();

    Ok(tags)
}

#[tracing::instrument]
pub async fn get_forge_versions() -> crate::Result<Manifest> {
    let state = State::get().await?;
    let tags = state.metadata.read().await.forge.clone();

    Ok(tags)
}

#[tracing::instrument]
pub async fn get_quilt_versions() -> crate::Result<Manifest> {
    let state = State::get().await?;
    let tags = state.metadata.read().await.quilt.clone();

    Ok(tags)
}
