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

use tracing_appender::non_blocking::WorkerGuard;

// Handling for the live development logging
// This will log to the console, and will not log to a file
#[cfg(debug_assertions)]
pub fn start_logger() -> Option<WorkerGuard> {
    use tracing_subscriber::prelude::*;

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            tracing_subscriber::EnvFilter::new("theseus=info,theseus_gui=info")
        });
    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .with(tracing_error::ErrorLayer::default());
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");
    None
}

// Handling for the live production logging
// This will log to a file in the logs directory, and will not show any logs in the console
#[cfg(not(debug_assertions))]
pub fn start_logger() -> Option<WorkerGuard> {
    use crate::prelude::DirectoryInfo;
    use tracing_appender::rolling::{RollingFileAppender, Rotation};
    use tracing_subscriber::fmt::time::ChronoLocal;
    use tracing_subscriber::prelude::*;

    // Initialize and get logs directory path
    let logs_dir = if let Some(d) = DirectoryInfo::launcher_logs_dir() {
        d
    } else {
        eprintln!("Could not start logger");
        return None;
    };

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("theseus=info"));

    let file_appender =
        RollingFileAppender::new(Rotation::DAILY, logs_dir, "theseus.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let subscriber = tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false) // disable ANSI escape codes
                .with_timer(ChronoLocal::rfc_3339()),
        )
        .with(filter)
        .with(tracing_error::ErrorLayer::default());

    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subscriber failed");

    Some(guard)
}
