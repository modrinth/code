use crate::api::Result;
use theseus::{
    tags::{
        Category, DonationPlatform, GameVersion, License, Loader, TagBundle,
    },
    window_scoped,
};

/// Gets cached category tags from the database
#[tauri::command]
pub async fn tags_get_categories(
    window: tauri::Window,
) -> Result<Vec<Category>> {
    Ok(window_scoped!(window, theseus::tags::get_category_tags()).await?)
}

/// Gets cached report type tags from the database
#[tauri::command]
pub async fn tags_get_report_types(
    window: tauri::Window,
) -> Result<Vec<String>> {
    Ok(window_scoped!(window, theseus::tags::get_report_type_tags()).await?)
}

/// Gets cached loader tags from the database
#[tauri::command]
pub async fn tags_get_loaders(window: tauri::Window) -> Result<Vec<Loader>> {
    Ok(window_scoped!(window, theseus::tags::get_loader_tags()).await?)
}

/// Gets cached game version tags from the database
#[tauri::command]
pub async fn tags_get_game_versions(
    window: tauri::Window,
) -> Result<Vec<GameVersion>> {
    Ok(window_scoped!(window, theseus::tags::get_game_version_tags()).await?)
}

/// Gets cached license tags from the database
#[tauri::command]
pub async fn tags_get_licenses(window: tauri::Window) -> Result<Vec<License>> {
    Ok(window_scoped!(window, theseus::tags::get_license_tags()).await?)
}

/// Gets cached donation platform tags from the database
#[tauri::command]
pub async fn tags_get_donation_platforms(
    window: tauri::Window,
) -> Result<Vec<DonationPlatform>> {
    Ok(
        window_scoped!(window, theseus::tags::get_donation_platform_tags())
            .await?,
    )
}

/// Gets cached tag bundle from the database
#[tauri::command]
pub async fn tags_get_tag_bundle(window: tauri::Window) -> Result<TagBundle> {
    Ok(window_scoped!(window, theseus::tags::get_tag_bundle()).await?)
}
