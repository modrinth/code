use std::time::Duration;

use anyhow::{Context, Result};
use toasty::migration::MigrationSet;
use tracing::{info, info_span, level_filters::LevelFilter};
use tracing_anyhow::FutureContext;
use tracing_subscriber::{
    EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt,
};

mod model;
mod task;
mod upstream;
mod util;

#[derive(Debug, clap::Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    Download,
}

pub async fn main() -> Result<()> {
    static MIGRATIONS: MigrationSet = toasty::embed_migrations!();

    let cli = <Cli as clap::Parser>::parse();
    tracing_subscriber::registry()
        .with(tracing_anyhow::ErrorLayer::default())
        .with(
            tracing_subscriber::fmt::layer()
                .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(
                    "%H:%M".to_owned(),
                ))
                .with_target(false)
                .with_writer(std::io::stderr)
                .with_filter(
                    EnvFilter::builder()
                        .with_default_directive(LevelFilter::INFO.into())
                        .from_env_lossy(),
                ),
        )
        .init();

    let mut db = connect_to_db().await?;
    let report = MIGRATIONS
        .apply(&db)
        .context(info_span!("applying migrations"))
        .await?;
    info!("applied {} migrations", report.applied());

    let state = AppState {
        http: reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("building HTTP client")?,
        db: connect_to_db().await?,
    };

    match cli.command {
        Command::Download => task::download_from_upstreams(&state).await,
    }
}

#[derive(Debug)]
struct AppState {
    http: reqwest::Client,
    db: toasty::Db,
}

pub async fn connect_to_db() -> Result<toasty::Db> {
    let db_url =
        "postgresql://launchermeta:launchermeta@localhost:5433/launchermeta"
            .to_string();
    toasty::Db::builder()
        .models(toasty::models!(crate::*))
        .connect(&db_url)
        .context(info_span!("connecting to database", %db_url))
        .await
}
