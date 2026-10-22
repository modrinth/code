use std::{sync::Arc, time::Duration};

use anyhow::{Context, Result, ensure};
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
    upstream::Upstreams,
    util::{TaskContext, fastly},
};

mod config;
mod export;
mod import;
mod store;
mod util;

pub use minecraft_meta_model::{model, upstream};

#[derive(Debug, clap::Parser)]
struct Cli {
    /// Process Minecraft game version info from Mojang?
    #[arg(long)]
    mojang: bool,
    /// Process Fabric loader info?
    #[arg(long)]
    fabric: bool,
    /// Process Forge loader info?
    #[arg(long)]
    forge: bool,
    /// Process NeoForge loader info?
    #[arg(long)]
    neoforge: bool,
    /// Process Quilt loader info?
    #[arg(long)]
    quilt: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    /// Download manifests and sources from upstreams, and extract their
    /// information into the local database
    Import,
    /// Take processed information from the database, produce JSON manifests
    /// from them, and upload them to the public file store
    Export,
}

pub fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_anyhow::ErrorLayer::default())
        .with(sentry::integrations::tracing::layer())
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
    let mut config = create_config()?;

    let _sentry = if let Some(sentry) = config.sentry.take() {
        let sentry = sentry::init(sentry::ClientOptions {
            dsn: Some(sentry.dsn.parse().context("invalid Sentry DSN")?),
            release: sentry::release_name!(),
            environment: Some(sentry.environment.into()),
            sample_rate: sentry.sample_rate,
            traces_sample_rate: sentry.traces_sample_rate,
            ..Default::default()
        });
        ensure!(
            sentry.is_enabled(),
            "config defines Sentry integration but is not enabled"
        );
        info!("enabled Sentry integration");
        unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
        Some(sentry)
    } else {
        None
    };

    tokio::runtime::Runtime::new()
        .context("creating async runtime")?
        .block_on(
            async {
                let result = main_(config).await;
                if let Err(error) = &result {
                    sentry::Hub::current()
                        .capture_error::<dyn std::error::Error>(error.as_ref());
                }
                result
            }
            .transaction_context(|| info_span!("meta-generator")),
        )
}

async fn main_(config: Config) -> Result<()> {
    static MIGRATIONS: MigrationSet = toasty::embed_migrations!();

    let cli = <Cli as clap::Parser>::parse();
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
        fastly::Client::builder()
            .http(http.clone())
            .base(config.base_url)
            .fastly_key(config.key)
            .service_id(config.service_id)
            .surrogate_key(config.surrogate_key)
            .build()
            .map(|client| Some(Arc::new(client)))
            .context("creating Fastly client")?
    } else {
        None
    };

    let public_blobs =
        BlobStore::new(config.public_s3, StoreVisibility::Public)
            .context(info_span!("creating public blob store"))
            .await?;
    // private store is never behind a CDN, so we don't need to purge with Fastly
    let private_blobs =
        BlobStore::new(config.private_s3, StoreVisibility::Private)
            .context(info_span!("creating private blob store"))
            .await?;
    let cas = BlobCas::new(private_blobs.clone());
    let maven = MavenStore::new(public_blobs.clone());

    let app = AppState {
        http,
        db,
        concurrency: config.concurrency,
        cas,
        public_blobs,
        maven,
        fastly,
    };

    let upstreams = {
        let all_upstreams = !cli.mojang
            && !cli.fabric
            && !cli.forge
            && !cli.neoforge
            && !cli.quilt;
        Upstreams {
            mojang: all_upstreams || cli.mojang,
            fabric: all_upstreams || cli.fabric,
            forge: all_upstreams || cli.forge,
            neoforge: all_upstreams || cli.neoforge,
            quilt: all_upstreams || cli.quilt,
        }
    };

    match cli.command {
        Command::Import => {
            import::run(app, upstreams)
                .context(info_span!("import"))
                .await
        }
        Command::Export => {
            export::run(app, upstreams)
                .context(info_span!("export"))
                .await
        }
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
    fastly: Option<Arc<fastly::Client>>,
}

pub fn create_config() -> Result<Config> {
    Figment::new()
        .merge(figment::providers::Env::prefixed("MR_").split("__"))
        .extract::<Config>()
        .context("parsing config")
}

pub async fn connect_to_db(config: &Config) -> Result<toasty::Db> {
    toasty::Db::builder()
        .models(toasty::models!(minecraft_meta_model::*))
        .connect(config.database_url.as_str())
        .context(info_span!("connecting to database"))
        .await
}
