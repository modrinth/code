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

    The default is theseus=show, meaning only logs from theseus will be displayed, and at the info or higher level.

*/

// Handling for the live development logging
// This will log to the console, and will not log to a file
#[cfg(debug_assertions)]
pub fn start_logger(_app_identifier: &str) -> Option<()> {
    use tracing_subscriber::prelude::*;

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            tracing_subscriber::EnvFilter::new(
                "theseus=info,theseus_gui=info,webview=trace",
            )
        });
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .with(tracing_error::ErrorLayer::default())
        .init();
    Some(())
}

#[cfg(not(debug_assertions))]
const SESSION_LOG_RETENTION: std::time::Duration =
    std::time::Duration::from_secs(14 * 24 * 60 * 60);

/// Deletes launcher session logs last written more than [`SESSION_LOG_RETENTION`] ago.
#[cfg(not(debug_assertions))]
fn prune_session_logs(logs_dir: &std::path::Path) {
    let Some(cutoff) =
        std::time::SystemTime::now().checked_sub(SESSION_LOG_RETENTION)
    else {
        return;
    };
    let entries = match std::fs::read_dir(logs_dir) {
        Ok(entries) => entries,
        Err(err) => {
            eprintln!("Could not read logs directory for pruning: {err}");
            return;
        }
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with("session_") || !name.ends_with(".log") {
            continue;
        }
        let is_expired = entry.metadata().is_ok_and(|metadata| {
            metadata.is_file()
                && metadata.modified().is_ok_and(|modified| modified < cutoff)
        });
        if is_expired && let Err(err) = std::fs::remove_file(entry.path()) {
            eprintln!("Could not remove old log file {name}: {err}");
        }
    }
}

// Handling for the live production logging
// This will log to a file in the logs directory, and will not show any logs in the console
#[cfg(not(debug_assertions))]
pub fn start_logger(app_identifier: &str) -> Option<()> {
    use crate::prelude::DirectoryInfo;
    use chrono::Local;
    use std::fs::OpenOptions;
    use tracing_subscriber::fmt::time::ChronoLocal;
    use tracing_subscriber::prelude::*;

    // Initialize and get logs directory path
    let logs_dir = if let Some(d) =
        DirectoryInfo::launcher_logs_dir_path(app_identifier)
    {
        d
    } else {
        eprintln!("Could not start logger");
        return None;
    };

    let log_file_name =
        format!("session_{}.log", Local::now().format("%Y%m%d_%H%M%S"));
    let log_file_path = logs_dir.join(log_file_name);

    if let Err(err) = std::fs::create_dir_all(&logs_dir) {
        eprintln!("Could not create logs directory: {err}");
    }
    prune_session_logs(&logs_dir);

    let file = match OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_file_path)
    {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Could not start open log file: {e}");
            return None;
        }
    };

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            tracing_subscriber::EnvFilter::new("theseus=info,webview=trace")
        });

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(file)
                .with_ansi(false) // disable ANSI escape codes
                .with_timer(ChronoLocal::rfc_3339()),
        )
        .with(filter)
        .with(tracing_error::ErrorLayer::default())
        .init();

    Some(())
}
