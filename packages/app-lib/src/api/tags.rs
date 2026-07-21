//! Theseus tag management interface
use crate::state::CachedEntry;
pub use crate::{
    InvocationContext, State,
    state::{Category, DonationPlatform, GameVersion, Loader},
};

/// Get category tags
#[tracing::instrument]
pub async fn get_category_tags(
    context: &InvocationContext,
) -> crate::Result<Vec<Category>> {
    let state = State::get().await?;
    let categories = CachedEntry::get_categories(
        context,
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await?
    .ok_or_else(|| crate::ErrorKind::NoValueFor("category tags".to_string()))?;

    Ok(categories)
}

/// Get report type tags
#[tracing::instrument]
pub async fn get_report_type_tags(
    context: &InvocationContext,
) -> crate::Result<Vec<String>> {
    let state = State::get().await?;
    let report_types = CachedEntry::get_report_types(
        context,
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await?
    .ok_or_else(|| {
        crate::ErrorKind::NoValueFor("report type tags".to_string())
    })?;

    Ok(report_types)
}

/// Get loader tags
#[tracing::instrument]
pub async fn get_loader_tags(
    context: &InvocationContext,
) -> crate::Result<Vec<Loader>> {
    let state = State::get().await?;
    let loaders = CachedEntry::get_loaders(
        context,
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await?
    .ok_or_else(|| crate::ErrorKind::NoValueFor("loader tags".to_string()))?;

    Ok(loaders)
}

/// Get game version tags
#[tracing::instrument]
pub async fn get_game_version_tags(
    context: &InvocationContext,
) -> crate::Result<Vec<GameVersion>> {
    let state = State::get().await?;
    let game_versions = CachedEntry::get_game_versions(
        context,
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await?
    .ok_or_else(|| {
        crate::ErrorKind::NoValueFor("game version tags".to_string())
    })?;

    Ok(game_versions)
}

/// Get donation platform tags
#[tracing::instrument]
pub async fn get_donation_platform_tags(
    context: &InvocationContext,
) -> crate::Result<Vec<DonationPlatform>> {
    let state = State::get().await?;
    let donation_platforms = CachedEntry::get_donation_platforms(
        context,
        None,
        &state.pool,
        &state.api_semaphore,
    )
    .await?
    .ok_or_else(|| {
        crate::ErrorKind::NoValueFor("donation platform tags".to_string())
    })?;

    Ok(donation_platforms)
}
