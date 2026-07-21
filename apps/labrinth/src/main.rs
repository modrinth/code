#![recursion_limit = "256"]

use actix_web::dev::Service;
use actix_web::middleware::from_fn;
use actix_web::{App, HttpResponse, HttpServer, web};
use actix_web_prom::PrometheusMetricsBuilder;
use clap::Parser;

use labrinth::background_task::BackgroundTask;
use labrinth::database::redis::RedisPool;
use labrinth::env::ENV;
use labrinth::file_hosting::{FileHost, FileHostKind, S3BucketConfig, S3Host};
use labrinth::queue::email::EmailQueue;
use labrinth::search;
use labrinth::util::anrok;
use labrinth::util::gotenberg::GotenbergClient;
use labrinth::util::ratelimit::rate_limit_middleware;
use labrinth::{app_data_config, app_fallback_config, app_routes_config, env};
use labrinth::{clickhouse, database, file_hosting};
use scalar_api_reference::actix_web::config as scalar_config;
use serde_json::json;
use std::ffi::CStr;
use std::sync::Arc;
use tracing::{Instrument, info, info_span};
use tracing_actix_web::TracingLogger;
use utoipa::OpenApi;
use utoipa::PartialSchema;
use utoipa::openapi::Content;
use utoipa::openapi::response::Response;
use utoipa::openapi::schema::Components;

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
    let file_host: Arc<dyn FileHost> = match storage_backend {
        FileHostKind::S3 => {
            let not_empty = |v: &str| -> String {
                assert!(!v.is_empty(), "S3 env var is empty");
                v.to_string()
            };

            Arc::new(
                S3Host::new(
                    S3BucketConfig {
                        name: not_empty(&ENV.S3_PUBLIC_BUCKET_NAME),
                        uses_path_style: ENV.S3_PUBLIC_USES_PATH_STYLE_BUCKET,
                        region: not_empty(&ENV.S3_PUBLIC_REGION),
                        url: not_empty(&ENV.S3_PUBLIC_URL),
                        access_token: not_empty(&ENV.S3_PUBLIC_ACCESS_TOKEN),
                        secret: not_empty(&ENV.S3_PUBLIC_SECRET),
                    },
                    S3BucketConfig {
                        name: not_empty(&ENV.S3_PRIVATE_BUCKET_NAME),
                        uses_path_style: ENV.S3_PRIVATE_USES_PATH_STYLE_BUCKET,
                        region: not_empty(&ENV.S3_PRIVATE_REGION),
                        url: not_empty(&ENV.S3_PRIVATE_URL),
                        access_token: not_empty(&ENV.S3_PRIVATE_ACCESS_TOKEN),
                        secret: not_empty(&ENV.S3_PRIVATE_SECRET),
                    },
                )
                .unwrap(),
            )
        }
        FileHostKind::Local => Arc::new(file_hosting::MockHost::new()),
    };
    let file_host = web::Data::<dyn FileHost>::from(file_host);

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
    let kafka_client = actix_web::web::Data::new(
        labrinth::util::kafka::KafkaClientState::new()
            .expect("Kafka connection failed"),
    );

    if let Some(task) = args.run_background_task {
        info!("Running task {task:?} and exiting");
        task.run(
            pool,
            ro_pool.into_inner(),
            redis_pool,
            search_backend,
            file_host,
            kafka_client,
            clickhouse,
            stripe_client,
            anrok_client.clone(),
            email_queue,
            muralpay,
        )
        .await
        .map_err(std::io::Error::other)?;
        return Ok(());
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
        kafka_client,
        !args.no_background_tasks,
    );

    info!("Starting Actix HTTP server!");

    HttpServer::new(move || {
        let mut docs_v2 = labrinth::routes::v2::ApiDoc::openapi();
        let mut docs_v3 = labrinth::routes::v3::ApiDoc::openapi();
        let mut docs_internal = labrinth::routes::internal::ApiDoc::openapi();
        #[cfg(target_os = "linux")]
        docs_v3.merge(labrinth::routes::debug::ApiDoc::openapi());
        document_error_responses(&mut docs_v2);
        document_error_responses(&mut docs_v3);
        document_error_responses(&mut docs_internal);

        let app = App::new()
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
            .configure(|cfg| app_data_config(cfg, labrinth_config.clone()))
            .configure(|cfg| app_routes_config(cfg, labrinth_config.clone()));

        let scalar_configuration = json!({
            "sources": [
                {
                    "title": "API v2",
                    "slug": "v2",
                    "url": "/openapi/v2.json",
                    "default": true
                },
                {
                    "title": "API v3 (UNSTABLE)",
                    "slug": "v3",
                    "url": "/openapi/v3.json"
                },
                {
                    "title": "Internal API (HIGHLY UNSTABLE)",
                    "slug": "internal",
                    "url": "/openapi/internal.json"
                }
            ],
            "agent": {
                "disabled": true
            },
            "mcp": {
                "disabled": true
            },
            "telemetry": false,

            "metaData": {
                "title": "Modrinth API Documentation",
                "description": "Reference documentation for the Modrinth API.",
                "ogTitle": "Modrinth API Documentation",
                "ogDescription": "Reference documentation for the Modrinth API."
            },

            "modelsSectionLabel": "Schemas",
            "defaultOpenFirstTag": true,
            "defaultOpenAllTags": false,
            "expandAllResponses": false,
            "expandAllSchemaProperties": false,
            "expandAllModelSections": false,
            "orderSchemaPropertiesBy": "preserve",
            "orderRequiredPropertiesFirst": true,
            "hideSearch": false,
            "searchHotKey": "k",
            "showOperationId": false,

            "defaultHttpClient": {
                "targetKey": "shell",
                "clientKey": "curl"
            },

            "persistAuth": false,
            "showDeveloperTools": "never",
        });

        app.service(openapi_json_service("/openapi/v2.json", docs_v2))
            .service(openapi_json_service("/openapi/v3.json", docs_v3))
            .service(openapi_json_service(
                "/openapi/internal.json",
                docs_internal,
            ))
            .configure(scalar_config("/docs", &scalar_configuration))
            .configure(app_fallback_config)
    })
    .bind(&ENV.BIND_ADDR)?
    .run()
    .await
}

fn openapi_json_service(
    path: &'static str,
    openapi: utoipa::openapi::OpenApi,
) -> actix_web::Resource {
    web::resource(path).route(web::get().to(move || {
        let openapi = openapi.clone();
        async move { openapi_json(openapi) }
    }))
}

fn openapi_json(openapi: utoipa::openapi::OpenApi) -> HttpResponse {
    match serde_json::to_string_pretty(&openapi) {
        Ok(body) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .body(body),
        Err(error) => {
            tracing::error!(%error, "Failed to serialize OpenAPI schema");
            HttpResponse::InternalServerError().finish()
        }
    }
}

fn document_error_responses(openapi: &mut utoipa::openapi::OpenApi) {
    let components = openapi.components.get_or_insert_with(Components::new);
    components.schemas.insert(
        "ApiError".to_string(),
        labrinth::models::error::ApiError::schema(),
    );

    for path_item in openapi.paths.paths.values_mut() {
        add_default_error_response(&mut path_item.get);
        add_default_error_response(&mut path_item.put);
        add_default_error_response(&mut path_item.post);
        add_default_error_response(&mut path_item.delete);
        add_default_error_response(&mut path_item.options);
        add_default_error_response(&mut path_item.head);
        add_default_error_response(&mut path_item.patch);
        add_default_error_response(&mut path_item.trace);
    }
}

fn add_default_error_response(
    operation: &mut Option<utoipa::openapi::path::Operation>,
) {
    if let Some(operation) = operation {
        for (status, response) in &mut operation.responses.responses {
            if !is_error_response_status(status) {
                continue;
            }

            if let utoipa::openapi::RefOr::T(response) = response {
                add_error_content(response);
            }
        }

        operation
            .responses
            .responses
            .entry("500".to_string())
            .or_insert_with(|| error_response().into());
    }
}

fn is_error_response_status(status: &str) -> bool {
    matches!(status.as_bytes().first(), Some(b'4' | b'5'))
        || status == "default"
}

fn error_response() -> Response {
    let mut response = Response::new("Error response");
    add_error_content(&mut response);
    response
}

fn add_error_content(response: &mut Response) {
    if response.content.is_empty() {
        response.content.insert(
            "application/json".to_string(),
            Content::new(Some(utoipa::openapi::Ref::from_schema_name(
                "ApiError",
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
