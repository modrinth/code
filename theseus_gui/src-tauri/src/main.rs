#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use theseus::prelude::*;

use tauri::{Manager, WindowEvent};
use tracing_error::ErrorLayer;
use tracing_subscriber::EnvFilter;

mod api;
mod error;

// Should be called in launcher initialization
#[tauri::command]
async fn initialize_state(app: tauri::AppHandle) -> api::Result<()> {
    theseus::EventState::init(app).await?;
    State::get().await?;
    State::update();
    Ok(())
}

use tracing_subscriber::prelude::*;

#[derive(Clone, serde::Serialize)]
struct Payload {
    args: Vec<String>,
    cwd: String,
}

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
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("theseus=info"));

    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .with(ErrorLayer::default());

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            app.emit_all("single-instance", Payload { args: argv, cwd })
                .unwrap();
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build());

    #[cfg(target_os = "macos")]
    {
        builder = builder
            .setup(|app| {
                use api::window_ext::WindowExt;
                let win = app.get_window("main").unwrap();
                win.set_transparent_titlebar(true);
                win.position_traffic_lights(0.0, 0.0);
                Ok(())
            })
            .on_window_event(|e| {
                use api::window_ext::WindowExt;
                if let WindowEvent::Resized(..) = e.event() {
                    let win = e.window();
                    win.position_traffic_lights(0., 0.);
                }
            })
    }
    let builder = builder
        .plugin(api::auth::init())
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
        .invoke_handler(tauri::generate_handler![initialize_state,]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
