#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use theseus::prelude::*;

use tauri::Manager;

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

#[tauri::command]
fn is_dev() -> bool {
    cfg!(debug_assertions)
}

use tracing_subscriber::prelude::*;

#[derive(Clone, serde::Serialize)]
struct Payload {
    args: Vec<String>,
    cwd: String,
}

fn main() {
    //let client = sentry::init("https://19a14416dafc4b4a858fa1a38db3b704@o485889.ingest.sentry.io/4505349067374592");

    //let _guard = sentry_rust_minidump::init(&client);
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

    #[cfg(not(target_os = "macos"))]
    {
        builder = builder.setup(|app| {
            let win = app.get_window("main").unwrap();
            win.set_decorations(false).unwrap();
            Ok(())
        })
    }

    #[cfg(not(target_os = "linux"))]
    {
        use window_shadows::set_shadow;

        builder = builder.setup(|app| {
            let win = app.get_window("main").unwrap();
            set_shadow(&win, true).unwrap();
            Ok(())
        });
    }

    #[cfg(target_os = "macos")]
    {
        use tauri::WindowEvent;

        builder = builder
            .setup(|app| {
                use api::window_ext::WindowExt;
                let win = app.get_window("main").unwrap();
                win.set_transparent_titlebar(true);
                win.position_traffic_lights(9.0, 16.0);
                Ok(())
            })
            .on_window_event(|e| {
                use api::window_ext::WindowExt;
                if let WindowEvent::Resized(..) = e.event() {
                    let win = e.window();
                    win.position_traffic_lights(9.0, 16.0);
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
        .invoke_handler(tauri::generate_handler![initialize_state, is_dev]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
