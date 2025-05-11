use tauri::{AppHandle, Runtime};
use tauri_plugin_opener::OpenerExt;
use theseus::profile::get_full_path;
use theseus::screenshots::{self, Screenshot, get_valid_screenshot_path};

pub fn init<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("screenshots")
        .invoke_handler(tauri::generate_handler![
            get_all_profile_screenshots,
            get_screenshot_data,
            delete_profile_screenshot,
            open_profile_screenshot
        ])
        .build()
}

#[tauri::command]
pub async fn get_all_profile_screenshots(
    path: &str,
) -> crate::api::Result<Vec<Screenshot>> {
    Ok(screenshots::get_all_profile_screenshots(path).await?)
}

#[tauri::command]
pub async fn get_screenshot_data(
    path: &str,
    screenshot: Screenshot,
) -> crate::api::Result<Option<String>> {
    let profile_dir = get_full_path(path).await?;
    Ok(screenshots::get_screenshot_data(&profile_dir, &screenshot).await?)
}

#[tauri::command]
pub async fn delete_profile_screenshot(
    path: &str,
    screenshot: Screenshot,
) -> crate::api::Result<bool> {
    Ok(screenshots::delete_profile_screenshot(path, &screenshot).await?)
}

#[tauri::command]
pub async fn open_profile_screenshot<R: Runtime>(
    app: AppHandle<R>,
    path: &str,
    screenshot: Screenshot,
) -> crate::api::Result<bool> {
    let profile_dir = get_full_path(path).await?;
    if let Some(path) =
        get_valid_screenshot_path(&profile_dir, &screenshot).await?
    {
        app.opener().reveal_item_in_dir(path).unwrap();
        Ok(true)
    } else {
        Ok(false)
    }
}
