use actix_web::dev::Service;
use actix_web::middleware::from_fn;
use actix_web::{App, HttpServer};
use actix_web_prom::PrometheusMetricsBuilder;
use clap::Parser;

use labrinth::background_task::BackgroundTask;
use labrinth::database::redis::RedisPool;
use labrinth::env::ENV;
use labrinth::file_hosting::{FileHostKind, S3BucketConfig, S3Host};
use labrinth::queue::email::EmailQueue;
use labrinth::search;
use labrinth::util::anrok;
use labrinth::util::gotenberg::GotenbergClient;
use labrinth::util::ratelimit::rate_limit_middleware;
use labrinth::utoipa_app_config;
use labrinth::{app_config, env};
use labrinth::{clickhouse, database, file_hosting};
use std::ffi::CStr;
use std::io;
use std::sync::Arc;
use tracing::{Instrument, info, info_span};
use tracing_actix_web::TracingLogger;
use utoipa::OpenApi;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa_actix_web::AppExt;
use utoipa_swagger_ui::SwaggerUi;

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

fn main() -> std::io::Result<()> {
    color_eyre::install().expect("failed to install `color-eyre`");
    modrinth_util::log::init().expect("failed to initialize logging");
    env::init().expect("failed to initialize environment variables");

    // Sentry must be set up before the async runtime is started
    // <https://docs.sentry.io/platforms/rust/guides/actix-web/>
    // DSN is from SENTRY_DSN env variable.
    // Has no effect if not set.
    let sentry = sentry::init(sentry::ClientOptions {
        release: sentry::release_name!(),
        traces_sample_rate: ENV.SENTRY_TRACES_SAMPLE_RATE,
        environment: Some((&ENV.SENTRY_ENVIRONMENT).into()),
        ..Default::default()
    });
    if sentry.is_enabled() {
        info!("Enabled Sentry integration");
        unsafe {
            std::env::set_var("RUST_BACKTRACE", "1");
        }
    }

    actix_rt::System::new().block_on(app())?;

    // Sentry guard must live until the end of the app
    drop(sentry);
    Ok(())
}

async fn app() -> std::io::Result<()> {
    let args = Args::parse();

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    if args.run_background_task.is_none() {
        info!("Starting labrinth on {}", &ENV.BIND_ADDR);

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
    let redis_pool = RedisPool::new("");

    let storage_backend = ENV.STORAGE_BACKEND;
    let file_host: Arc<dyn file_hosting::FileHost + Send + Sync> =
        match storage_backend {
            FileHostKind::S3 => {
                let not_empty = |v: &str| -> String {
                    assert!(!v.is_empty(), "S3 env var is empty");
                    v.to_string()
                };

                Arc::new(
                    S3Host::new(
                        S3BucketConfig {
                            name: not_empty(&ENV.S3_PUBLIC_BUCKET_NAME),
                            uses_path_style: ENV
                                .S3_PUBLIC_USES_PATH_STYLE_BUCKET,
                            region: not_empty(&ENV.S3_PUBLIC_REGION),
                            url: not_empty(&ENV.S3_PUBLIC_URL),
                            access_token: not_empty(
                                &ENV.S3_PUBLIC_ACCESS_TOKEN,
                            ),
                            secret: not_empty(&ENV.S3_PUBLIC_SECRET),
                        },
                        S3BucketConfig {
                            name: not_empty(&ENV.S3_PRIVATE_BUCKET_NAME),
                            uses_path_style: ENV
                                .S3_PRIVATE_USES_PATH_STYLE_BUCKET,
                            region: not_empty(&ENV.S3_PRIVATE_REGION),
                            url: not_empty(&ENV.S3_PRIVATE_URL),
                            access_token: not_empty(
                                &ENV.S3_PRIVATE_ACCESS_TOKEN,
                            ),
                            secret: not_empty(&ENV.S3_PRIVATE_SECRET),
                        },
                    )
                    .unwrap(),
                )
            }
            FileHostKind::Local => Arc::new(file_hosting::MockHost::new()),
        };

    info!("Initializing clickhouse connection");
    let mut clickhouse = clickhouse::init_client().await.unwrap();

    let search_backend =
        actix_web::web::Data::from(Arc::from(search::backend(None)));

    let stripe_client = stripe::Client::new(ENV.STRIPE_API_KEY.clone());

    let anrok_client = anrok::Client::from_env().unwrap();
    let email_queue =
        EmailQueue::init(pool.clone(), redis_pool.clone()).unwrap();

    let gotenberg_client = GotenbergClient::from_env(redis_pool.clone())
        .expect("Failed to create Gotenberg client");
    let muralpay = labrinth::queue::payouts::create_muralpay_client()
        .expect("Failed to create MuralPay client");

    if let Some(task) = args.run_background_task {
        info!("Running task {task:?} and exiting");
        return task
            .run(
                pool,
                ro_pool.into_inner(),
                redis_pool,
                search_backend,
                clickhouse,
                stripe_client,
                anrok_client.clone(),
                email_queue,
                muralpay,
            )
            .await
            .map_err(io::Error::other);
    }

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

    labrinth::routes::debug::register_and_set_metrics(&prometheus.registry)
        .expect("Failed to register debug metrics");

    let labrinth_config = labrinth::app_setup(
        pool.clone(),
        ro_pool.clone(),
        redis_pool.clone(),
        search_backend.clone(),
        &mut clickhouse,
        file_host.clone(),
        stripe_client,
        anrok_client.clone(),
        email_queue,
        gotenberg_client,
        !args.no_background_tasks,
    );

    info!("Starting Actix HTTP server!");

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap_fn(|req, srv| {
                // We capture the same fields as `tracing-actix-web`'s `RootSpanBuilder`.
                // See `root_span!` macro.
                let span = info_span!(
                    "HTTP request",
                    http.method = %req.method(),
                    http.client_ip = %req.connection_info().realip_remote_addr().unwrap_or(""),
                    http.user_agent = %req.headers().get("User-Agent").map_or("", |h| h.to_str().unwrap_or("")),
                    http.target = %req.uri().path_and_query().map_or("", |p| p.as_str()),
                    http.authenticated = %req.headers().get("Authorization").is_some()
                );

                let fut = srv.call(req);
                async move {
                    fut.await.inspect(|resp| {
                        let _span = info_span!(
                            "HTTP response",
                            http.status = %resp.response().status().as_u16(),
                        ).entered();

                        resp.response()
                            .error()
                            .inspect(|err| log_error(err));
                    })
                }
                .instrument(span)
            })
            .wrap(prometheus.clone())
            .wrap(from_fn(rate_limit_middleware))
            .wrap(actix_web::middleware::Compress::default())
            // Sentry integration
            // `sentry_actix::Sentry` provides an Actix middleware for making
            // transactions out of HTTP requests. However, we have to use our
            // own - See `sentry::SentryErrorReporting` for why.
            .wrap(labrinth::util::sentry::SentryErrorReporting)
            // Use `utoipa` for OpenAPI generation
            .into_utoipa_app()
            .configure(|cfg| utoipa_app_config(cfg, labrinth_config.clone()))
            .openapi_service(|api| SwaggerUi::new("/docs/swagger-ui/{_:.*}")
                .config(utoipa_swagger_ui::Config::default().try_it_out_enabled(true))
                .url("/docs/openapi.json", ApiDoc::openapi().merge_from(api)))
            .into_app()
            .configure(|cfg| app_config(cfg, labrinth_config.clone()))
    })
    .bind(&ENV.BIND_ADDR)?
    .run()
    .await
}

#[derive(utoipa::OpenApi)]
#[openapi(info(title = "Labrinth"), modifiers(&SecurityAddon))]
struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new(
                "authorization",
            ))),
        );
    }
}

fn log_error(err: &actix_web::Error) {
    if err.as_response_error().status_code().is_client_error() {
        tracing::debug!(
            "Error encountered while processing the incoming HTTP request: {err:#}"
        );
    } else {
        tracing::error!(
            "Error encountered while processing the incoming HTTP request: {err:#}"
        );
    }
}
