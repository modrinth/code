use std::collections::HashSet;
use std::time::{Duration, Instant};
use tauri::plugin::TauriPlugin;
use tauri::{Manager, PhysicalPosition, PhysicalSize, Runtime};
use tauri_plugin_opener::OpenerExt;
use theseus::settings;
use tokio::sync::RwLock;

pub struct AdsState {
    pub shown: bool,
    pub modal_shown: bool,
    pub last_click: Option<Instant>,
    pub malicious_origins: HashSet<String>,
}

const AD_LINK: &str = "https://modrinth.com/wrapper/app-ads-cookie";

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::<R>::new("ads")
        .setup(|app, _api| {
            app.manage(RwLock::new(AdsState {
                shown: true,
                modal_shown: false,
                last_click: None,
                malicious_origins: HashSet::new(),
            }));

            // We refresh the ads window every 5 minutes for performance
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    if let Some(webview) = app.webviews().get_mut("ads-window")
                    {
                        let _ = webview.navigate(AD_LINK.parse().unwrap());
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
            show_ads_window,
            record_ads_click,
            open_link,
            get_ads_personalization,
        ])
        .build()
}

fn get_webview_position<R: Runtime>(
    app: &tauri::AppHandle<R>,
    dpr: f32,
) -> crate::api::Result<(PhysicalPosition<f32>, PhysicalSize<f32>)> {
    let main_window = app.get_window("main").unwrap();

    let width = 300.0 * dpr;
    let height = 250.0 * dpr;

    let main_window_size = main_window.outer_size()?;
    let x = (main_window_size.width as f32) - width;
    let y = (main_window_size.height as f32) - height;

    Ok((
        PhysicalPosition::new(x, y),
        PhysicalSize::new(width, height),
    ))
}

#[tauri::command]
#[cfg(not(target_os = "linux"))]
pub async fn init_ads_window<R: Runtime>(
    app: tauri::AppHandle<R>,
    dpr: f32,
    override_shown: bool,
) -> crate::api::Result<()> {
    use tauri::WebviewUrl;

    let state = app.state::<RwLock<AdsState>>();
    let mut state = state.write().await;

    if override_shown {
        state.shown = true;
    }

    if state.modal_shown {
        return Ok(());
    }

    if let Ok((position, size)) = get_webview_position(&app, dpr) {
        if let Some(webview) = app.webviews().get("ads-window") {
            if state.shown {
                let _ = webview.set_position(position);
                let _ = webview.set_size(size);
            } else {
                let _ =
                    webview.set_position(PhysicalPosition::new(-1000, -1000));
            }
        } else if let Some(window) = app.get_window("main") {
            let webview = window.add_child(
                tauri::webview::WebviewBuilder::new(
                    "ads-window",
                    WebviewUrl::External(
                        AD_LINK.parse().unwrap(),
                    ),
                )
                .initialization_script_for_all_frames(include_str!("ads-init.js"))
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36")
                .zoom_hotkeys_enabled(false)
                .transparent(true)
                .on_new_window(|_, _| tauri::webview::NewWindowResponse::Deny),
                if state.shown {
                    position
                } else {
                    PhysicalPosition::new(-1000.0, -1000.0)
                },
                size,
            )?;

            webview.with_webview(#[allow(unused_variables)] |webview2| {
                #[cfg(windows)]
                {
                    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2_8;
                    use windows_core::Interface;

                    let webview2_controller = webview2.controller();
                    let Ok(webview2_8) = unsafe { webview2_controller.CoreWebView2() }
                        .and_then(|core_webview2| core_webview2.cast::<ICoreWebView2_8>())
                    else {
                        return;
                    };

                    unsafe { webview2_8.SetIsMuted(true) }.ok();
                }
            })?;
        }
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
    dpr: f32,
) -> crate::api::Result<()> {
    if let Some(webview) = app.webviews().get("ads-window") {
        let state = app.state::<RwLock<AdsState>>();
        let mut state = state.write().await;

        state.modal_shown = false;

        if state.shown {
            let (position, size) = get_webview_position(&app, dpr)?;
            let _ = webview.set_size(size);
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

        if reset.unwrap_or(false) {
            state.shown = false;
        } else {
            state.modal_shown = true;
        }

        let _ = webview.set_position(PhysicalPosition::new(-1000, -1000));
    }

    Ok(())
}

#[tauri::command]
pub async fn record_ads_click<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> crate::api::Result<()> {
    let state = app.state::<RwLock<AdsState>>();

    let mut state = state.write().await;
    state.last_click = Some(Instant::now());

    Ok(())
}

#[tauri::command]
pub async fn open_link<R: Runtime>(
    app: tauri::AppHandle<R>,
    path: String,
    origin: String,
) -> crate::api::Result<()> {
    let state = app.state::<RwLock<AdsState>>();
    let mut state = state.write().await;

    if url::Url::parse(&path).is_ok()
        && !state.malicious_origins.contains(&origin)
        && let Some(last_click) = state.last_click
        && last_click.elapsed() < Duration::from_millis(100)
    {
        let _ = app.opener().open_url(&path, None::<String>);
        state.last_click = None;

        return Ok(());
    }

    tracing::info!("Malicious click: {path} origin {origin}");
    state.malicious_origins.insert(origin);

    Ok(())
}

#[tauri::command]
pub async fn get_ads_personalization() -> crate::api::Result<bool> {
    let res = settings::get().await?;
    Ok(res.personalized_ads)
}
