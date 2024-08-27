#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use native_dialog::{MessageDialog, MessageType};
use tauri::{Listener, Manager, PhysicalSize};
use tauri_plugin_window_state::{StateFlags, WindowExt};
use theseus::prelude::*;

mod api;
mod error;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
#[macro_use]
extern crate cocoa;

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

// Should be called in launcher initialization
#[tracing::instrument(skip_all)]
#[tauri::command]
async fn initialize_state(app: tauri::AppHandle) -> api::Result<()> {
    theseus::EventState::init(app.clone()).await?;
    State::init().await?;

    let state = State::get().await?;
    app.asset_protocol_scope()
        .allow_directory(state.directories.caches_dir(), true)?;

    Ok(())
}

// Should be call once Vue has mounted the app
#[tracing::instrument(skip_all)]
#[tauri::command]
fn show_window(app: tauri::AppHandle) {
    let win = app.get_webview_window("main").unwrap();
    if let Err(e) = win.show() {
        MessageDialog::new()
            .set_type(MessageType::Error)
            .set_title("Initialization error")
            .set_text(&format!(
                "Cannot display application window due to an error:\n{}",
                e
            ))
            .show_alert()
            .unwrap();
        panic!("cannot display application window")
    }
}

#[tauri::command]
fn is_dev() -> bool {
    cfg!(debug_assertions)
}

// Toggles decorations
#[tauri::command]
async fn toggle_decorations(b: bool, window: tauri::Window) -> api::Result<()> {
    window.set_decorations(b).map_err(|e| {
        theseus::Error::from(theseus::ErrorKind::OtherError(format!(
            "Failed to toggle decorations: {}",
            e
        )))
    })?;
    Ok(())
}

// if Tauri app is called with arguments, then those arguments will be treated as commands
// ie: deep links or filepaths for .mrpacks
fn main() {
    /*
        tracing is set basd on the environment variable RUST_LOG=xxx, depending on the amount of logs to show
            ERROR > WARN > INFO > DEBUG > TRACE
        eg. RUST_LOG=info will show info, warn, and error logs
            RUST_LOG="theseus=trace" will show *all* messages but from theseus only (and not dependencies using similar crates)
            RUST_LOG="theseus=trace" will show *all* messages but from theseus only (and not dependencies using similar crates)

        Error messages returned to Tauri will display as traced error logs if they return an error.
        This will also include an attached span trace if the error is from a tracing error, and the level is set to info, debug, or trace

        on unix:
            RUST_LOG="theseus=trace" {run command}

    */
    let _log_guard = theseus::start_logger();

    tracing::info!("Initialized tracing subscriber. Loading Modrinth App!");

    let mut builder = tauri::Builder::default();
    builder = builder
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            let res = {
                use macos::deep_link::InitialPayload;
                let mtx = std::sync::Arc::new(tokio::sync::Mutex::new(None));

                app.manage(InitialPayload {
                    payload: mtx.clone(),
                });

                let mtx_copy = mtx.clone();
                app.listen("deep-link://new-url", move |url| {
                    let mtx_copy_copy = mtx_copy.clone();
                    let request = url.payload().to_owned();

                    tauri::async_runtime::spawn(async move {
                        tracing::info!("Handling deep link {request}");

                        let mut payload = mtx_copy_copy.lock().await;
                        if payload.is_none() {
                            *payload = Some(request.clone());
                        }

                        let _ = api::utils::handle_command(request).await;
                    });
                });
            };

            #[cfg(not(target_os = "macos"))]
            app.listen("deep-link://new-url", |url| {
                let payload = url.payload().to_owned();
                tracing::info!("Handling deep link {payload}");
                tauri::async_runtime::spawn(api::utils::handle_command(
                    payload,
                ));
                dbg!(url);
            });

            if let Some(window) = app.get_window("main") {
                // Hide window to prevent white flash on startup
                let _ = window.hide();

                #[cfg(not(target_os = "linux"))]
                {
                    window.set_shadow(true).unwrap();
                }
            }

            Ok(())
        });

    let mut builder = builder
        .plugin(api::auth::init())
        .plugin(api::mr_auth::init())
        .plugin(api::import::init())
        .plugin(api::logs::init())
        .plugin(api::jre::init())
        .plugin(api::metadata::init())
        .plugin(api::pack::init())
        .plugin(api::process::init())
        .plugin(api::profile::init())
        .plugin(api::profile_create::init())
        .plugin(api::settings::init())
        .plugin(api::tags::init())
        .plugin(api::utils::init())
        .plugin(api::cache::init())
        .invoke_handler(tauri::generate_handler![
            initialize_state,
            is_dev,
            toggle_decorations,
            api::mr_auth::modrinth_auth_login,
            show_window,
        ]);

    #[cfg(target_os = "macos")] {
        builder = builder.plugin(macos::window_ext::init());
    }

    let app = builder.build(tauri::generate_context!());

    match app {
        Ok(app) => {
            #[allow(unused_variables)]
            app.run(|app, event| {
                #[cfg(any(target_os = "macos"))]
                if let tauri::RunEvent::Opened { urls } = event {
                    tracing::info!("Handling webview open {urls:?}");

                    let file = urls
                        .into_iter()
                        .filter_map(|url| url.to_file_path().ok())
                        .next();

                    if let Some(file) = file {
                        use macos::deep_link::InitialPayload;
                        let initial_payload = app.state::<InitialPayload>();
                        let request = file.to_string_lossy().to_string();

                        let mtx_copy = initial_payload.payload.clone();
                        tauri::async_runtime::spawn(async move {
                            let mut payload = mtx_copy.lock().await;
                            if payload.is_none() {
                                *payload = Some(request.clone());
                            }

                            let _ = api::utils::handle_command(request).await;
                        });
                    }
                }
            });
        },
        Err(e) => {
            #[cfg(target_os = "windows")]
            {
                // tauri doesn't expose runtime errors, so matching a string representation seems like the only solution
                if format!("{:?}", e)
                    .contains("Runtime(CreateWebview(WebView2Error(WindowsError")
                {
                    MessageDialog::new()
                        .set_type(MessageType::Error)
                        .set_title("Initialization error")
                        .set_text("Your Microsoft Edge WebView2 installation is corrupt.\n\nMicrosoft Edge WebView2 is required to run Modrinth App.\n\nLearn how to repair it at https://docs.modrinth.com/faq/app/webview2")
                        .show_alert()
                        .unwrap();

                    panic!("webview2 initialization failed")
                }
            }

            MessageDialog::new()
                .set_type(MessageType::Error)
                .set_title("Initialization error")
                .set_text(&format!(
                    "Cannot initialize application due to an error:\n{:?}",
                    e
                ))
                .show_alert()
                .unwrap();

            panic!("{1}: {:?}", e, "error while running tauri application")
        }
    }
}
