use actix_web::{App, HttpServer};
use actix_web_prom::PrometheusMetricsBuilder;
use clap::Parser;
use env_logger::Env;
use labrinth::background_task::BackgroundTask;
use labrinth::database::redis::RedisPool;
use labrinth::file_hosting::S3Host;
use labrinth::routes::internal::statuses::close_socket;
use labrinth::search;
use labrinth::util::ratelimit::RateLimit;
use labrinth::{check_env_vars, clickhouse, database, file_hosting, queue};
use log::{error, info};
use std::sync::Arc;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[derive(Clone)]
pub struct Pepper {
    pub pepper: String,
}

#[derive(Parser)]
#[command(version)]
struct Args {
    /// Don't run regularly scheduled background tasks. This means the tasks should be run
    /// manually with --run-background-task.
    #[arg(long)]
    no_background_tasks: bool,

    /// Run a single background task and then exit. Perfect for cron jobs.
    #[arg(long, value_enum, id = "task")]
    run_background_task: Option<BackgroundTask>,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    dotenvy::dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .init();

    if check_env_vars() {
        error!("Some environment variables are missing!");
    }

    // DSN is from SENTRY_DSN env variable.
    // Has no effect if not set.
    let sentry = sentry::init(sentry::ClientOptions {
        release: sentry::release_name!(),
        traces_sample_rate: 0.1,
        ..Default::default()
    });
    if sentry.is_enabled() {
        info!("Enabled Sentry integration");
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    if args.run_background_task.is_none() {
        info!(
            "Starting Labrinth on {}",
            dotenvy::var("BIND_ADDR").unwrap()
        );
    }

    database::check_for_migrations()
        .await
        .expect("An error occurred while running migrations.");

    // Database Connector
    let pool = database::connect()
        .await
        .expect("Database connection failed");

    // Redis connector
    let redis_pool = RedisPool::new(None);

    let storage_backend =
        dotenvy::var("STORAGE_BACKEND").unwrap_or_else(|_| "local".to_string());

    let file_host: Arc<dyn file_hosting::FileHost + Send + Sync> =
        match storage_backend.as_str() {
            "backblaze" => Arc::new(
                file_hosting::BackblazeHost::new(
                    &dotenvy::var("BACKBLAZE_KEY_ID").unwrap(),
                    &dotenvy::var("BACKBLAZE_KEY").unwrap(),
                    &dotenvy::var("BACKBLAZE_BUCKET_ID").unwrap(),
                )
                .await,
            ),
            "s3" => Arc::new(
                S3Host::new(
                    &dotenvy::var("S3_BUCKET_NAME").unwrap(),
                    &dotenvy::var("S3_REGION").unwrap(),
                    &dotenvy::var("S3_URL").unwrap(),
                    &dotenvy::var("S3_ACCESS_TOKEN").unwrap(),
                    &dotenvy::var("S3_SECRET").unwrap(),
                )
                .unwrap(),
            ),
            "local" => Arc::new(file_hosting::MockHost::new()),
            _ => panic!("Invalid storage backend specified. Aborting startup!"),
        };

    info!("Initializing clickhouse connection");
    let mut clickhouse = clickhouse::init_client().await.unwrap();

    let search_config = search::SearchConfig::new(None);

    let stripe_client =
        stripe::Client::new(dotenvy::var("STRIPE_API_KEY").unwrap());

    if let Some(task) = args.run_background_task {
        info!("Running task {task:?} and exiting");
        task.run(pool, redis_pool, search_config, clickhouse, stripe_client)
            .await;
        return Ok(());
    }

    let maxmind_reader =
        Arc::new(queue::maxmind::MaxMindIndexer::new().await.unwrap());

    let prometheus = PrometheusMetricsBuilder::new("labrinth")
        .endpoint("/metrics")
        .exclude("/_internal/launcher_socket")
        .build()
        .expect("Failed to create prometheus metrics middleware");

    let labrinth_config = labrinth::app_setup(
        pool.clone(),
        redis_pool.clone(),
        search_config.clone(),
        &mut clickhouse,
        file_host.clone(),
        maxmind_reader.clone(),
        stripe_client,
        !args.no_background_tasks,
    );

    info!("Starting Actix HTTP server!");

    // Init App
    HttpServer::new(move || {
        App::new()
            .wrap(prometheus.clone())
            .wrap(RateLimit(Arc::clone(&labrinth_config.rate_limiter)))
            .wrap(actix_web::middleware::Compress::default())
            .wrap(sentry_actix::Sentry::new())
            .configure(|cfg| labrinth::app_config(cfg, labrinth_config.clone()))
    })
    .bind(dotenvy::var("BIND_ADDR").unwrap())?
    .run()
    .await
}
