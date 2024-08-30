use serde::Serialize;
use tauri::plugin::TauriPlugin;
use tauri::{
    Emitter, LogicalPosition, LogicalSize, Manager, Runtime, WebviewUrl,
};
use tokio::sync::RwLock;

pub struct AdsState {
    pub size: Option<LogicalSize<f32>>,
    pub position: Option<LogicalPosition<f32>>,
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::<R>::new("ads")
        .setup(|app, _api| {
            app.manage(RwLock::new(AdsState {
                size: None,
                position: None,
            }));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            init_ads_window,
            hide_ads_window,
            scroll_ads_window,
            show_ads_window,
        ])
        .build()
}

const LINK_SCRIPT: &str = include_str!("ads-init.js");

// TODO: make ads work on linux

#[tauri::command]
pub async fn init_ads_window<R: Runtime>(
    app: tauri::AppHandle<R>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> crate::api::Result<()> {
    #[cfg(not(target_os = "linux"))]
    {
        let mut state = app.state::<RwLock<AdsState>>();
        {
            let mut state = state.write().await;
            state.size = Some(LogicalSize::new(width, height));
            state.position = Some(LogicalPosition::new(x, y));
        }

        if let Some(webview) = app.webviews().get("ads-window") {
            let _ = webview.set_position(LogicalPosition::new(x, y));
            let _ = webview.set_size(LogicalSize::new(width, height));
        } else if let Some(window) = app.get_window("main") {
            let _ = window.add_child(
                tauri::webview::WebviewBuilder::new(
                    "ads-window",
                    WebviewUrl::External(
                        "https://modrinth.com/wrapper/app-ads".parse().unwrap(),
                    ),
                )
                .initialization_script(LINK_SCRIPT)
                .user_agent("ModrinthApp Ads Webview")
                .zoom_hotkeys_enabled(false)
                .transparent(true),
                LogicalPosition::new(x, y),
                LogicalSize::new(width, height),
            );
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn show_ads_window<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> crate::api::Result<()> {
    #[cfg(not(target_os = "linux"))]
    {
        if let Some(webview) = app.webviews().get("ads-window") {
            let state = app.state::<RwLock<AdsState>>();
            let state = state.read().await;

            if let Some(size) = state.size {
                let _ = webview.set_size(size);
            }

            if let Some(position) = state.position {
                let _ = webview.set_position(position);
            }
        }
    }

    Ok(())
}


#[tauri::command]
pub async fn hide_ads_window<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> crate::api::Result<()> {
    if let Some(webview) = app.webviews().get("ads-window") {
        let _ = webview.set_position(LogicalPosition::new(-1000, -1000));
    }

    Ok(())
}

#[derive(Serialize, Clone)]
struct ScrollEvent {
    scroll: f32,
}

#[tauri::command]
pub async fn scroll_ads_window<R: Runtime>(
    app: tauri::AppHandle<R>,
    scroll: f32,
) -> crate::api::Result<()> {
    let _ = app.emit("ads-scroll", ScrollEvent { scroll });

    Ok(())
}
