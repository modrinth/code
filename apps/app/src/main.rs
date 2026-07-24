#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![recursion_limit = "256"]

use native_dialog::{DialogBuilder, MessageLevel};
use std::env;
use tauri::{Listener, Manager};
use tauri_plugin_fs::FsExt;
use theseus::prelude::*;

mod api;
mod error;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(feature = "updater")]
mod updater_impl;

#[cfg(not(feature = "updater"))]
mod updater_impl_noop;

/// Returns `true` if an NVIDIA GPU is present and active on this Linux system.
///
/// Two detection strategies are used in order:
///
/// - `/dev/nvidia0` exists — the proprietary driver creates this node when it
///   has claimed the GPU.
/// - A DRM sysfs vendor ID of `0x10de` (NVIDIA's PCI vendor) is found under
///   `/sys/class/drm/*/device/vendor` — covers the open kernel module and
///   systems where the device node name differs.
#[cfg(target_os = "linux")]
fn detect_nvidia_gpu() -> bool {
    use std::path::Path;

    if Path::new("/dev/nvidia0").exists() {
        return true;
    }

    if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let vendor_path = entry.path().join("device/vendor");
            if let Ok(vendor) = std::fs::read_to_string(&vendor_path) {
                if vendor.trim().eq_ignore_ascii_case("0x10de") {
                    return true;
                }
            }
        }
    }

    false
}

/// Applies environment-variable workarounds for known WebKit2GTK + NVIDIA
/// driver incompatibilities on Linux.  Returns `true` when an NVIDIA GPU is
/// detected and at least one workaround was applied.
///
/// **Must be called before `tauri::Builder::build()`** — that is the point at
/// which Tauri spawns the WebKit/WPE subprocess that inherits the environment.
///
/// ### Why this is needed
///
/// NVIDIA's EGL implementation (driver ≥ 560) triggers a null-pointer
/// dereference inside `libwebkit2gtk` when the driver uses explicit GPU-fence
/// synchronisation.  Setting `__NV_DISABLE_EXPLICIT_SYNC=1` keeps DMA-BUF
/// compositing active while suppressing the broken sync path, eliminating
/// both the segfault and the rendering lag.
///
/// The previous workaround (`WEBKIT_DISABLE_DMABUF_RENDERER=1`) disabled
/// DMA-BUF entirely.  In WebKit 2.44+ this caused a significant performance
/// regression that was the root cause of the severe lag reported in
/// <https://github.com/modrinth/code/issues/3057>.
///
/// Variables already present in the environment are left untouched, so users
/// who have set them explicitly retain full control.
#[cfg(target_os = "linux")]
fn apply_nvidia_webkit_workarounds() -> bool {
    if !detect_nvidia_gpu() {
        return false;
    }

    if env::var("__NV_DISABLE_EXPLICIT_SYNC").is_err() {
        env::set_var("__NV_DISABLE_EXPLICIT_SYNC", "1");
    }

    true
}

// Should be called in launcher initialization
#[tracing::instrument(skip_all)]
#[tauri::command]
async fn initialize_state(app: tauri::AppHandle) -> api::Result<()> {
    tracing::info!("Initializing app event state...");
    theseus::EventState::init(app.clone()).await?;

    tracing::info!("Initializing app state...");
    State::init(app.config().identifier.clone()).await?;

    let state = State::get().await?;
    app.asset_protocol_scope()
        .allow_directory(state.directories.caches_dir(), true)?;
    app.asset_protocol_scope()
        .allow_directory(state.directories.caches_dir().join("icons"), true)?;
    app.fs_scope()
        .allow_directory(state.directories.profiles_dir(), true)?;

    Ok(())
}

// Should be call once Vue has mounted the app
#[tracing::instrument(skip_all)]
#[tauri::command]
fn show_window(app: tauri::AppHandle) {
    let win = app.get_window("main").unwrap();
    if let Err(e) = win.show() {
        DialogBuilder::message()
            .set_level(MessageLevel::Error)
            .set_title("Initialization error")
            .set_text(format!(
                "Cannot display application window due to an error:\n{e}"
            ))
            .alert()
            .show()
            .unwrap();
        panic!("cannot display application window")
    } else {
        let _ = win.set_focus();
    }
}

#[tauri::command]
fn is_dev() -> bool {
    cfg!(debug_assertions)
}

#[tauri::command]
fn are_updates_enabled() -> bool {
    cfg!(feature = "updater")
        && env::var("MODRINTH_EXTERNAL_UPDATE_PROVIDER").is_err()
}

#[cfg(feature = "updater")]
pub use updater_impl::*;

#[cfg(not(feature = "updater"))]
pub use updater_impl_noop::*;

// Toggles decorations
#[tauri::command]
async fn toggle_decorations(b: bool, window: tauri::Window) -> api::Result<()> {
    window.set_decorations(b).map_err(|e| {
        theseus::Error::from(theseus::ErrorKind::OtherError(format!(
            "Failed to toggle decorations: {e}"
        )))
    })?;
    Ok(())
}

#[tauri::command]
fn restart_app(app: tauri::AppHandle) {
    app.restart();
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

    // Apply Linux + NVIDIA workarounds before Tauri initializes WebKit.
    // The WebKit subprocess inherits the process environment at builder.build()
    // time, so env vars must be set here — before tauri::generate_context!().
    #[cfg(target_os = "linux")]
    let nvidia_workarounds_applied = apply_nvidia_webkit_workarounds();

    let tauri_context = tauri::generate_context!();
    let _log_guard = theseus::start_logger(&tauri_context.config().identifier);

    #[cfg(target_os = "linux")]
    if nvidia_workarounds_applied {
        tracing::info!(
            "NVIDIA GPU detected — applied WebKit2GTK workaround \
             (__NV_DISABLE_EXPLICIT_SYNC=1) to prevent EGL segfault/lag. \
             See https://github.com/modrinth/code/issues/3057"
        );
    }

    tracing::info!("Initialized tracing subscriber. Loading Modrinth App!");

    let mut builder = tauri::Builder::default();

    #[cfg(feature = "updater")]
    {
        use tauri_plugin_http::reqwest::header::{HeaderValue, USER_AGENT};
        use theseus::launcher_user_agent;

        builder = builder.plugin(
            tauri_plugin_updater::Builder::new()
                .header(
                    USER_AGENT,
                    HeaderValue::from_str(&launcher_user_agent()).unwrap(),
                )
                .unwrap()
                .build(),
        );
    }

    builder = builder
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if let Some(payload) = args.get(1) {
                tracing::info!("Handling deep link from arg {payload}");
                let payload = payload.clone();
                tauri::async_runtime::spawn(api::utils::handle_command(
                    payload,
                ));
            }
            if let Some(win) = app.get_window("main") {
                let _ = win.set_focus();
            }
        }))
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_filename("app-window-state.json")
                // Use *only* POSITION and SIZE state flags, because saving VISIBLE causes the `visible: false` to not take effect
                .with_state_flags(
                    tauri_plugin_window_state::StateFlags::POSITION
                        | tauri_plugin_window_state::StateFlags::SIZE
                        | tauri_plugin_window_state::StateFlags::MAXIMIZED,
                )
                .build(),
        )
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                let payload = macos::deep_link::get_or_init_payload(app);
                let mtx_copy = payload.payload;
                app.listen("deep-link://new-url", move |url| {
                    let mtx_copy_copy = mtx_copy.clone();
                    let request = url.payload().to_owned();
                    let actual_request =
                        serde_json::from_str::<Vec<String>>(&request)
                            .ok()
                            .map(|mut x| x.remove(0))
                            .unwrap_or(request);
                    tauri::async_runtime::spawn(async move {
                        tracing::info!("Handling deep link {actual_request}");
                        let mut payload = mtx_copy_copy.lock().await;
                        if payload.is_none() {
                            *payload = Some(actual_request.clone());
                        }
                        let _ =
                            api::utils::handle_command(actual_request).await;
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
            });

            #[cfg(not(target_os = "linux"))]
            if let Some(window) = app.get_window("main")
                && let Err(e) = window.set_shadow(true)
            {
                tracing::warn!("Failed to set window shadow: {e}");
            }

            Ok(())
        });

    builder = builder
        .plugin(api::auth::init())
        .plugin(api::mr_auth::init())
        .plugin(api::import::init())
        .plugin(api::logs::init())
        .plugin(api::jre::init())
        .plugin(api::metadata::init())
        .plugin(api::minecraft_skins::init())
        .plugin(api::pack::init())
        .plugin(api::process::init())
        .plugin(api::profile::init())
        .plugin(api::profile_create::init())
        .plugin(api::settings::init())
        .plugin(api::tags::init())
        .plugin(api::utils::init())
        .plugin(api::cache::init())
        .plugin(api::files::init())
        .plugin(api::ads::init())
        .plugin(api::friends::init())
        .plugin(api::worlds::init())
        .manage(PendingUpdateData::default())
        .invoke_handler(tauri::generate_handler![
            initialize_state,
            is_dev,
            are_updates_enabled,
            get_update_size,
            enqueue_update_for_installation,
            remove_enqueued_update,
            toggle_decorations,
            show_window,
            restart_app,
        ]);

    tracing::info!("Initializing app...");

    let app = builder.build(tauri_context);
    match app {
        Ok(app) => {
            app.run(|app, event| {
                #[cfg(not(any(feature = "updater", target_os = "macos")))]
                drop((app, event));

                #[cfg(feature = "updater")]
                if matches!(event, tauri::RunEvent::Exit) {
                    let update_data = app.state::<PendingUpdateData>().inner();
                    if let Some((update, data)) = &*update_data.0.lock().unwrap() {
                        fn set_changelog_toast(version: Option<String>) {
                            let toast_result: theseus::Result<()> =
                                tauri::async_runtime::block_on(async move {
                                    let mut settings = settings::get().await?;
                                    settings.pending_update_toast_for_version =
                                        version;
                                    settings::set(settings).await?;
                                    Ok(())
                                });
                            if let Err(e) = toast_result {
                                tracing::warn!(
                                    "Failed to set pending_update_toast: {e}"
                                )
                            }
                        }

                        set_changelog_toast(Some(update.version.clone()));
                        if let Err(e) = update.install(data) {
                            tracing::error!("Error while updating: {e}");
                            set_changelog_toast(None);
                            DialogBuilder::message()
                                .set_level(MessageLevel::Error)
                                .set_title("Update error")
                                .set_text(format!(
                                    "Failed to install update due to an error:\n{e}"
                                ))
                                .alert()
                                .show()
                                .unwrap();
                        }
                        app.restart();
                    }
                }

                #[cfg(target_os = "macos")]
                if let tauri::RunEvent::Opened { urls } = event {
                    tracing::info!("Handling webview open {urls:?}");
                    let file = urls
                        .into_iter()
                        .find_map(|url| url.to_file_path().ok());
                    if let Some(file) = file {
                        let payload =
                            macos::deep_link::get_or_init_payload(app);
                        let mtx_copy = payload.payload;
                        let request = file.to_string_lossy().to_string();
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
        }
        Err(e) => {
            tracing::error!("Error while running tauri application: {:?}", e);

            #[cfg(target_os = "windows")]
            {
                // tauri doesn't expose runtime errors, so matching a string representation seems like the only solution
                if format!("{e:?}").contains(
                    "Runtime(CreateWebview(WebView2Error(WindowsError",
                ) {
                    DialogBuilder::message()
                        .set_level(MessageLevel::Error)
                        .set_title("Initialization error")
                        .set_text("Your Microsoft Edge WebView2 installation is corrupt.\n\nMicrosoft Edge WebView2 is required to run Modrinth App.\n\nLearn how to repair it at https://support.modrinth.com/en/articles/8797765-corrupted-microsoft-edge-webview2-installation")
                        .alert()
                        .show()
                        .unwrap();
                    panic!("webview2 initialization failed")
                }
            }

            DialogBuilder::message()
                .set_level(MessageLevel::Error)
                .set_title("Initialization error")
                .set_text(format!(
                    "Cannot initialize application due to an error:\n{e:?}"
                ))
                .alert()
                .show()
                .unwrap();
            panic!("{1}: {:?}", e, "error while running tauri application")
        }
    }
}
  
