use crate::file_hosting::S3Host;
use crate::queue::analytics::AnalyticsQueue;
use crate::queue::download::DownloadQueue;
use crate::queue::payouts::{process_payout, PayoutsQueue};
use crate::queue::session::AuthQueue;
use crate::queue::socket::ActiveSockets;
use crate::ratelimit::errors::ARError;
use crate::ratelimit::memory::{MemoryStore, MemoryStoreActor};
use crate::ratelimit::middleware::RateLimiter;
use crate::util::cors::default_cors;
use crate::util::env::{parse_strings_from_var, parse_var};
use actix_web::{web, App, HttpServer};
use chrono::{DateTime, Utc};
use deadpool_redis::{Config, Runtime};
use env_logger::Env;
use log::{error, info, warn};
use search::indexing::index_projects;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

mod auth;
mod clickhouse;
mod database;
mod file_hosting;
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
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

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

    // Redis connector
    let redis_cfg = Config::from_url(dotenvy::var("REDIS_URL").expect("Redis URL not set"));
    let redis_pool = redis_cfg
        .create_pool(Some(Runtime::Tokio1))
        .expect("Redis connection failed");

    let storage_backend = dotenvy::var("STORAGE_BACKEND").unwrap_or_else(|_| "local".to_string());

    let file_host: Arc<dyn file_hosting::FileHost + Send + Sync> = match storage_backend.as_str() {
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
    let local_index_interval =
        std::time::Duration::from_secs(parse_var("LOCAL_INDEX_INTERVAL").unwrap_or(3600));

    let pool_ref = pool.clone();
    let search_config_ref = search_config.clone();
    scheduler.run(local_index_interval, move || {
        let pool_ref = pool_ref.clone();
        let search_config_ref = search_config_ref.clone();
        async move {
            info!("Indexing local database");
            let result = index_projects(pool_ref, &search_config_ref).await;
            if let Err(e) = result {
                warn!("Local project indexing failed: {:?}", e);
            }
            info!("Done indexing local database");
        }
    });

    // Changes statuses of scheduled projects/versions
    let pool_ref = pool.clone();
    // TODO: Clear cache when these are run
    scheduler.run(std::time::Duration::from_secs(60 * 5), move || {
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
                warn!("Syncing scheduled releases for projects failed: {:?}", e);
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
                warn!("Syncing scheduled releases for versions failed: {:?}", e);
            }

            info!("Finished releasing scheduled versions/projects");
        }
    });

    // Reminding moderators to review projects which have been in the queue longer than 40hr
    let pool_ref = pool.clone();
    let webhook_message_sent = Arc::new(Mutex::new(Vec::<(
        database::models::ProjectId,
        DateTime<Utc>,
    )>::new()));

    scheduler.run(std::time::Duration::from_secs(10 * 60), move || {
        let pool_ref = pool_ref.clone();
        let webhook_message_sent_ref = webhook_message_sent.clone();
        info!("Checking reviewed projects submitted more than 40hrs ago");

        async move {
            let do_steps = async {
                use futures::TryStreamExt;

                let project_ids = sqlx::query!(
                    "
                    SELECT id FROM mods
                    WHERE status = $1 AND queued < NOW() - INTERVAL '40 hours'
                    ORDER BY updated ASC
                    ",
                    crate::models::projects::ProjectStatus::Processing.as_str(),
                )
                    .fetch_many(&pool_ref)
                    .try_filter_map(|e| async {
                        Ok(e.right().map(|m| database::models::ProjectId(m.id)))
                    })
                    .try_collect::<Vec<database::models::ProjectId>>()
                    .await?;

                let mut webhook_message_sent_ref = webhook_message_sent_ref.lock().await;

                webhook_message_sent_ref.retain(|x| Utc::now() - x.1 < chrono::Duration::hours(12));

                for project in project_ids {
                    if webhook_message_sent_ref.iter().any(|x| x.0 == project) { continue; }

                    if let Ok(webhook_url) =
                        dotenvy::var("MODERATION_DISCORD_WEBHOOK")
                    {
                        util::webhook::send_discord_webhook(
                            project.into(),
                            &pool_ref,
                            webhook_url,
                            Some("<@&783155186491195394> This project has been in the queue for over 40 hours!".to_string()),
                        )
                            .await
                            .ok();

                        webhook_message_sent_ref.push((project, Utc::now()));
                    }
                }

                Ok::<(), routes::ApiError>(())
            };

            if let Err(e) = do_steps.await {
                warn!(
                    "Checking reviewed projects submitted more than 40hrs ago failed: {:?}",
                    e
                );
            }

            info!("Finished checking reviewed projects submitted more than 40hrs ago");
        }
    });

    scheduler::schedule_versions(&mut scheduler, pool.clone());

    let download_queue = web::Data::new(DownloadQueue::new());

    let pool_ref = pool.clone();
    let download_queue_ref = download_queue.clone();
    scheduler.run(std::time::Duration::from_secs(60 * 5), move || {
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

    let session_queue = web::Data::new(AuthQueue::new());

    let pool_ref = pool.clone();
    let redis_ref = redis_pool.clone();
    let session_queue_ref = session_queue.clone();
    scheduler.run(std::time::Duration::from_secs(60 * 30), move || {
        let pool_ref = pool_ref.clone();
        let redis_ref = redis_ref.clone();
        let session_queue_ref = session_queue_ref.clone();

        async move {
            info!("Indexing sessions queue");
            let result = session_queue_ref.index(&pool_ref, &redis_ref).await;
            if let Err(e) = result {
                warn!("Indexing sessions queue failed: {:?}", e);
            }
            info!("Done indexing sessions queue");
        }
    });

    info!("Initializing clickhouse connection");
    let clickhouse = clickhouse::init_client().await.unwrap();

    let reader = Arc::new(queue::maxmind::MaxMindIndexer::new().await.unwrap());
    {
        let reader_ref = reader.clone();
        scheduler.run(std::time::Duration::from_secs(60 * 60 * 24), move || {
            let reader_ref = reader_ref.clone();

            async move {
                info!("Downloading MaxMind GeoLite2 country database");
                let result = reader_ref.index().await;
                if let Err(e) = result {
                    warn!(
                        "Downloading MaxMind GeoLite2 country database failed: {:?}",
                        e
                    );
                }
                info!("Done downloading MaxMind GeoLite2 country database");
            }
        });
    }
    info!("Downloading MaxMind GeoLite2 country database");

    let analytics_queue = Arc::new(AnalyticsQueue::new());
    {
        let client_ref = clickhouse.clone();
        let analytics_queue_ref = analytics_queue.clone();
        scheduler.run(std::time::Duration::from_secs(60 * 5), move || {
            let client_ref = client_ref.clone();
            let analytics_queue_ref = analytics_queue_ref.clone();

            async move {
                info!("Indexing analytics queue");
                let result = analytics_queue_ref.index(client_ref).await;
                if let Err(e) = result {
                    warn!("Indexing analytics queue failed: {:?}", e);
                }
                info!("Done indexing analytics queue");
            }
        });
    }

    // {
    //     let pool_ref = pool.clone();
    //     let redis_ref = redis_pool.clone();
    //     let client_ref = clickhouse.clone();
    //     scheduler.run(std::time::Duration::from_secs(60 * 60 * 6), move || {
    //         let pool_ref = pool_ref.clone();
    //         let redis_ref = redis_ref.clone();
    //         let client_ref = client_ref.clone();
    //
    //         async move {
    //             info!("Done running payouts");
    //             let result = process_payout(&pool_ref, &redis_ref, &client_ref).await;
    //             if let Err(e) = result {
    //                 warn!("Payouts run failed: {:?}", e);
    //             }
    //             info!("Done running payouts");
    //         }
    //     });
    // }

    let ip_salt = Pepper {
        pepper: models::ids::Base62Id(models::ids::random_base62(11)).to_string(),
    };

    let payouts_queue = web::Data::new(Mutex::new(PayoutsQueue::new()));
    let active_sockets = web::Data::new(RwLock::new(ActiveSockets::default()));

    let store = MemoryStore::new();

    info!("Starting Actix HTTP server!");

    // Init App
    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Compress::default())
            .wrap(
                RateLimiter::new(MemoryStoreActor::from(store.clone()).start())
                    .with_identifier(|req| {
                        let connection_info = req.connection_info();
                        let ip =
                            String::from(if parse_var("CLOUDFLARE_INTEGRATION").unwrap_or(false) {
                                if let Some(header) = req.headers().get("CF-Connecting-IP") {
                                    header.to_str().map_err(|_| ARError::Identification)?
                                } else {
                                    connection_info.peer_addr().ok_or(ARError::Identification)?
                                }
                            } else {
                                connection_info.peer_addr().ok_or(ARError::Identification)?
                            });

                        Ok(ip)
                    })
                    .with_interval(std::time::Duration::from_secs(60))
                    .with_max_requests(300)
                    .with_ignore_key(dotenvy::var("RATE_LIMIT_IGNORE_KEY").ok()),
            )
            .wrap(sentry_actix::Sentry::new())
            .app_data(
                web::FormConfig::default().error_handler(|err, _req| {
                    routes::ApiError::Validation(err.to_string()).into()
                }),
            )
            .app_data(
                web::PathConfig::default().error_handler(|err, _req| {
                    routes::ApiError::Validation(err.to_string()).into()
                }),
            )
            .app_data(
                web::QueryConfig::default().error_handler(|err, _req| {
                    routes::ApiError::Validation(err.to_string()).into()
                }),
            )
            .app_data(
                web::JsonConfig::default().error_handler(|err, _req| {
                    routes::ApiError::Validation(err.to_string()).into()
                }),
            )
            .app_data(web::Data::new(redis_pool.clone()))
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(file_host.clone()))
            .app_data(web::Data::new(search_config.clone()))
            .app_data(download_queue.clone())
            .app_data(session_queue.clone())
            .app_data(payouts_queue.clone())
            .app_data(web::Data::new(ip_salt.clone()))
            .app_data(web::Data::new(analytics_queue.clone()))
            .app_data(web::Data::new(clickhouse.clone()))
            .app_data(web::Data::new(reader.clone()))
            .app_data(active_sockets.clone())
            .configure(routes::v2::config)
            .configure(routes::v3::config)
            .configure(routes::root_config)
            .default_service(web::get().wrap(default_cors()).to(routes::not_found))
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

    failed |= check_var::<String>("SITE_URL");
    failed |= check_var::<String>("CDN_URL");
    failed |= check_var::<String>("LABRINTH_ADMIN_KEY");
    failed |= check_var::<String>("RATE_LIMIT_IGNORE_KEY");
    failed |= check_var::<String>("DATABASE_URL");
    failed |= check_var::<String>("MEILISEARCH_ADDR");
    failed |= check_var::<String>("MEILISEARCH_KEY");
    failed |= check_var::<String>("REDIS_URL");
    failed |= check_var::<String>("BIND_ADDR");
    failed |= check_var::<String>("SELF_ADDR");

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

    if parse_strings_from_var("WHITELISTED_MODPACK_DOMAINS").is_none() {
        warn!("Variable `WHITELISTED_MODPACK_DOMAINS` missing in dotenv or not a json array of strings");
        failed |= true;
    }

    if parse_strings_from_var("ALLOWED_CALLBACK_URLS").is_none() {
        warn!("Variable `ALLOWED_CALLBACK_URLS` missing in dotenv or not a json array of strings");
        failed |= true;
    }

    failed |= check_var::<String>("PAYPAL_API_URL");
    failed |= check_var::<String>("PAYPAL_CLIENT_ID");
    failed |= check_var::<String>("PAYPAL_CLIENT_SECRET");

    failed |= check_var::<String>("GITHUB_CLIENT_ID");
    failed |= check_var::<String>("GITHUB_CLIENT_SECRET");
    failed |= check_var::<String>("GITLAB_CLIENT_ID");
    failed |= check_var::<String>("GITLAB_CLIENT_SECRET");
    failed |= check_var::<String>("DISCORD_CLIENT_ID");
    failed |= check_var::<String>("DISCORD_CLIENT_SECRET");
    failed |= check_var::<String>("MICROSOFT_CLIENT_ID");
    failed |= check_var::<String>("MICROSOFT_CLIENT_SECRET");
    failed |= check_var::<String>("GOOGLE_CLIENT_ID");
    failed |= check_var::<String>("GOOGLE_CLIENT_SECRET");
    failed |= check_var::<String>("STEAM_API_KEY");

    failed |= check_var::<String>("TURNSTILE_SECRET");

    failed |= check_var::<String>("SMTP_USERNAME");
    failed |= check_var::<String>("SMTP_PASSWORD");
    failed |= check_var::<String>("SMTP_HOST");

    failed |= check_var::<String>("SITE_VERIFY_EMAIL_PATH");
    failed |= check_var::<String>("SITE_RESET_PASSWORD_PATH");

    failed |= check_var::<String>("BEEHIIV_PUBLICATION_ID");
    failed |= check_var::<String>("BEEHIIV_API_KEY");

    if parse_strings_from_var("ANALYTICS_ALLOWED_ORIGINS").is_none() {
        warn!(
            "Variable `ANALYTICS_ALLOWED_ORIGINS` missing in dotenv or not a json array of strings"
        );
        failed |= true;
    }

    failed |= check_var::<String>("CLICKHOUSE_URL");
    failed |= check_var::<String>("CLICKHOUSE_USER");
    failed |= check_var::<String>("CLICKHOUSE_PASSWORD");
    failed |= check_var::<String>("CLICKHOUSE_DATABASE");

    failed |= check_var::<String>("MAXMIND_LICENSE_KEY");

    failed |= check_var::<u64>("PAYOUTS_BUDGET");

    failed
}
