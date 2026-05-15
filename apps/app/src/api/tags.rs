use crate::api::Result;
use pteron::tags::{Category, DonationPlatform, GameVersion, Loader};

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("tags")
        .invoke_handler(tauri::generate_handler![
            tags_get_categories,
            tags_get_report_types,
            tags_get_loaders,
            tags_get_game_versions,
            tags_get_donation_platforms,
        ])
        .build()
}

/// Gets cached category tags from the database
#[tauri::command]
pub async fn tags_get_categories() -> Result<Vec<Category>> {
    Ok(pteron::tags::get_category_tags().await?)
}

/// Gets cached report type tags from the database
#[tauri::command]
pub async fn tags_get_report_types() -> Result<Vec<String>> {
    Ok(pteron::tags::get_report_type_tags().await?)
}

/// Gets cached loader tags from the database
#[tauri::command]
pub async fn tags_get_loaders() -> Result<Vec<Loader>> {
    Ok(pteron::tags::get_loader_tags().await?)
}

/// Gets cached game version tags from the database
#[tauri::command]
pub async fn tags_get_game_versions() -> Result<Vec<GameVersion>> {
    Ok(pteron::tags::get_game_version_tags().await?)
}

/// Gets cached donation platform tags from the database
#[tauri::command]
pub async fn tags_get_donation_platforms() -> Result<Vec<DonationPlatform>> {
    Ok(pteron::tags::get_donation_platform_tags().await?)
}
