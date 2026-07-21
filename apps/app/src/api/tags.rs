use crate::api::Result;
use theseus::tags::{Category, DonationPlatform, GameVersion, Loader};

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
pub async fn tags_get_categories(
    invocation_context: theseus::InvocationContext,
) -> Result<Vec<Category>> {
    let context = crate::api::operation_context(invocation_context);
    Ok(theseus::tags::get_category_tags(&context).await?)
}

/// Gets cached report type tags from the database
#[tauri::command]
pub async fn tags_get_report_types(
    invocation_context: theseus::InvocationContext,
) -> Result<Vec<String>> {
    let context = crate::api::operation_context(invocation_context);
    Ok(theseus::tags::get_report_type_tags(&context).await?)
}

/// Gets cached loader tags from the database
#[tauri::command]
pub async fn tags_get_loaders(
    invocation_context: theseus::InvocationContext,
) -> Result<Vec<Loader>> {
    let context = crate::api::operation_context(invocation_context);
    Ok(theseus::tags::get_loader_tags(&context).await?)
}

/// Gets cached game version tags from the database
#[tauri::command]
pub async fn tags_get_game_versions(
    invocation_context: theseus::InvocationContext,
) -> Result<Vec<GameVersion>> {
    let context = crate::api::operation_context(invocation_context);
    Ok(theseus::tags::get_game_version_tags(&context).await?)
}

/// Gets cached donation platform tags from the database
#[tauri::command]
pub async fn tags_get_donation_platforms(
    invocation_context: theseus::InvocationContext,
) -> Result<Vec<DonationPlatform>> {
    let context = crate::api::operation_context(invocation_context);
    Ok(theseus::tags::get_donation_platform_tags(&context).await?)
}
