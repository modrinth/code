use serde::Serialize;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use tauri::{plugin::TauriPlugin, Emitter, Listener, LogicalPosition, LogicalSize, Manager, Runtime};
use tauri_plugin_shell::{open, ShellExt};
use tokio::sync::RwLock;

const AD_REFRESH_INTERVAL_SECS: u64 = 300; // 5 minutes
const ADS_URL: &str = "https://modrinth.com/wrapper/app-ads-cookie";
const HIDDEN_POSITION: LogicalPosition<f32> = LogicalPosition::new(-1000.0, -1000.0);

pub struct AdsState {
    pub shown: bool,
    pub size: Option<LogicalSize<f32>>,
    pub position: Option<LogicalPosition<f32>>,
    pub last_click: Option<Instant>,
    pub malicious_origins: HashSet<String>,
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::<R>::new("ads")
        .setup(|app, _api| {
            app.manage(RwLock::new(AdsState {
                shown: true,
                size: None,
                position: None,
                last_click: None,
                malicious_origins: HashSet::new(),
            }));

            setup_ad_refresh_loop(app.clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            init_ads_window,
            hide_ads_window,
            scroll_ads_window,
            show_ads_window,
            record_ads_click,
            open_link,
        ])
        .build()
}

fn setup_ad_refresh_loop<R: Runtime>(app: tauri::AppHandle<R>) {
    tauri::async_runtime::spawn(async move {
        loop {
            if let Some(webview) = app.webviews().get_mut("ads-window") {
                if let Ok(url) = ADS_URL.parse() {
                    let _ = webview.navigate(url);
                }
            }
            tokio::time::sleep(Duration::from_secs(AD_REFRESH_INTERVAL_SECS)).await;
        }
    });
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
    const LINK_SCRIPT: &str = include_str!("ads-init.js");

    let mut state = app.state::<RwLock<AdsState>>().write().await;
    state.size = Some(LogicalSize::new(width, height));
    state.position = Some(LogicalPosition::new(x, y));
    if override_shown {
        state.shown = true;
    }

    let window_position = if state.shown {
        LogicalPosition::new(x, y)
    } else {
        HIDDEN_POSITION
    };

    if let Some(webview) = app.webviews().get("ads-window") {
        apply_webview_settings(webview, &state);
    } else {
        create_ads_webview(app, window_position, width, height).await?;
    }

    Ok(())
}

async fn create_ads_webview<R: Runtime>(
    app: tauri::AppHandle<R>,
    position: LogicalPosition<f32>,
    width: f32,
    height: f32,
) -> crate::api::Result<()> {
    if let Some(window) = app.get_window("main") {
        let webview = tauri::webview::WebviewBuilder::new("ads-window", ADS_URL.parse().unwrap())
            .initialization_script(LINK_SCRIPT)
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
            .zoom_hotkeys_enabled(false)
            .transparent(true)
            .build()?;

        window.add_child(webview, position, LogicalSize::new(width, height))?;

        webview.listen_any("click", |event| {
            println!("click: {:?}", event);
        });
    }
    Ok(())
}

fn apply_webview_settings(webview: &mut tauri::Webview, state: &AdsState) {
    if let Some(size) = state.size {
        let _ = webview.set_size(size);
    }
    if let Some(position) = state.position {
        let _ = webview.set_position(position);
    }
}

#[tauri::command]
pub async fn show_ads_window<R: Runtime>(app: tauri::AppHandle<R>) -> crate::api::Result<()> {
    let mut state = app.state::<RwLock<AdsState>>().write().await;
    state.shown = true;

    if let Some(webview) = app.webviews().get("ads-window") {
        apply_webview_settings(webview, &state);
    }

    Ok(())
}

#[tauri::command]
pub async fn hide_ads_window<R: Runtime>(
    app: tauri::AppHandle<R>,
    reset: Option<bool>,
) -> crate::api::Result<()> {
    let mut state = app.state::<RwLock<AdsState>>().write().await;
    state.shown = false;

    if reset.unwrap_or(false) {
        state.size = None;
        state.position = None;
    }

    if let Some(webview) = app.webviews().get("ads-window") {
        let _ = webview.set_position(HIDDEN_POSITION);
    }

    Ok(())
}

#[tauri::command]
pub async fn scroll_ads_window<R: Runtime>(app: tauri::AppHandle<R>, scroll: f32) -> crate::api::Result<()> {
    app.emit("ads-scroll", ScrollEvent { scroll })?;
    Ok(())
}

#[tauri::command]
pub async fn record_ads_click<R: Runtime>(app: tauri::AppHandle<R>) -> crate::api::Result<()> {
    let mut state = app.state::<RwLock<AdsState>>().write().await;
    state.last_click = Some(Instant::now());
    Ok(())
}

#[tauri::command]
pub async fn open_link<R: Runtime>(
    app: tauri::AppHandle<R>,
    path: String,
    origin: String,
) -> crate::api::Result<()> {
    let mut state = app.state::<RwLock<AdsState>>().write().await;

    if url::Url::parse(&path).is_ok() && !state.malicious_origins.contains(&origin) {
        if let Some(last_click) = state.last_click {
            if last_click.elapsed() < Duration::from_millis(100) {
                app.shell().open(&path, None)?;
                state.last_click = None;
                return Ok(());
            }
        }
    }

    tracing::info!("Malicious click: {path} origin {origin}");
    state.malicious_origins.insert(origin);

    Ok(())
}

