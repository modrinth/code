use crate::api::Result;
use std::process::Command;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("utils")
        .invoke_handler(tauri::generate_handler![
            should_disable_mouseover,
            show_in_folder,
            progress_bars_list,
        ])
        .build()
}

// Lists active progress bars
// Create a new HashMap with the same keys
// Values provided should not be used directly, as they are not guaranteed to be up-to-date
#[tauri::command]
pub async fn progress_bars_list(
) -> Result<std::collections::HashMap<uuid::Uuid, theseus::LoadingBar>> {
    let res = theseus::EventState::list_progress_bars().await?;
    Ok(res)
}

// cfg only on mac os
// disables mouseover and fixes a random crash error only fixed by recent versions of macos
#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn should_disable_mouseover() -> bool {
    // We try to match version to 12.2 or higher. If unrecognizable to pattern or lower, we default to the css with disabled mouseover for safety
    let os = os_info::get();
    if let os_info::Version::Semantic(major, minor, _) = os.version() {
        if *major >= 12 && *minor >= 3 {
            // Mac os version is 12.3 or higher, we allow mouseover
            return false;
        }
    }
    true
}
#[cfg(not(target_os = "macos"))]
#[tauri::command]
pub async fn should_disable_mouseover() -> bool {
    false
}

#[tauri::command]
pub fn show_in_folder(path: String) -> Result<()> {
    {
        #[cfg(target_os = "windows")]
        {
            Command::new("explorer")
                .args(["/select,", &path]) // The comma after select is not a typo
                .spawn()?;
        }

        #[cfg(target_os = "linux")]
        {
            use std::fs::metadata;
            use std::path::PathBuf;

            if path.contains(",") {
                // see https://gitlab.freedesktop.org/dbus/dbus/-/issues/76
                let new_path = match metadata(&path)?.is_dir() {
                    true => path.clone(),
                    false => {
                        let mut path2 = PathBuf::from(path.clone());
                        path2.pop();
                        path2.to_string_lossy().to_string()
                    }
                };
                Command::new("xdg-open").arg(&new_path).spawn()?;
            } else {
                Command::new("xdg-open").arg(&path).spawn()?;
            }
        }

        #[cfg(target_os = "macos")]
        {
            Command::new("open").args(["-R", &path]).spawn()?;
        }

        Ok::<(), theseus::Error>(())
    }?;

    Ok(())
}
