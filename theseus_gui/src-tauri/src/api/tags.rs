use crate::api::Result;
use theseus::tags::{
    Category, DonationPlatform, GameVersion, License, Loader, TagBundle,
};

/// Gets cached category tags from the database
#[tauri::command]
pub async fn tags_get_category_tags() -> Result<Vec<Category>> {
    Ok(theseus::tags::get_category_tags().await?)
}

/// Gets cached report type tags from the database
#[tauri::command]
pub async fn tags_get_report_type_tags() -> Result<Vec<String>> {
    Ok(theseus::tags::get_report_type_tags().await?)
}

/// Gets cached loader tags from the database
#[tauri::command]
pub async fn tags_get_loader_tags() -> Result<Vec<Loader>> {
    Ok(theseus::tags::get_loader_tags().await?)
}

/// Gets cached game version tags from the database
#[tauri::command]
pub async fn tags_get_game_version_tags() -> Result<Vec<GameVersion>> {
    Ok(theseus::tags::get_game_version_tags().await?)
}

/// Gets cached license tags from the database
#[tauri::command]
pub async fn tags_get_license_tags() -> Result<Vec<License>> {
    Ok(theseus::tags::get_license_tags().await?)
}

/// Gets cached donation platform tags from the database
#[tauri::command]
pub async fn tags_get_donation_platform_tags() -> Result<Vec<DonationPlatform>>
{
    Ok(theseus::tags::get_donation_platform_tags().await?)
}

/// Gets cached tag bundle from the database
#[tauri::command]
pub async fn tags_get_tag_bundle() -> Result<TagBundle> {
    Ok(theseus::tags::get_tag_bundle().await?)
}
