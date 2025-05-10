use tauri::{AppHandle, Runtime};
use theseus::{screenshots};

pub fn init<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("screenshots")
        .invoke_handler(tauri::generate_handler![
            get_all_profile_screenshots
        ])
        .build()
}

#[tauri::command]
pub async fn get_all_profile_screenshots<R: Runtime>(
    app_handle: AppHandle<R>,
    path: &str,
) -> crate::api::Result<Vec<screenshots::Screenshot>> {
    Ok(screenshots::get_all_profile_screenshots(path).await?)
}