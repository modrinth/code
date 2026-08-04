use std::path::PathBuf;

use theseus::AppEvent;
use ts_rs::{Config, TS};

#[path = "../../app_event_bindings.rs"]
#[allow(dead_code)]
mod app_event_bindings;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest_dir.join("../..");
    let output = workspace.join("apps/app-frontend/src/generated/app-events");
    let config = Config::default()
        .with_out_dir(output)
        .with_large_int("number");

    app_event_bindings::clear_generated_types(config.out_dir())?;
    AppEvent::export_all(&config)?;
    app_event_bindings::write_manifest(&manifest_dir, config.out_dir())?;
    Ok(())
}
