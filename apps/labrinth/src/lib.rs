use std::sync::Arc;
use std::time::Duration;

use actix_web::web;
use database::redis::RedisPool;
use queue::{
    analytics::AnalyticsQueue, payouts::PayoutsQueue, session::AuthQueue,
    socket::ActiveSockets,
};
use sqlx::Postgres;
use tracing::{info, warn};

extern crate clickhouse as clickhouse_crate;
use clickhouse_crate::Client;
use util::cors::default_cors;

use crate::background_task::update_versions;
use crate::queue::moderation::AutomatedModerationQueue;
use crate::queue::payouts::insert_bank_balances;
use crate::util::env::{parse_strings_from_var, parse_var};
use crate::util::ratelimit::{AsyncRateLimiter, GCRAParameters};
use sync::friends::handle_pubsub;

pub mod auth;
pub mod background_task;
pub mod clickhouse;
pub mod database;
pub mod file_hosting;
pub mod models;
pub mod queue;
pub mod routes;
pub mod scheduler;
pub mod search;
pub mod sync;
pub mod util;
pub mod validate;

#[derive(Clone)]
pub struct Pepper {
    pub pepper: String,
}

#[derive(Clone)]
pub struct LabrinthConfig {
    pub pool: sqlx::Pool<Postgres>,
    pub redis_pool: RedisPool,
    pub clickhouse: Client,
    pub file_host: Arc<dyn file_hosting::FileHost + Send + Sync>,
    pub maxmind: Arc<queue::maxmind::MaxMindIndexer>,
    pub scheduler: Arc<scheduler::Scheduler>,
    pub ip_salt: Pepper,
    pub search_config: search::SearchConfig,
    pub session_queue: web::Data<AuthQueue>,
    pub payouts_queue: web::Data<PayoutsQueue>,
    pub analytics_queue: Arc<AnalyticsQueue>,
    pub active_sockets: web::Data<ActiveSockets>,
    pub automated_moderation_queue: web::Data<AutomatedModerationQueue>,
    pub rate_limiter: web::Data<AsyncRateLimiter>,
    pub stripe_client: stripe::Client,
}

#[allow(clippy::too_many_arguments)]
pub fn app_setup(
    pool: sqlx::Pool<Postgres>,
    redis_pool: RedisPool,
    search_config: search::SearchConfig,
    clickhouse: &mut Client,
    file_host: Arc<dyn file_hosting::FileHost + Send + Sync>,
    maxmind: Arc<queue::maxmind::MaxMindIndexer>,
    stripe_client: stripe::Client,
    enable_background_tasks: bool,
) -> LabrinthConfig {
    info!(
        "Starting Labrinth on {}",
        dotenvy::var("BIND_ADDR").unwrap()
    );

    let automated_moderation_queue =
        web::Data::new(AutomatedModerationQueue::default());

    {
        let automated_moderation_queue_ref = automated_moderation_queue.clone();
        let pool_ref = pool.clone();
        let redis_pool_ref = redis_pool.clone();
        actix_rt::spawn(async move {
            automated_moderation_queue_ref
                .task(pool_ref, redis_pool_ref)
                .await;
        });
    }

    let mut scheduler = scheduler::Scheduler::new();

    let limiter = web::Data::new(AsyncRateLimiter::new(
        redis_pool.clone(),
        GCRAParameters::new(300, 300),
    ));

    if enable_background_tasks {
        // The interval in seconds at which the local database is indexed
        // for searching.  Defaults to 1 hour if unset.
        let local_index_interval = Duration::from_secs(
            parse_var("LOCAL_INDEX_INTERVAL").unwrap_or(3600),
        );
        let pool_ref = pool.clone();
        let search_config_ref = search_config.clone();
        let redis_pool_ref = redis_pool.clone();
        scheduler.run(local_index_interval, move || {
            let pool_ref = pool_ref.clone();
            let redis_pool_ref = redis_pool_ref.clone();
            let search_config_ref = search_config_ref.clone();
            async move {
                background_task::index_search(
                    pool_ref,
                    redis_pool_ref,
                    search_config_ref,
                )
                .await;
            }
        });

        // Changes statuses of scheduled projects/versions
        let pool_ref = pool.clone();
        // TODO: Clear cache when these are run
        scheduler.run(Duration::from_secs(60 * 5), move || {
            let pool_ref = pool_ref.clone();
            async move {
                background_task::release_scheduled(pool_ref).await;
            }
        });

        let version_index_interval = Duration::from_secs(
            parse_var("VERSION_INDEX_INTERVAL").unwrap_or(1800),
        );
        let pool_ref = pool.clone();
        let redis_pool_ref = redis_pool.clone();
        scheduler.run(version_index_interval, move || {
            let pool_ref = pool_ref.clone();
            let redis = redis_pool_ref.clone();
            async move {
                update_versions(pool_ref, redis).await;
            }
        });

        let pool_ref = pool.clone();
        let client_ref = clickhouse.clone();
        scheduler.run(Duration::from_secs(60 * 60 * 6), move || {
            let pool_ref = pool_ref.clone();
            let client_ref = client_ref.clone();
            async move {
                background_task::payouts(pool_ref, client_ref).await;
            }
        });

        let pool_ref = pool.clone();
        let redis_ref = redis_pool.clone();
        let stripe_client_ref = stripe_client.clone();
        actix_rt::spawn(async move {
            loop {
                routes::internal::billing::index_billing(
                    stripe_client_ref.clone(),
                    pool_ref.clone(),
                    redis_ref.clone(),
                )
                .await;
                tokio::time::sleep(Duration::from_secs(60 * 5)).await;
            }
        });

        let pool_ref = pool.clone();
        let redis_ref = redis_pool.clone();
        actix_rt::spawn(async move {
            loop {
                routes::internal::billing::index_subscriptions(
                    pool_ref.clone(),
                    redis_ref.clone(),
                )
                .await;
                tokio::time::sleep(Duration::from_secs(60 * 5)).await;
            }
        });
    }

    let session_queue = web::Data::new(AuthQueue::new());

    let pool_ref = pool.clone();
    let redis_ref = redis_pool.clone();
    let session_queue_ref = session_queue.clone();
    scheduler.run(Duration::from_secs(60 * 30), move || {
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

    let reader = maxmind.clone();
    {
        let reader_ref = reader;
        scheduler.run(Duration::from_secs(60 * 60 * 24), move || {
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
        let pool_ref = pool.clone();
        let redis_ref = redis_pool.clone();
        scheduler.run(Duration::from_secs(15), move || {
            let client_ref = client_ref.clone();
            let analytics_queue_ref = analytics_queue_ref.clone();
            let pool_ref = pool_ref.clone();
            let redis_ref = redis_ref.clone();

            async move {
                info!("Indexing analytics queue");
                let result = analytics_queue_ref
                    .index(client_ref, &redis_ref, &pool_ref)
                    .await;
                if let Err(e) = result {
                    warn!("Indexing analytics queue failed: {:?}", e);
                }
                info!("Done indexing analytics queue");
            }
        });
    }

    let ip_salt = Pepper {
        pepper: ariadne::ids::Base62Id(ariadne::ids::random_base62(11))
            .to_string(),
    };

    let payouts_queue = web::Data::new(PayoutsQueue::new());

    let payouts_queue_ref = payouts_queue.clone();
    let pool_ref = pool.clone();
    scheduler.run(Duration::from_secs(60 * 60 * 6), move || {
        let payouts_queue_ref = payouts_queue_ref.clone();
        let pool_ref = pool_ref.clone();
        async move {
            info!("Started updating bank balances");
            let result =
                insert_bank_balances(&payouts_queue_ref, &pool_ref).await;
            if let Err(e) = result {
                warn!("Bank balance update failed: {:?}", e);
            }
            info!("Done updating bank balances");
        }
    });

    let active_sockets = web::Data::new(ActiveSockets::default());

    {
        let pool = pool.clone();
        let redis_client = redis::Client::open(redis_pool.url.clone()).unwrap();
        let sockets = active_sockets.clone();
        actix_rt::spawn(async move {
            let pubsub = redis_client.get_async_pubsub().await.unwrap();
            handle_pubsub(pubsub, pool, sockets).await;
        });
    }

    LabrinthConfig {
        pool,
        redis_pool,
        clickhouse: clickhouse.clone(),
        file_host,
        maxmind,
        scheduler: Arc::new(scheduler),
        ip_salt,
        search_config,
        session_queue,
        payouts_queue,
        analytics_queue,
        active_sockets,
        automated_moderation_queue,
        rate_limiter: limiter,
        stripe_client,
    }
}

pub fn app_config(
    cfg: &mut web::ServiceConfig,
    labrinth_config: LabrinthConfig,
) {
    cfg.app_data(web::FormConfig::default().error_handler(|err, _req| {
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
    .app_data(web::Data::new(labrinth_config.redis_pool.clone()))
    .app_data(web::Data::new(labrinth_config.pool.clone()))
    .app_data(web::Data::new(labrinth_config.file_host.clone()))
    .app_data(web::Data::new(labrinth_config.search_config.clone()))
    .app_data(labrinth_config.session_queue.clone())
    .app_data(labrinth_config.payouts_queue.clone())
    .app_data(web::Data::new(labrinth_config.ip_salt.clone()))
    .app_data(web::Data::new(labrinth_config.analytics_queue.clone()))
    .app_data(web::Data::new(labrinth_config.clickhouse.clone()))
    .app_data(web::Data::new(labrinth_config.maxmind.clone()))
    .app_data(labrinth_config.active_sockets.clone())
    .app_data(labrinth_config.automated_moderation_queue.clone())
    .app_data(web::Data::new(labrinth_config.stripe_client.clone()))
    .app_data(labrinth_config.rate_limiter.clone())
    .configure({
        #[cfg(target_os = "linux")]
        {
            |cfg| routes::debug::config(cfg)
        }
        #[cfg(not(target_os = "linux"))]
        {
            |_cfg| ()
        }
    })
    .configure(routes::v2::config)
    .configure(routes::v3::config)
    .configure(routes::internal::config)
    .configure(routes::root_config)
    .default_service(web::get().wrap(default_cors()).to(routes::not_found));
}

// This is so that env vars not used immediately don't panic at runtime
pub fn check_env_vars() -> bool {
    let mut failed = false;

    fn check_var<T: std::str::FromStr>(var: &str) -> bool {
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
        Some("s3") => {
            let mut check_var_set = |var_prefix| {
                failed |= check_var::<String>(&format!(
                    "S3_{var_prefix}_BUCKET_NAME"
                ));
                failed |= check_var::<bool>(&format!(
                    "S3_{var_prefix}_USES_PATH_STYLE_BUCKET"
                ));
                failed |=
                    check_var::<String>(&format!("S3_{var_prefix}_REGION"));
                failed |= check_var::<String>(&format!("S3_{var_prefix}_URL"));
                failed |= check_var::<String>(&format!(
                    "S3_{var_prefix}_ACCESS_TOKEN"
                ));
                failed |=
                    check_var::<String>(&format!("S3_{var_prefix}_SECRET"));
            };

            check_var_set("PUBLIC");
            check_var_set("PRIVATE");
        }
        Some("local") => {
            failed |= check_var::<String>("MOCK_FILE_PATH");
        }
        Some(backend) => {
            warn!(
                "Variable `STORAGE_BACKEND` contains an invalid value: {backend}. Expected \"s3\" or \"local\"."
            );
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
        warn!(
            "Variable `WHITELISTED_MODPACK_DOMAINS` missing in dotenv or not a json array of strings"
        );
        failed |= true;
    }

    if parse_strings_from_var("ALLOWED_CALLBACK_URLS").is_none() {
        warn!(
            "Variable `ALLOWED_CALLBACK_URLS` missing in dotenv or not a json array of strings"
        );
        failed |= true;
    }

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

    failed |= check_var::<String>("TREMENDOUS_API_URL");
    failed |= check_var::<String>("TREMENDOUS_API_KEY");
    failed |= check_var::<String>("TREMENDOUS_PRIVATE_KEY");

    failed |= check_var::<String>("PAYPAL_API_URL");
    failed |= check_var::<String>("PAYPAL_WEBHOOK_ID");
    failed |= check_var::<String>("PAYPAL_CLIENT_ID");
    failed |= check_var::<String>("PAYPAL_CLIENT_SECRET");
    failed |= check_var::<String>("PAYPAL_NVP_USERNAME");
    failed |= check_var::<String>("PAYPAL_NVP_PASSWORD");
    failed |= check_var::<String>("PAYPAL_NVP_SIGNATURE");

    failed |= check_var::<String>("HCAPTCHA_SECRET");

    failed |= check_var::<String>("SMTP_USERNAME");
    failed |= check_var::<String>("SMTP_PASSWORD");
    failed |= check_var::<String>("SMTP_HOST");
    failed |= check_var::<u16>("SMTP_PORT");
    failed |= check_var::<String>("SMTP_TLS");

    failed |= check_var::<String>("SITE_VERIFY_EMAIL_PATH");
    failed |= check_var::<String>("SITE_RESET_PASSWORD_PATH");
    failed |= check_var::<String>("SITE_BILLING_PATH");

    failed |= check_var::<String>("SENDY_URL");
    failed |= check_var::<String>("SENDY_LIST_ID");
    failed |= check_var::<String>("SENDY_API_KEY");

    if parse_strings_from_var("ANALYTICS_ALLOWED_ORIGINS").is_none() {
        warn!(
            "Variable `ANALYTICS_ALLOWED_ORIGINS` missing in dotenv or not a json array of strings"
        );
        failed |= true;
    }

    failed |= check_var::<bool>("CLICKHOUSE_REPLICATED");
    failed |= check_var::<String>("CLICKHOUSE_URL");
    failed |= check_var::<String>("CLICKHOUSE_USER");
    failed |= check_var::<String>("CLICKHOUSE_PASSWORD");
    failed |= check_var::<String>("CLICKHOUSE_DATABASE");

    failed |= check_var::<String>("MAXMIND_LICENSE_KEY");

    failed |= check_var::<String>("FLAME_ANVIL_URL");

    failed |= check_var::<String>("STRIPE_API_KEY");
    failed |= check_var::<String>("STRIPE_WEBHOOK_SECRET");

    failed |= check_var::<String>("ADITUDE_API_KEY");

    failed |= check_var::<String>("PYRO_API_KEY");

    failed |= check_var::<String>("BREX_API_URL");
    failed |= check_var::<String>("BREX_API_KEY");

    failed |= check_var::<String>("DELPHI_URL");

    failed |= check_var::<String>("ARCHON_URL");

    failed
}
