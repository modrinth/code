use serde::{Deserialize, Serialize};
use tauri::webview::{NewWindowResponse, WebviewBuilder};
use tauri::{Manager, PhysicalPosition, PhysicalSize, Runtime, WebviewUrl};
use tauri_plugin_opener::OpenerExt;
use theseus::{
    handler,
    prelude::{CommandPayload, DirectoryInfo},
};

use crate::api::{Result, TheseusSerializableError};
use dashmap::DashMap;
use std::path::{Path, PathBuf};
use theseus::prelude::canonicalize;
use url::Url;

pub fn init<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("utils")
        .invoke_handler(tauri::generate_handler![
            get_os,
            is_network_metered,
            should_disable_mouseover,
            highlight_in_folder,
            open_path,
            open_video_overlay,
            close_video_overlay,
            show_launcher_logs_folder,
            progress_bars_list,
            get_opening_command
        ])
        .build()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::enum_variant_names)]
pub enum OS {
    Windows,
    Linux,
    MacOS,
}

/// Gets OS
#[tauri::command]
pub fn get_os() -> OS {
    #[cfg(target_os = "windows")]
    let os = OS::Windows;
    #[cfg(target_os = "linux")]
    let os = OS::Linux;
    #[cfg(target_os = "macos")]
    let os = OS::MacOS;
    os
}

#[tauri::command]
pub async fn is_network_metered() -> Result<bool> {
    Ok(theseus::prelude::is_network_metered().await?)
}

// Lists active progress bars
// Create a new HashMap with the same keys
// Values provided should not be used directly, as they are not guaranteed to be up-to-date
#[tauri::command]
pub async fn progress_bars_list()
-> Result<DashMap<uuid::Uuid, theseus::LoadingBar>> {
    let res = theseus::EventState::list_progress_bars().await?;
    Ok(res)
}

// disables mouseover and fixes a random crash error only fixed by recent versions of macos
#[tauri::command]
pub async fn should_disable_mouseover() -> bool {
    if cfg!(target_os = "macos") {
        // We try to match version to 12.2 or higher. If unrecognizable to pattern or lower, we default to the css with disabled mouseover for safety
        if let tauri_plugin_os::Version::Semantic(major, minor, _) =
            tauri_plugin_os::version()
            && major >= 12
            && minor >= 3
        {
            // Mac os version is 12.3 or higher, we allow mouseover
            return false;
        }
        true
    } else {
        // Not macos, we allow mouseover
        false
    }
}

#[tauri::command]
pub async fn highlight_in_folder<R: Runtime>(
    app: tauri::AppHandle<R>,
    path: PathBuf,
) {
    tauri::async_runtime::spawn_blocking(move || {
        if let Err(e) = app.opener().reveal_item_in_dir(path) {
            tracing::error!("Failed to highlight file in folder: {}", e);
        }
    })
    .await
    .ok();
}

#[tauri::command]
pub async fn open_path<R: Runtime>(app: tauri::AppHandle<R>, path: PathBuf) {
    tauri::async_runtime::spawn_blocking(move || {
        if let Err(e) =
            app.opener().open_path(path.to_string_lossy(), None::<&str>)
        {
            tracing::error!("Failed to open path: {}", e);
        }
    })
    .await
    .ok();
}

const VIDEO_WEBVIEW_LABEL: &str = "video-overlay";

fn is_valid_video_id(video_id: &str) -> bool {
    !video_id.is_empty()
        && video_id.len() <= 16
        && video_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// Computes the centered video rect (16:9) within the main window, leaving a
/// margin so the surrounding backdrop and the close button stay visible.
fn video_overlay_rect(
    window_size: PhysicalSize<u32>,
) -> (PhysicalPosition<i32>, PhysicalSize<u32>) {
    let win_w = window_size.width as f32;
    let win_h = window_size.height as f32;

    // Use at most 80% of each dimension, keeping a 16:9 aspect ratio.
    let max_w = win_w * 0.8;
    let max_h = win_h * 0.8;
    let mut width = max_w;
    let mut height = width * 9.0 / 16.0;
    if height > max_h {
        height = max_h;
        width = height * 16.0 / 9.0;
    }

    let x = ((win_w - width) / 2.0).max(0.0);
    let y = ((win_h - height) / 2.0).max(0.0);

    (
        PhysicalPosition::new(x as i32, y as i32),
        PhysicalSize::new(width as u32, height as u32),
    )
}

/// Opens a YouTube video as an in-app overlay webview centered over the main
/// window.
///
/// Inline YouTube `/embed` iframes fail with "Video player configuration error"
/// (Error 153) inside the app, because the main webview loads from a custom
/// `tauri://localhost` scheme and macOS WKWebView refuses to attach a `Referer`
/// to the cross-origin subframe. Loading the standard `watch?v=` page in a
/// separate child webview gives YouTube a real origin, so playback works. The
/// frontend draws a dimmed backdrop and close button behind/around it.
#[tauri::command]
pub async fn open_video_overlay<R: Runtime>(
    app: tauri::AppHandle<R>,
    video_id: String,
) -> Result<()> {
    if !is_valid_video_id(&video_id) {
        tracing::error!(
            "Refusing to open invalid YouTube video id: {video_id}"
        );
        return Ok(());
    }

    let Some(window) = app.get_window("main") else {
        return Ok(());
    };

    let url: Url = format!("https://www.youtube.com/watch?v={video_id}")
        .parse()
        .map_err(|_| {
            TheseusSerializableError::Theseus(
                theseus::ErrorKind::OtherError(
                    "Failed to parse video URL".to_string(),
                )
                .as_error(),
            )
        })?;

    let (position, size) = video_overlay_rect(window.inner_size()?);

    if let Some(webview) = app.webviews().get(VIDEO_WEBVIEW_LABEL) {
        webview.navigate(url)?;
        webview.set_size(size)?;
        webview.set_position(position)?;
        webview.show().ok();
    } else {
        window.add_child(
            WebviewBuilder::new(VIDEO_WEBVIEW_LABEL, WebviewUrl::External(url))
                .initialization_script_for_all_frames(include_str!(
                    "youtube-theater.js"
                ))
                .zoom_hotkeys_enabled(false)
                .on_new_window(|_, _| NewWindowResponse::Deny),
            position,
            size,
        )?;
    }

    Ok(())
}

/// Hides the in-app video overlay webview (moved offscreen and hidden).
#[tauri::command]
pub async fn close_video_overlay<R: Runtime>(app: tauri::AppHandle<R>) {
    if let Some(webview) = app.webviews().get(VIDEO_WEBVIEW_LABEL) {
        // Navigate away first so audio stops, then hide offscreen.
        if let Ok(url) = "about:blank".parse() {
            let _ = webview.navigate(url);
        }
        webview
            .set_position(PhysicalPosition::new(-10000, -10000))
            .ok();
        webview.hide().ok();
    }
}

#[tauri::command]
pub async fn show_launcher_logs_folder<R: Runtime>(app: tauri::AppHandle<R>) {
    if let Some(d) = DirectoryInfo::global_handle_if_ready() {
        let path = d.launcher_logs_dir().unwrap_or_default();
        // failure to get folder just opens filesystem
        // (ie: if in debug mode only and launcher_logs never created)
        open_path(app, path).await;
    }
}

// Get opening command
// For example, if a user clicks on an .mrpack to open the app.
// This should be called once and only when the app is done booting up and ready to receive a command
// Returns a Command struct- see events.js
#[tauri::command]
#[cfg(target_os = "macos")]
pub async fn get_opening_command(
    state: tauri::State<'_, crate::macos::deep_link::InitialPayload>,
) -> Result<Option<CommandPayload>> {
    let payload = state.payload.lock().await;

    return if let Some(payload) = payload.as_ref() {
        tracing::info!("opening command {payload}");

        Ok(Some(handler::parse_command(payload).await?))
    } else {
        Ok(None)
    };
}

#[tauri::command]
#[cfg(not(target_os = "macos"))]
pub async fn get_opening_command() -> Result<Option<CommandPayload>> {
    // Tauri is not CLI, we use arguments as path to file to call
    let cmd_arg = std::env::args_os().nth(1);

    tracing::info!("opening command {cmd_arg:?}");

    let cmd_arg = cmd_arg.map(|path| path.to_string_lossy().to_string());
    if let Some(cmd) = cmd_arg {
        tracing::debug!("Opening command: {:?}", cmd);
        return Ok(Some(handler::parse_command(&cmd).await?));
    }
    Ok(None)
}

// helper function called when redirected by a weblink (ie: modrith://do-something) or when redirected by a .mrpack file (in which case its a filepath)
// We hijack the deep link library (which also contains functionality for instance-checking)
pub async fn handle_command(command: String) -> Result<()> {
    tracing::info!("handle command: {command}");
    Ok(theseus::handler::parse_and_emit_command(&command).await?)
}

// Remove when (and if) https://github.com/tauri-apps/tauri/issues/12022 is implemented
pub(crate) fn tauri_convert_file_src(path: &Path) -> Result<Url> {
    #[cfg(any(windows, target_os = "android"))]
    const BASE: &str = "http://asset.localhost/";
    #[cfg(not(any(windows, target_os = "android")))]
    const BASE: &str = "asset://localhost/";

    macro_rules! theseus_try {
        ($test:expr) => {
            match $test {
                Ok(val) => val,
                Err(e) => {
                    return Err(TheseusSerializableError::Theseus(e.into()))
                }
            }
        };
    }

    let path = theseus_try!(canonicalize(path));
    let path = path.to_string_lossy();
    let encoded = urlencoding::encode(&path);

    Ok(theseus_try!(Url::parse(&format!("{BASE}{encoded}"))))
}
