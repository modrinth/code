use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use clap::Parser;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod application;
mod cli;
mod config;
mod domain;
mod infrastructure;
mod ports;
mod presentation;

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;

    cli::execute(cli::Cli::parse()).await
}

pub(crate) fn init_tracing() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::from_default_env()
                .add_directive("copal=info".parse().unwrap()),
        )
        .init();
}

pub(crate) async fn run_server() -> color_eyre::eyre::Result<()> {
    let config = config::Config::from_env()?;
    tokio::fs::create_dir_all(&config.data_dir).await?;

    let db_path = config.data_dir.join("data.db");
    let pool = infrastructure::db::connect(&db_path).await?;

    infrastructure::db::migrate(&pool).await?;

    let port = config.port;
    let bind_host = config.bind_host.clone();
    let state = application::state::AppState::new(config, pool).await?;

    #[cfg(unix)]
    tokio::spawn(cli::admin::serve(Arc::clone(&state)));

    tokio::spawn(application::instance_service::restore_instances(
        Arc::clone(&state),
    ));
    tokio::spawn(application::server_source_service::resume(Arc::clone(
        &state,
    )));
    tokio::spawn(gc_ws_tickets(Arc::clone(&state)));
    tokio::spawn(application::pairing_service::run(Arc::clone(&state)));
    let router = presentation::router::create_router(state);
    let host: IpAddr = bind_host.parse()?;
    let addr = SocketAddr::new(host, port);

    info!("Copal listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}

/// SEC-02: Drain expired WebSocket tickets every 5 minutes to prevent unbounded growth.
async fn gc_ws_tickets(state: Arc<application::state::AppState>) {
    use std::time::Instant;
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
        state
            .ws_tickets
            .retain(|_, t| t.expires_at > Instant::now());
    }
}
