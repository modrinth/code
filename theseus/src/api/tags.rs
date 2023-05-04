//! Theseus tag management interface
pub use crate::{
    state::{Category, DonationPlatform, GameVersion, Loader, Tags},
    State,
};

// Get bundled set of tags
#[tracing::instrument]
pub async fn get_tag_bundle() -> crate::Result<Tags> {
    let state = State::get().await?;
    let tags = state.tags.read().await;

    Ok(tags.get_tag_bundle())
}

/// Get category tags
#[tracing::instrument]
pub async fn get_category_tags() -> crate::Result<Vec<Category>> {
    let state = State::get().await?;
    let tags = state.tags.read().await;

    Ok(tags.get_categories())
}

/// Get report type tags
#[tracing::instrument]
pub async fn get_report_type_tags() -> crate::Result<Vec<String>> {
    let state = State::get().await?;
    let tags = state.tags.read().await;

    Ok(tags.get_report_types())
}

/// Get loader tags
#[tracing::instrument]
pub async fn get_loader_tags() -> crate::Result<Vec<Loader>> {
    let state = State::get().await?;
    let tags = state.tags.read().await;

    Ok(tags.get_loaders())
}

/// Get game version tags
#[tracing::instrument]
pub async fn get_game_version_tags() -> crate::Result<Vec<GameVersion>> {
    let state = State::get().await?;
    let tags = state.tags.read().await;

    Ok(tags.get_game_versions())
}

/// Get donation platform tags
#[tracing::instrument]
pub async fn get_donation_platform_tags() -> crate::Result<Vec<DonationPlatform>>
{
    let state = State::get().await?;
    let tags = state.tags.read().await;

    Ok(tags.get_donation_platforms())
}
