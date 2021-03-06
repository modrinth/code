use crate::file_hosting::S3Host;
use actix_cors::Cors;
use actix_ratelimit::errors::ARError;
use actix_ratelimit::{MemoryStore, MemoryStoreActor, RateLimiter};
use actix_web::{http, web, App, HttpServer};
use env_logger::Env;
use gumdrop::Options;
use log::{error, info, warn};
use rand::Rng;
use search::indexing::index_mods;
use search::indexing::IndexingSettings;
use std::sync::Arc;

mod auth;
mod database;
mod file_hosting;
mod models;
mod routes;
mod scheduler;
mod search;

#[derive(Debug, Options)]
struct Config {
    #[options(help = "Print help message")]
    help: bool,

    #[options(no_short, help = "Skip indexing on startup")]
    skip_first_index: bool,
    #[options(no_short, help = "Reset the settings of the indices")]
    reconfigure_indices: bool,
    #[options(no_short, help = "Reset the documents in the indices")]
    reset_indices: bool,

    #[options(
        no_short,
        help = "Allow missing environment variables on startup. This is a bad idea, but it may work in some cases."
    )]
    allow_missing_vars: bool,
}

#[derive(Clone)]
pub struct Pepper {
    pub pepper: String,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let config = Config::parse_args_default_or_exit();

    if check_env_vars() {
        error!("Some environment variables are missing!");
        if !config.allow_missing_vars {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Missing required environment variables",
            ));
        }
    }

    let search_config = search::SearchConfig {
        address: dotenv::var("MEILISEARCH_ADDR").unwrap(),
        key: dotenv::var("MEILISEARCH_KEY").unwrap(),
    };

    if config.reset_indices {
        info!("Resetting indices");
        search::indexing::reset_indices(&search_config)
            .await
            .unwrap();
        return Ok(());
    } else if config.reconfigure_indices {
        info!("Reconfiguring indices");
        search::indexing::reconfigure_indices(&search_config)
            .await
            .unwrap();
        return Ok(());
    }

    // Allow manually skipping the initial indexing for quicker iteration
    // and startup times.
    let skip_initial = config.skip_first_index;
    if skip_initial {
        info!("Skipping initial indexing");
    }

    // DSN is from SENTRY_DSN env variable.
    // Has no effect if not set.
    let sentry = sentry::init(());
    if sentry.is_enabled() {
        info!("Enabled Sentry integration");
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    database::check_for_migrations()
        .await
        .expect("An error occurred while running migrations.");

    // Database Connector
    let pool = database::connect()
        .await
        .expect("Database connection failed");

    let storage_backend = dotenv::var("STORAGE_BACKEND").unwrap_or_else(|_| "local".to_string());

    let file_host: Arc<dyn file_hosting::FileHost + Send + Sync> = if storage_backend == "backblaze"
    {
        Arc::new(
            file_hosting::BackblazeHost::new(
                &dotenv::var("BACKBLAZE_KEY_ID").unwrap(),
                &dotenv::var("BACKBLAZE_KEY").unwrap(),
                &dotenv::var("BACKBLAZE_BUCKET_ID").unwrap(),
            )
            .await,
        )
    } else if storage_backend == "s3" {
        Arc::new(
            S3Host::new(
                &*dotenv::var("S3_BUCKET_NAME").unwrap(),
                &*dotenv::var("S3_REGION").unwrap(),
                &*dotenv::var("S3_URL").unwrap(),
                &*dotenv::var("S3_ACCESS_TOKEN").unwrap(),
                &*dotenv::var("S3_SECRET").unwrap(),
            )
            .unwrap(),
        )
    } else if storage_backend == "local" {
        Arc::new(file_hosting::MockHost::new())
    } else {
        panic!("Invalid storage backend specified. Aborting startup!")
    };

    let mut scheduler = scheduler::Scheduler::new();

    // The interval in seconds at which the local database is indexed
    // for searching.  Defaults to 1 hour if unset.
    let local_index_interval = std::time::Duration::from_secs(
        dotenv::var("LOCAL_INDEX_INTERVAL")
            .ok()
            .map(|i| i.parse().unwrap())
            .unwrap_or(3600),
    );

    let pool_ref = pool.clone();
    let thread_search_config = search_config.clone();
    let mut skip = skip_initial;
    scheduler.run(local_index_interval, move || {
        let pool_ref = pool_ref.clone();
        let thread_search_config = thread_search_config.clone();
        let local_skip = skip;
        if skip {
            skip = false;
        }
        async move {
            if local_skip {
                return;
            }
            info!("Indexing local database");
            let settings = IndexingSettings {
                index_local: true,
                index_external: false,
            };
            let result = index_mods(pool_ref, settings, &thread_search_config).await;
            if let Err(e) = result {
                warn!("Local mod indexing failed: {:?}", e);
            }
            info!("Done indexing local database");
        }
    });

    let pool_ref = pool.clone();
    scheduler.run(std::time::Duration::from_secs(15 * 60), move || {
        let pool_ref = pool_ref.clone();
        // Use sqlx to delete records more than an hour old
        info!("Deleting old records from temporary tables");

        async move {
            let downloads_result = sqlx::query!(
                "
                DELETE FROM downloads
                WHERE date < (CURRENT_DATE - INTERVAL '30 minutes ago')
                "
            )
            .execute(&pool_ref)
            .await;

            if let Err(e) = downloads_result {
                warn!(
                    "Deleting old records from temporary table downloads failed: {:?}",
                    e
                );
            }

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

    let indexing_queue = Arc::new(search::indexing::queue::CreationQueue::new());

    let queue_ref = indexing_queue.clone();
    let thread_search_config = search_config.clone();
    let mut skip = skip_initial;
    scheduler.run(std::time::Duration::from_secs(15 * 60), move || {
        let queue = queue_ref.clone();
        let thread_search_config = thread_search_config.clone();
        let local_skip = skip;
        if skip {
            skip = false;
        }
        async move {
            if local_skip {
                return;
            }
            info!("Indexing created mod queue");
            let result = search::indexing::queue::index_queue(&*queue, &thread_search_config).await;
            if let Err(e) = result {
                warn!("Indexing created mods failed: {:?}", e);
            }
            info!("Done indexing created mod queue");
        }
    });

    scheduler::schedule_versions(&mut scheduler, pool.clone(), skip_initial);

    let ip_salt = Pepper {
        pepper: crate::models::ids::Base62Id(crate::models::ids::random_base62(11)).to_string(),
    };

    let store = MemoryStore::new();

    info!("Starting Actix HTTP server!");

    // Init App
    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::new()
                    .allowed_methods(vec!["GET", "POST", "DELETE", "PATCH", "PUT"])
                    .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                    .allowed_header(http::header::CONTENT_TYPE)
                    .send_wildcard()
                    .max_age(3600)
                    .finish(),
            )
            .wrap(
                // This is a hacky workaround to allowing the frontend server-side renderer to have
                // an unlimited rate limit, since there is no current way with this library to
                // have dynamic rate-limit max requests
                RateLimiter::new(MemoryStoreActor::from(store.clone()).start())
                    .with_identifier(|req| {
                        let connection_info = req.connection_info();
                        let ip = String::from(
                            connection_info
                                .remote_addr()
                                .ok_or(ARError::IdentificationError)?,
                        );

                        let ignore_ips = dotenv::var("RATE_LIMIT_IGNORE_IPS")
                            .ok()
                            .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
                            .unwrap_or_else(Vec::new);

                        if ignore_ips.contains(&ip) {
                            // At an even distribution of numbers, this will allow at the most
                            // 3000 requests per minute from the frontend, which is reasonable
                            // (50 requests per second)
                            let random = rand::thread_rng().gen_range(1, 15);
                            return Ok(format!("{}-{}", ip, random));
                        }

                        Ok(ip)
                    })
                    .with_interval(std::time::Duration::from_secs(60))
                    .with_max_requests(200),
            )
            .wrap(sentry_actix::Sentry::new())
            .data(pool.clone())
            .data(file_host.clone())
            .data(indexing_queue.clone())
            .data(search_config.clone())
            .data(ip_salt.clone())
            .service(routes::index_get)
            .service(
                web::scope("/api/v1/")
                    .configure(routes::auth_config)
                    .configure(routes::tags_config)
                    .configure(routes::mods_config)
                    .configure(routes::versions_config)
                    .configure(routes::teams_config)
                    .configure(routes::users_config)
                    .configure(routes::moderation_config)
                    .configure(routes::reports_config)
                    .configure(routes::notifications_config),
            )
            .default_service(web::get().to(routes::not_found))
    })
    .bind(dotenv::var("BIND_ADDR").unwrap())?
    .run()
    .await
}

// This is so that env vars not used immediately don't panic at runtime
fn check_env_vars() -> bool {
    let mut failed = false;

    fn check_var<T: std::str::FromStr>(var: &str) -> bool {
        if dotenv::var(var)
            .ok()
            .and_then(|s| s.parse::<T>().ok())
            .is_none()
        {
            warn!(
                "Variable `{}` missing in dotenv or not of type `{}`",
                var,
                std::any::type_name::<T>()
            );
            true
        } else {
            false
        }
    }

    if dotenv::var("RATE_LIMIT_IGNORE_IPS")
        .ok()
        .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
        .is_none()
    {
        warn!("Variable `RATE_LIMIT_IGNORE_IPS` missing in dotenv or not a json array of strings");
        failed |= true;
    }

    failed |= check_var::<String>("CDN_URL");
    failed |= check_var::<String>("DATABASE_URL");
    failed |= check_var::<String>("MEILISEARCH_ADDR");
    failed |= check_var::<String>("MEILISEARCH_KEY");
    failed |= check_var::<String>("BIND_ADDR");

    failed |= check_var::<String>("STORAGE_BACKEND");

    let storage_backend = dotenv::var("STORAGE_BACKEND").ok();

    if storage_backend.as_deref() == Some("backblaze") {
        failed |= check_var::<String>("BACKBLAZE_KEY_ID");
        failed |= check_var::<String>("BACKBLAZE_KEY");
        failed |= check_var::<String>("BACKBLAZE_BUCKET_ID");
    } else if storage_backend.as_deref() == Some("s3") {
        failed |= check_var::<String>("S3_ACCESS_TOKEN");
        failed |= check_var::<String>("S3_SECRET");
        failed |= check_var::<String>("S3_URL");
        failed |= check_var::<String>("S3_REGION");
        failed |= check_var::<String>("S3_BUCKET_NAME");
    } else if storage_backend.as_deref() == Some("local") {
        failed |= check_var::<String>("MOCK_FILE_PATH");
    } else if let Some(backend) = storage_backend {
        warn!("Variable `STORAGE_BACKEND` contains an invalid value: {}. Expected \"backblaze\", \"s3\", or \"local\".", backend);
        failed |= true;
    }

    failed |= check_var::<usize>("LOCAL_INDEX_INTERVAL");

    failed |= check_var::<usize>("VERSION_INDEX_INTERVAL");

    failed |= check_var::<String>("GITHUB_CLIENT_ID");
    failed |= check_var::<String>("GITHUB_CLIENT_SECRET");

    failed
}
