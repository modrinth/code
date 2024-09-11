use serde::Serialize;
use tauri::plugin::TauriPlugin;
use tauri::{Emitter, LogicalPosition, LogicalSize, Manager, Runtime};
use tokio::sync::RwLock;

pub struct AdsState {
    pub shown: bool,
    pub size: Option<LogicalSize<f32>>,
    pub position: Option<LogicalPosition<f32>>,
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::<R>::new("ads")
        .setup(|app, _api| {
            app.manage(RwLock::new(AdsState {
                shown: true,
                size: None,
                position: None,
            }));

            // We refresh the ads window every 5 minutes for performance
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    if let Some(webview) = app.webviews().get_mut("ads-window")
                    {
                        let _ = webview.navigate(
                            "https://modrinth.com/wrapper/app-ads-cookie"
                                .parse()
                                .unwrap(),
                        );
                    }

                    tokio::time::sleep(std::time::Duration::from_secs(60 * 5))
                        .await;
                }
            });

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

#[tauri::command]
#[cfg(not(target_os = "linux"))]
pub async fn init_ads_window<R: Runtime>(
    app: tauri::AppHandle<R>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    override_shown: bool,
) -> crate::api::Result<()> {
    use tauri::WebviewUrl;
    const LINK_SCRIPT: &str = include_str!("ads-init.js");

    let state = app.state::<RwLock<AdsState>>();
    let mut state = state.write().await;
    state.size = Some(LogicalSize::new(width, height));
    state.position = Some(LogicalPosition::new(x, y));

    if override_shown {
        state.shown = true;
    }

    if let Some(webview) = app.webviews().get("ads-window") {
        if state.shown {
            let _ = webview.set_position(LogicalPosition::new(x, y));
            let _ = webview.set_size(LogicalSize::new(width, height));
        }
    } else if let Some(window) = app.get_window("main") {
        let _ = window.add_child(
            tauri::webview::WebviewBuilder::new(
                "ads-window",
                WebviewUrl::External(
                    "https://modrinth.com/wrapper/app-ads-cookie".parse().unwrap(),
                ),
            )
            .initialization_script(LINK_SCRIPT)
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36")
            .zoom_hotkeys_enabled(false)
            .transparent(true),
            if state.shown {
                LogicalPosition::new(x, y)
            } else {
                LogicalPosition::new(-1000.0, -1000.0)
            },
            LogicalSize::new(width, height),
        );
    }

    Ok(())
}

// TODO: make ads work on linux
#[tauri::command]
#[cfg(target_os = "linux")]
pub async fn init_ads_window() {}

#[tauri::command]
pub async fn show_ads_window<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> crate::api::Result<()> {
    if let Some(webview) = app.webviews().get("ads-window") {
        let state = app.state::<RwLock<AdsState>>();
        let mut state = state.write().await;

        state.shown = true;
        if let Some(size) = state.size {
            let _ = webview.set_size(size);
        }

        if let Some(position) = state.position {
            let _ = webview.set_position(position);
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn hide_ads_window<R: Runtime>(
    app: tauri::AppHandle<R>,
    reset: Option<bool>,
) -> crate::api::Result<()> {
    if let Some(webview) = app.webviews().get("ads-window") {
        let state = app.state::<RwLock<AdsState>>();
        let mut state = state.write().await;
        state.shown = false;

        if reset.unwrap_or(false) {
            state.size = None;
            state.position = None;
        }

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
