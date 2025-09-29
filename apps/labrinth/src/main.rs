use actix_web::dev::Service;
use actix_web::middleware::from_fn;
use actix_web::{App, HttpServer};
use actix_web_prom::PrometheusMetricsBuilder;
use clap::Parser;
use futures::FutureExt;
use labrinth::app_config;
use labrinth::background_task::BackgroundTask;
use labrinth::database::redis::RedisPool;
use labrinth::file_hosting::{S3BucketConfig, S3Host};
use labrinth::queue::email::EmailQueue;
use labrinth::search;
use labrinth::util::anrok;
use labrinth::util::env::parse_var;
use labrinth::util::ratelimit::rate_limit_middleware;
use labrinth::{check_env_vars, clickhouse, database, file_hosting, queue};
use std::ffi::CStr;
use std::io::{self, stdout};
use std::str::FromStr;
use std::sync::Arc;
use tracing::level_filters::LevelFilter;
use tracing::{Level, error, info};
use tracing_actix_web::TracingLogger;
use tracing_ecs::ECSLayerBuilder;
use tracing_subscriber::fmt::SubscriberBuilder;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

#[cfg(target_os = "linux")]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[unsafe(export_name = "malloc_conf")]
pub static MALLOC_CONF: &CStr = c"prof:true,prof_active:true,lg_prof_sample:19";

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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
enum OutputFormat {
    #[default]
    Human,
    Json,
}

impl FromStr for OutputFormat {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "human" => Ok(Self::Human),
            "json" => Ok(Self::Json),
            _ => Err(()),
        }
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    color_eyre::install().expect("failed to install `color-eyre`");
    dotenvy::dotenv().ok();
    let console_layer = console_subscriber::spawn();
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    let output_format =
        dotenvy::var("LABRINTH_FORMAT").map_or(OutputFormat::Human, |format| {
            format
                .parse::<OutputFormat>()
                .unwrap_or_else(|_| panic!("invalid output format '{format}'"))
        });

    match output_format {
        OutputFormat::Human => {
            tracing_subscriber::registry()
                .with(console_layer)
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer())
                .init();
        }
        OutputFormat::Json => {
            tracing_subscriber::registry()
                .with(console_layer)
                .with(env_filter)
                .with(ECSLayerBuilder::default().stdout())
                .init();
        }
    }

    if check_env_vars() {
        error!("Some environment variables are missing!");
        std::process::exit(1);
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
            "Starting labrinth on {}",
            dotenvy::var("BIND_ADDR").unwrap()
        );

        if !args.no_migrations {
            database::check_for_migrations()
                .await
                .expect("An error occurred while running migrations.");
        }
    }

    // Database Connector
    let (pool, ro_pool) = database::connect_all()
        .await
        .expect("Database connection failed");

    // Redis connector
    let redis_pool = RedisPool::new(None);

    let storage_backend =
        dotenvy::var("STORAGE_BACKEND").unwrap_or_else(|_| "local".to_string());

    let file_host: Arc<dyn file_hosting::FileHost + Send + Sync> =
        match storage_backend.as_str() {
            "s3" => {
                let config_from_env = |bucket_type| S3BucketConfig {
                    name: parse_var(&format!("S3_{bucket_type}_BUCKET_NAME"))
                        .unwrap(),
                    uses_path_style: parse_var(&format!(
                        "S3_{bucket_type}_USES_PATH_STYLE_BUCKET"
                    ))
                    .unwrap(),
                    region: parse_var(&format!("S3_{bucket_type}_REGION"))
                        .unwrap(),
                    url: parse_var(&format!("S3_{bucket_type}_URL")).unwrap(),
                    access_token: parse_var(&format!(
                        "S3_{bucket_type}_ACCESS_TOKEN"
                    ))
                    .unwrap(),
                    secret: parse_var(&format!("S3_{bucket_type}_SECRET"))
                        .unwrap(),
                };

                Arc::new(
                    S3Host::new(
                        config_from_env("PUBLIC"),
                        config_from_env("PRIVATE"),
                    )
                    .unwrap(),
                )
            }
            "local" => Arc::new(file_hosting::MockHost::new()),
            _ => panic!("Invalid storage backend specified. Aborting startup!"),
        };

    info!("Initializing clickhouse connection");
    let mut clickhouse = clickhouse::init_client().await.unwrap();

    let search_config = search::SearchConfig::new(None);

    let stripe_client =
        stripe::Client::new(dotenvy::var("STRIPE_API_KEY").unwrap());

    let anrok_client = anrok::Client::from_env().unwrap();
    let email_queue =
        EmailQueue::init(pool.clone(), redis_pool.clone()).unwrap();

    if let Some(task) = args.run_background_task {
        info!("Running task {task:?} and exiting");
        task.run(
            pool,
            redis_pool,
            search_config,
            clickhouse,
            stripe_client,
            anrok_client.clone(),
            email_queue,
        )
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
        ro_pool.clone(),
        redis_pool.clone(),
        search_config.clone(),
        &mut clickhouse,
        file_host.clone(),
        maxmind_reader.clone(),
        stripe_client,
        anrok_client.clone(),
        email_queue,
        !args.no_background_tasks,
    );

    info!("Starting Actix HTTP server!");

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap_fn(|req, srv| {
                srv.call(req).inspect(|result| {
                    _ = result.as_ref().inspect(|resp| {
                        resp.response().error().inspect(|err| log_error(*err));
                    });
                })
            })
            .wrap(prometheus.clone())
            .wrap(from_fn(rate_limit_middleware))
            .wrap(actix_web::middleware::Compress::default())
            .wrap(sentry_actix::Sentry::new())
            .configure(|cfg| app_config(cfg, labrinth_config.clone()))
    })
    .bind(dotenvy::var("BIND_ADDR").unwrap())?
    .run()
    .await
}

fn log_error(err: &actix_web::Error) {
    if err.as_response_error().status_code().is_client_error() {
        tracing::debug!(
            "Error encountered while processing the incoming HTTP request: {err}"
        );
    } else {
        tracing::error!(
            "Error encountered while processing the incoming HTTP request: {err}"
        );
    }
}
