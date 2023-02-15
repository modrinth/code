use crate::file_hosting::S3Host;
use crate::queue::download::DownloadQueue;
use crate::queue::payouts::PayoutsQueue;
use crate::ratelimit::errors::ARError;
use crate::ratelimit::memory::{MemoryStore, MemoryStoreActor};
use crate::ratelimit::middleware::RateLimiter;
use crate::util::env::{parse_strings_from_var, parse_var};
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use env_logger::Env;
use log::{error, info, warn};
use search::indexing::index_projects;
use search::indexing::IndexingSettings;
use std::sync::Arc;
use tokio::sync::Mutex;

mod database;
mod file_hosting;
mod health;
mod models;
mod queue;
mod ratelimit;
mod routes;
mod scheduler;
mod search;
mod util;
mod validate;

#[derive(Clone)]
pub struct Pepper {
    pub pepper: String,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
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
        enable_profiling: true,
        profiles_sample_rate: 0.1,
        ..Default::default()
    });
    if sentry.is_enabled() {
        info!("Enabled Sentry integration");
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    info!(
        "Starting Labrinth on {}",
        dotenvy::var("BIND_ADDR").unwrap()
    );

    let search_config = search::SearchConfig {
        address: dotenvy::var("MEILISEARCH_ADDR").unwrap(),
        key: dotenvy::var("MEILISEARCH_KEY").unwrap(),
    };

    database::check_for_migrations()
        .await
        .expect("An error occurred while running migrations.");

    // Database Connector
    let pool = database::connect()
        .await
        .expect("Database connection failed");

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

    let mut scheduler = scheduler::Scheduler::new();

    // The interval in seconds at which the local database is indexed
    // for searching.  Defaults to 1 hour if unset.
    let local_index_interval = std::time::Duration::from_secs(
        parse_var("LOCAL_INDEX_INTERVAL").unwrap_or(3600),
    );

    let pool_ref = pool.clone();
    let search_config_ref = search_config.clone();
    scheduler.run(local_index_interval, move || {
        let pool_ref = pool_ref.clone();
        let search_config_ref = search_config_ref.clone();
        async move {
            info!("Indexing local database");
            let settings = IndexingSettings { index_local: true };
            let result =
                index_projects(pool_ref, settings, &search_config_ref).await;
            if let Err(e) = result {
                warn!("Local project indexing failed: {:?}", e);
            }
            info!("Done indexing local database");
        }
    });

    // Deleting old authentication states from the database every 15 minutes
    let pool_ref = pool.clone();
    scheduler.run(std::time::Duration::from_secs(15 * 60), move || {
        let pool_ref = pool_ref.clone();
        // Use sqlx to delete records more than an hour old
        info!("Deleting old records from temporary tables");

        async move {
            let states_result = sqlx::query!(
                "
                DELETE FROM states
                WHERE expires < CURRENT_DATE
                "
            )
            .execute(&pool_ref)
            .await;

            if let Err(e) = states_result {
                warn!(
                    "Deleting old records from temporary table states failed: {:?}",
                    e
                );
            }

            info!("Finished deleting old records from temporary tables");
        }
    });

    // Changes statuses of scheduled projects/versions
    let pool_ref = pool.clone();
    scheduler.run(std::time::Duration::from_secs(60), move || {
        let pool_ref = pool_ref.clone();
        info!("Releasing scheduled versions/projects!");

        async move {
            let projects_results = sqlx::query!(
                "
                UPDATE mods
                SET status = requested_status
                WHERE status = $1 AND approved < CURRENT_DATE AND requested_status IS NOT NULL
                ",
                crate::models::projects::ProjectStatus::Scheduled.as_str(),
            )
                .execute(&pool_ref)
                .await;

            if let Err(e) = projects_results {
                warn!(
                    "Syncing scheduled releases for projects failed: {:?}",
                    e
                );
            }

            let versions_results = sqlx::query!(
                "
                UPDATE versions
                SET status = requested_status
                WHERE status = $1 AND date_published < CURRENT_DATE AND requested_status IS NOT NULL
                ",
                crate::models::projects::VersionStatus::Scheduled.as_str(),
            )
                .execute(&pool_ref)
                .await;

            if let Err(e) = versions_results {
                warn!(
                    "Syncing scheduled releases for versions failed: {:?}",
                    e
                );
            }

            info!("Finished releasing scheduled versions/projects");
        }
    });

    scheduler::schedule_versions(&mut scheduler, pool.clone());

    let download_queue = Arc::new(DownloadQueue::new());

    let pool_ref = pool.clone();
    let download_queue_ref = download_queue.clone();
    scheduler.run(std::time::Duration::from_secs(30), move || {
        let pool_ref = pool_ref.clone();
        let download_queue_ref = download_queue_ref.clone();

        async move {
            info!("Indexing download queue");
            let result = download_queue_ref.index(&pool_ref).await;
            if let Err(e) = result {
                warn!("Indexing download queue failed: {:?}", e);
            }
            info!("Done indexing download queue");
        }
    });

    let ip_salt = Pepper {
        pepper: models::ids::Base62Id(models::ids::random_base62(11))
            .to_string(),
    };

    let payouts_queue = Arc::new(Mutex::new(PayoutsQueue::new()));

    let store = MemoryStore::new();

    info!("Starting Actix HTTP server!");

    // Init App
    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Compress::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_header()
                    .allow_any_method()
                    .max_age(3600)
                    .send_wildcard(),
            )
            .wrap(
                RateLimiter::new(MemoryStoreActor::from(store.clone()).start())
                    .with_identifier(|req| {
                        let connection_info = req.connection_info();
                        let ip = String::from(
                            if parse_var("CLOUDFLARE_INTEGRATION")
                                .unwrap_or(false)
                            {
                                if let Some(header) =
                                    req.headers().get("CF-Connecting-IP")
                                {
                                    header
                                        .to_str()
                                        .map_err(|_| ARError::Identification)?
                                } else {
                                    connection_info
                                        .peer_addr()
                                        .ok_or(ARError::Identification)?
                                }
                            } else {
                                connection_info
                                    .peer_addr()
                                    .ok_or(ARError::Identification)?
                            },
                        );

                        Ok(ip)
                    })
                    .with_interval(std::time::Duration::from_secs(60))
                    .with_max_requests(300)
                    .with_ignore_key(
                        dotenvy::var("RATE_LIMIT_IGNORE_KEY").ok(),
                    ),
            )
            .app_data(web::FormConfig::default().error_handler(|err, _req| {
                routes::ApiError::Validation(err.to_string()).into()
            }))
            .app_data(web::PathConfig::default().error_handler(|err, _req| {
                routes::ApiError::Validation(err.to_string()).into()
            }))
            .app_data(web::QueryConfig::default().error_handler(|err, _req| {
                routes::ApiError::Validation(err.to_string()).into()
            }))
            .app_data(web::JsonConfig::default().error_handler(|err, _req| {
                routes::ApiError::Validation(err.to_string()).into()
            }))
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(file_host.clone()))
            .app_data(web::Data::new(search_config.clone()))
            .app_data(web::Data::new(download_queue.clone()))
            .app_data(web::Data::new(payouts_queue.clone()))
            .app_data(web::Data::new(ip_salt.clone()))
            .wrap(sentry_actix::Sentry::new())
            .configure(routes::v1_config)
            .configure(routes::v2_config)
            .service(routes::index_get)
            .service(routes::health_get)
            .service(web::scope("maven").configure(routes::maven_config))
            .service(web::scope("updates").configure(routes::updates))
            .default_service(web::get().to(routes::not_found))
    })
    .bind(dotenvy::var("BIND_ADDR").unwrap())?
    .run()
    .await
}

// This is so that env vars not used immediately don't panic at runtime
fn check_env_vars() -> bool {
    let mut failed = false;

    fn check_var<T: std::str::FromStr>(var: &'static str) -> bool {
        let check = parse_var::<T>(var).is_none();
        if check {
            warn!(
                "Variable `{}` missing in dotenv or not of type `{}`",
                var,
                std::any::type_name::<T>()
            );
        }
        check
    }

    if parse_strings_from_var("WHITELISTED_MODPACK_DOMAINS").is_none() {
        warn!("Variable `WHITELISTED_MODPACK_DOMAINS` missing in dotenv or not a json array of strings");
        failed |= true;
    }

    if parse_strings_from_var("ALLOWED_CALLBACK_URLS").is_none() {
        warn!("Variable `ALLOWED_CALLBACK_URLS` missing in dotenv or not a json array of strings");
        failed |= true;
    }

    failed |= check_var::<String>("SITE_URL");
    failed |= check_var::<String>("CDN_URL");
    failed |= check_var::<String>("LABRINTH_ADMIN_KEY");
    failed |= check_var::<String>("RATE_LIMIT_IGNORE_KEY");
    failed |= check_var::<String>("DATABASE_URL");
    failed |= check_var::<String>("MEILISEARCH_ADDR");
    failed |= check_var::<String>("MEILISEARCH_KEY");
    failed |= check_var::<String>("BIND_ADDR");

    failed |= check_var::<String>("STORAGE_BACKEND");

    let storage_backend = dotenvy::var("STORAGE_BACKEND").ok();
    match storage_backend.as_deref() {
        Some("backblaze") => {
            failed |= check_var::<String>("BACKBLAZE_KEY_ID");
            failed |= check_var::<String>("BACKBLAZE_KEY");
            failed |= check_var::<String>("BACKBLAZE_BUCKET_ID");
        }
        Some("s3") => {
            failed |= check_var::<String>("S3_ACCESS_TOKEN");
            failed |= check_var::<String>("S3_SECRET");
            failed |= check_var::<String>("S3_URL");
            failed |= check_var::<String>("S3_REGION");
            failed |= check_var::<String>("S3_BUCKET_NAME");
        }
        Some("local") => {
            failed |= check_var::<String>("MOCK_FILE_PATH");
        }
        Some(backend) => {
            warn!("Variable `STORAGE_BACKEND` contains an invalid value: {}. Expected \"backblaze\", \"s3\", or \"local\".", backend);
            failed |= true;
        }
        _ => {
            warn!("Variable `STORAGE_BACKEND` is not set!");
            failed |= true;
        }
    }
    failed |= check_var::<usize>("LOCAL_INDEX_INTERVAL");

    failed |= check_var::<usize>("VERSION_INDEX_INTERVAL");

    failed |= check_var::<String>("GITHUB_CLIENT_ID");
    failed |= check_var::<String>("GITHUB_CLIENT_SECRET");

    failed |= check_var::<String>("ARIADNE_ADMIN_KEY");
    failed |= check_var::<String>("ARIADNE_URL");

    failed |= check_var::<String>("STRIPE_TOKEN");
    failed |= check_var::<String>("STRIPE_WEBHOOK_SECRET");

    failed |= check_var::<String>("PAYPAL_API_URL");
    failed |= check_var::<String>("PAYPAL_CLIENT_ID");
    failed |= check_var::<String>("PAYPAL_CLIENT_SECRET");

    failed
}
