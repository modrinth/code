use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
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
#[cfg(not(target_os = "linux"))]
const ADS_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";

#[cfg(windows)]
fn ads_user_agent_override_params() -> String {
    serde_json::json!({
        "userAgent": ADS_USER_AGENT,
        "platform": "Win32",
        "userAgentMetadata": {
            "brands": [
                { "brand": "Chromium", "version": "128" },
                { "brand": "Google Chrome", "version": "128" },
                { "brand": "Not=A?Brand", "version": "99" },
            ],
            "fullVersion": "128.0.0.0",
            "fullVersionList": [
                { "brand": "Chromium", "version": "128.0.0.0" },
                { "brand": "Google Chrome", "version": "128.0.0.0" },
                { "brand": "Not=A?Brand", "version": "99.0.0.0" },
            ],
            "platform": "Windows",
            "platformVersion": "10.0.0",
            "architecture": "x86",
            "bitness": "64",
            "model": "",
            "mobile": false,
        },
    })
    .to_string()
}

#[cfg(windows)]
fn configure_ads_cookie_settings(
    core_webview2: &webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2,
) {
    use webview2_com::Microsoft::Web::WebView2::Win32::{
        COREWEBVIEW2_TRACKING_PREVENTION_LEVEL_NONE, ICoreWebView2,
        ICoreWebView2_13, ICoreWebView2Profile3,
    };
    use windows_core::Interface;

    match core_webview2
        .cast::<ICoreWebView2_13>()
        .and_then(|core_webview2| unsafe { core_webview2.Profile() })
        .and_then(|profile| profile.cast::<ICoreWebView2Profile3>())
    {
        Ok(profile) => {
            if let Err(error) = unsafe {
                profile.SetPreferredTrackingPreventionLevel(
                    COREWEBVIEW2_TRACKING_PREVENTION_LEVEL_NONE,
                )
            } {
                tracing::warn!(
                    ?error,
                    "Failed to disable ads WebView2 tracking prevention"
                );
            }
        }
        Err(error) => {
            tracing::warn!(
                ?error,
                "Failed to access ads WebView2 profile tracking prevention settings"
            );
        }
    }
}

fn set_webview_visible<R: Runtime>(
    webview: &tauri::Webview<R>,
    _visible: bool,
) {
    webview
        .with_webview(
            #[allow(unused_variables)]
            move |wv| {
                #[cfg(windows)]
                {
                    let controller = wv.controller();
                    unsafe { controller.SetIsVisible(_visible) }.ok();
                }
            },
        )
        .ok();
}

fn set_webview_visible_for_window<R: Runtime>(
    app: &tauri::AppHandle<R>,
    webview: &tauri::Webview<R>,
    visible: bool,
) {
    let is_minimized = app
        .get_window("main")
        .and_then(|window| window.is_minimized().ok())
        .unwrap_or(false);

    set_webview_visible(webview, visible && !is_minimized);
}

fn sync_webview_visibility_for_main_window<R: Runtime>(
    app: &tauri::AppHandle<R>,
    main_window: &tauri::Window<R>,
    was_minimized: &AtomicBool,
) {
    let is_minimized = main_window.is_minimized().unwrap_or(false);
    let was = was_minimized.load(Ordering::SeqCst);

    if is_minimized == was {
        return;
    }

    was_minimized.store(is_minimized, Ordering::SeqCst);

    let ads_visible = if is_minimized {
        false
    } else {
        match app.state::<RwLock<AdsState>>().try_read() {
            Ok(state) => state.shown && !state.modal_shown,
            Err(_) => false,
        }
    };

    let mut webviews = Vec::new();
    let mut seen_webviews = HashSet::new();

    for webview in main_window.webviews() {
        seen_webviews.insert(webview.label().to_string());
        webviews.push(webview);
    }

    for webview in app.webviews().into_values() {
        if seen_webviews.insert(webview.label().to_string()) {
            webviews.push(webview);
        }
    }

    for webview in webviews {
        let visible =
            !is_minimized && (webview.label() != "ads-window" || ads_visible);

        set_webview_visible(&webview, visible);
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::<R>::new("ads")
        .setup(|app, _api| {
            app.manage(RwLock::new(AdsState {
                shown: true,
                modal_shown: false,
                last_click: None,
                malicious_origins: HashSet::new(),
            }));

            // We refresh the ads window every 5 minutes to mitigate memory leak issues.
            // While this loop doesn't include explicit checks to see if the window is still
            // visible when we refresh, the Aditude wrapper will not make any ad requests
            // unless Chromium reports the page as visible. The refresh does not reset the
            // visibility state.
            let refresh_app = app.clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    if let Some(webview) =
                        refresh_app.webviews().get_mut("ads-window")
                    {
                        let _ = webview.navigate(AD_LINK.parse().unwrap());
                    }

                    tokio::time::sleep(std::time::Duration::from_secs(60 * 5))
                        .await;
                }
            });

            if let Some(main_window) = app.get_window("main") {
                let app_handle = app.clone();
                let event_window = main_window.clone();
                let was_minimized = Arc::new(AtomicBool::new(false));

                main_window.on_window_event(move |_| {
                    sync_webview_visibility_for_main_window(
                        &app_handle,
                        &event_window,
                        &was_minimized,
                    );

                    let delayed_app_handle = app_handle.clone();
                    let delayed_event_window = event_window.clone();
                    let delayed_was_minimized = was_minimized.clone();

                    tauri::async_runtime::spawn(async move {
                        tokio::time::sleep(Duration::from_millis(100)).await;

                        sync_webview_visibility_for_main_window(
                            &delayed_app_handle,
                            &delayed_event_window,
                            &delayed_was_minimized,
                        );
                    });
                });
            }

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
        let webview = if let Some(webview) = app.webviews().get("ads-window") {
            // set both the `hide`/`show` state and `position`,
            // to ensure that the webview is actually shown/hidden
            if state.shown {
                webview.show().ok();
                webview.set_position(position).ok();
                webview.set_size(size).ok();
                set_webview_visible_for_window(&app, webview, true);
            } else {
                webview.hide().ok();
                webview
                    .set_position(PhysicalPosition::new(-1000, -1000))
                    .ok();
                set_webview_visible(webview, false);
            }

            Some(webview.clone())
        } else if let Some(window) = app.get_window("main") {
            #[cfg(windows)]
            let webview_url =
                WebviewUrl::External("about:blank".parse().unwrap());
            #[cfg(not(windows))]
            let webview_url = WebviewUrl::External(AD_LINK.parse().unwrap());

            let webview = window.add_child(
                tauri::webview::WebviewBuilder::new("ads-window", webview_url)
                    .initialization_script_for_all_frames(include_str!(
                        "ads-init.js"
                    ))
                    // We use a standard Chrome user agent for compatibility with our ad provider,
                    // since Tauri is not recognized by ad providers by default.
                    // Aditude has separately informed SSPs and IVT vendors that this traffic
                    // originates from a desktop app.
                    .user_agent(ADS_USER_AGENT)
                    .zoom_hotkeys_enabled(false)
                    .transparent(true)
                    .on_new_window(|_, _| {
                        tauri::webview::NewWindowResponse::Deny
                    }),
                // set both the `hide`/`show` state and `position`,
                // to ensure that the webview is actually shown/hidden
                if state.shown {
                    position
                } else {
                    PhysicalPosition::new(-1000.0, -1000.0)
                },
                size,
            )?;

            if state.shown {
                webview.show().ok();
                set_webview_visible_for_window(&app, &webview, true);
            } else {
                webview.hide().ok();
                set_webview_visible(&webview, false);
            }

            webview.with_webview(#[allow(unused_variables)] |webview2| {
                #[cfg(windows)]
                {
                    use webview2_com::CallDevToolsProtocolMethodCompletedHandler;
                    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2_8;
                    use windows_core::Interface;
                    use windows_core::HSTRING;

                    let core_webview2 =
                        unsafe { webview2.controller().CoreWebView2() };

                    if let Ok(core_webview2) = core_webview2 {
                        configure_ads_cookie_settings(&core_webview2);

                        let navigate_webview = core_webview2.clone();
                        let handler =
                            CallDevToolsProtocolMethodCompletedHandler::create(
                                Box::new(move |result: windows_core::Result<()>, _| {
                                    if let Err(error) = result {
                                        tracing::error!(
                                            ?error,
                                            "Failed to override ads user-agent client hints"
                                        );
                                    }

                                    unsafe {
                                        navigate_webview
                                            .Navigate(&HSTRING::from(AD_LINK))
                                            .ok();
                                    }

                                    Ok(())
                                }) as Box<_>,
                            );

                        unsafe {
                            if let Err(error) = core_webview2
                                .CallDevToolsProtocolMethod(
                                    &HSTRING::from(
                                        "Emulation.setUserAgentOverride",
                                    ),
                                    &HSTRING::from(
                                        ads_user_agent_override_params(),
                                    ),
                                    &handler,
                                )
                            {
                                tracing::error!(
                                    ?error,
                                    "Failed to install ads user-agent client hints override"
                                );

                                core_webview2.Navigate(&HSTRING::from(AD_LINK)).ok();
                            }
                        }
                    }

                    let webview2_controller = webview2.controller();
                    let Ok(webview2_8) = unsafe { webview2_controller.CoreWebView2() }
                        .and_then(|core_webview2| core_webview2.cast::<ICoreWebView2_8>())
                    else {
                        return;
                    };

                    unsafe { webview2_8.SetIsMuted(true) }.ok();
                }
            })?;

            Some(webview)
        } else {
            None
        };

        if webview.is_none() {
            return Ok(());
        }

        // tauri::async_runtime::spawn(async move {
        //     loop {
        //         webview.with_webview(|wv| {
        //             #[cfg(windows)]
        //             {
        //                 use webview2_com::ExecuteScriptCompletedHandler;

        //                 let core_webview2 = unsafe {
        //                     webview.controller().CoreWebView2().unwrap()
        //                 };

        //                 let handler = ExecuteScriptCompletedHandler::create(Box::new(
        //                     move |hr: windows_core::Result<()>, result: String| {
        //                         if hr.is_ok() {
        //                             let hidden: bool = serde_json::from_str(&result).unwrap_or(true);
        //                             tracing::error!("!! ads wv hidden? {}", hidden);
        //                         }
        //                         Ok(())
        //                     },
        //                 ) as Box<_>);

        //                 unsafe {
        //                     let _ = core_webview2.ExecuteScript(
        //                         windows_core::w!("document.hidden"),
        //                         &handler,
        //                     );
        //                 }
        //             }

        //             #[cfg(not(windows))]
        //             {
        //                 use webkit2gtk::WebViewExt;

        //                 wv.inner().evaluate_javascript(
        //                     "document.hidden",
        //                     None,
        //                     None,
        //                     None::<&webkit2gtk::gio::Cancellable>,
        //                     |result| {
        //                         use javascriptcore::ValueExt;

        //                         let hidden = result.map(|v| v.to_boolean());
        //                         tracing::error!("!! ads wv hidden? {hidden:?}");
        //                     },
        //                 );
        //             }
        //         });

        //         tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        //     }
        // });
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
            // set both the `hide`/`show` state and `position`,
            // to ensure that the webview is actually shown/hidden
            webview.set_size(size).ok();
            webview.set_position(position).ok();
            webview.show().ok();
            set_webview_visible_for_window(&app, webview, true);
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

        // set both the `hide`/`show` state and `position`,
        // to ensure that the webview is actually shown/hidden
        webview
            .set_position(PhysicalPosition::new(-1000, -1000))
            .ok();
        webview.hide().ok();
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
