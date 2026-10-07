use std::{sync::Arc, time::Duration};

use anyhow::{Context, Result};
use figment::Figment;
use toasty::migration::MigrationSet;
use tracing::{info, info_span, level_filters::LevelFilter};
use tracing_anyhow::FutureContext;
use tracing_subscriber::{
    EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt,
};

use crate::{
    config::Config,
    store::{BlobCas, BlobStore, MavenStore, StoreVisibility},
    util::fastly,
};

mod config;
mod export;
mod model;
mod store;
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
    /// Download manifests and sources from upstreamms
    Download {
        /// Download Minecraft game version info from Mojang?
        #[arg(long)]
        mojang: bool,
        /// Download Fabric loader info?
        #[arg(long)]
        fabric: bool,
        /// Download Forge loader info?
        #[arg(long)]
        forge: bool,
        /// Download NeoForge loader info?
        #[arg(long)]
        neoforge: bool,
        /// Download Quilt loader info?
        #[arg(long)]
        quilt: bool,
    },
    /// Extract downloaded installer JARs for their manifests and libraries
    ExtractInstallers,
    /// Take processed information from the database, produce JSON manifests
    /// from them, and upload them to the public file store.
    Export,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum Upstream {
    Fabric,
    Forge,
    Mojang,
    NeoForge,
    Quilt,
}

pub async fn main() -> Result<()> {
    static MIGRATIONS: MigrationSet = toasty::embed_migrations!();

    dotenvy::dotenv().ok();
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

    let config = create_config()?;

    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .context("building HTTP client")?;

    let db = connect_to_db(&config).await?;
    let report = MIGRATIONS
        .apply(&db)
        .context(info_span!("applying migrations"))
        .await?;
    info!("applied {} migrations", report.applied());

    let fastly = if let Some(config) = config.fastly {
        fastly::Client::new(http.clone(), config.base_url, config.key)
            .map(|client| Some(Arc::new(client)))
            .context("creating Fastly client")?
    } else {
        None
    };

    let public_blobs = BlobStore::new(
        config.public_s3,
        fastly.clone(),
        StoreVisibility::Public,
    )
    .context(info_span!("creating public blob store"))
    .await?;
    // private store is never behind a CDN, so we don't need to purge with Fastly
    let private_blobs =
        BlobStore::new(config.private_s3, None, StoreVisibility::Private)
            .context(info_span!("creating private blob store"))
            .await?;
    let cas = BlobCas::new(private_blobs.clone());
    let maven = MavenStore::new(public_blobs.clone());

    let mut app = AppState {
        http,
        db,
        concurrency: config.concurrency,
        cas,
        public_blobs,
        maven,
    };

    match cli.command {
        Command::Download {
            mojang,
            fabric,
            forge,
            neoforge,
            quilt,
        } => {
            let all_upstreams =
                !mojang && !fabric && !forge && !neoforge && !quilt;
            let upstreams = task::Upstreams {
                mojang: all_upstreams || mojang,
                fabric: all_upstreams || fabric,
                forge: all_upstreams || forge,
                neoforge: all_upstreams || neoforge,
                quilt: all_upstreams || quilt,
            };
            task::download_from_upstreams(&app, upstreams).await
        }
        Command::ExtractInstallers => task::extract_installers(&mut app).await,
        Command::Export => task::export(&mut app).await,
    }
}

#[derive(Debug)]
struct AppState {
    http: reqwest::Client,
    db: toasty::Db,
    concurrency: config::Concurrency,
    cas: BlobCas,
    public_blobs: BlobStore,
    maven: MavenStore,
}

pub fn create_config() -> Result<Config> {
    Figment::new()
        .merge(figment::providers::Env::prefixed("MR_").split("__"))
        .extract::<Config>()
        .context("parsing config")
}

pub async fn connect_to_db(config: &Config) -> Result<toasty::Db> {
    toasty::Db::builder()
        .models(toasty::models!(crate::*))
        .connect(config.database_url.as_str())
        .context(info_span!("connecting to database"))
        .await
}
