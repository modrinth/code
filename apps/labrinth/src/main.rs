use actix_web::middleware::from_fn;
use actix_web::{App, HttpServer};
use actix_web_prom::PrometheusMetricsBuilder;
use clap::Parser;
use labrinth::background_task::BackgroundTask;
use labrinth::database::redis::RedisPool;
use labrinth::file_hosting::S3Host;
use labrinth::search;
use labrinth::util::ratelimit::rate_limit_middleware;
use labrinth::{check_env_vars, clickhouse, database, file_hosting, queue};
use std::sync::Arc;
use tracing::{error, info};
use tracing_actix_web::TracingLogger;

#[cfg(target_os = "linux")]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[allow(non_upper_case_globals)]
#[unsafe(export_name = "malloc_conf")]
pub static malloc_conf: &[u8] =
    b"prof:true,prof_active:true,lg_prof_sample:19\0";

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

    /// Don't automatically run migrations. This means the migrations should be run via --run-background-task.
    #[arg(long)]
    no_migrations: bool,

    /// Run a single background task and then exit. Perfect for cron jobs.
    #[arg(long, value_enum, id = "task")]
    run_background_task: Option<BackgroundTask>,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    dotenvy::dotenv().ok();
    console_subscriber::init();

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
        unsafe {
            std::env::set_var("RUST_BACKTRACE", "1");
        }
    }

    if args.run_background_task.is_none() {
        info!(
            "Starting Labrinth on {}",
            dotenvy::var("BIND_ADDR").unwrap()
        );

        if !args.no_migrations {
            database::check_for_migrations()
                .await
                .expect("An error occurred while running migrations.");
        }
    }

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
        .exclude_regex(r"^/api/v1/.*$")
        .exclude_regex(r"^/maven/.*$")
        .exclude("/_internal/launcher_socket")
        .mask_unmatched_patterns("UNKNOWN")
        .build()
        .expect("Failed to create prometheus metrics middleware");

    database::register_and_set_metrics(&pool, &prometheus.registry)
        .await
        .expect("Failed to register database metrics");
    redis_pool
        .register_and_set_metrics(&prometheus.registry)
        .await
        .expect("Failed to register redis metrics");

    #[cfg(target_os = "linux")]
    labrinth::routes::debug::jemalloc_memory_stats(&prometheus.registry)
        .expect("Failed to register jemalloc metrics");

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
            .wrap(TracingLogger::default())
            .wrap(prometheus.clone())
            .wrap(from_fn(rate_limit_middleware))
            .wrap(actix_web::middleware::Compress::default())
            .wrap(sentry_actix::Sentry::new())
            .configure(|cfg| labrinth::app_config(cfg, labrinth_config.clone()))
    })
    .bind(dotenvy::var("BIND_ADDR").unwrap())?
    .run()
    .await
}
