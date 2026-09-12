use crate::api::Result;
use tauri::Runtime;
use theseus::prelude::*;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("settings")
        .invoke_handler(tauri::generate_handler![
            settings_get,
            settings_set,
            store_usage,
            store_cleanup,
            store_set_cache_limit,
            store_verify,
            cancel_directory_change
        ])
        .build()
}

#[tauri::command]
pub async fn store_usage() -> Result<settings::StoreUsage> {
    Ok(settings::store_usage().await?)
}

#[tauri::command]
pub async fn store_cleanup() -> Result<u64> {
    Ok(settings::store_cleanup().await?)
}

#[tauri::command]
pub async fn store_set_cache_limit(bytes: u64) -> Result<()> {
    Ok(settings::store_set_cache_limit(bytes).await?)
}

#[tauri::command]
pub async fn store_verify(
	repair: bool,
	on_progress: tauri::ipc::Channel<(u64, u64)>,
) -> Result<settings::StoreVerification> {
	let last_sent = std::sync::Mutex::new(std::time::Instant::now() - std::time::Duration::from_secs(1));
	Ok(settings::store_verify_with_progress(repair, &|current, total| {
		let mut last = last_sent.lock().unwrap();
		if current == 0 || current >= total || last.elapsed() >= std::time::Duration::from_millis(200) {
			let _ = on_progress.send((current, total));
			*last = std::time::Instant::now();
		}
	}).await?)
}

// Get full settings
// invoke('plugin:settings|settings_get')
#[tauri::command]
pub async fn settings_get() -> Result<Settings> {
    let res = settings::get().await?;
    Ok(res)
}

// Set full settings
// invoke('plugin:settings|settings_set', settings)
#[tauri::command]
pub async fn settings_set(settings: Settings) -> Result<()> {
    settings::set(settings).await?;
    Ok(())
}

#[tauri::command]
pub async fn cancel_directory_change<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<()> {
    let identifier = &app.config().identifier;
    settings::cancel_directory_change(identifier).await?;
    Ok(())
}
