use std::sync::Arc;
use std::time::Duration;

use actix_web::web;
use database::redis::RedisPool;
use queue::{
    analytics::AnalyticsQueue, email::EmailQueue, payouts::PayoutsQueue,
    session::AuthQueue, socket::ActiveSockets,
};
use tracing::{debug, info, warn};

extern crate clickhouse as clickhouse_crate;
use clickhouse_crate::Client;
use util::cors::default_cors;
use util::gotenberg::GotenbergClient;

use crate::background_task::update_versions;
use crate::database::{PgPool, ReadOnlyPgPool};
use crate::env::ENV;
use crate::queue::billing::{index_billing, index_subscriptions};
use crate::queue::moderation::AutomatedModerationQueue;
use crate::util::anrok;
use crate::util::archon::ArchonClient;
use crate::util::ratelimit::{AsyncRateLimiter, GCRAParameters};
use sync::friends::handle_pubsub;

pub mod auth;
pub mod background_task;
pub mod clickhouse;
pub mod database;
pub mod env;
pub mod file_hosting;
pub mod models;
pub mod queue;
pub mod routes;
pub mod scheduler;
pub mod search;
pub mod sync;
pub mod util;
pub mod validate;

#[cfg(feature = "test")]
pub mod test;

#[derive(Clone)]
pub struct Pepper {
    pub pepper: String,
}

#[derive(Clone)]
pub struct LabrinthConfig {
    pub pool: PgPool,
    pub ro_pool: ReadOnlyPgPool,
    pub redis_pool: RedisPool,
    pub clickhouse: Client,
    pub file_host: Arc<dyn file_hosting::FileHost + Send + Sync>,
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
    pub anrok_client: anrok::Client,
    pub email_queue: web::Data<EmailQueue>,
    pub archon_client: web::Data<ArchonClient>,
    pub gotenberg_client: GotenbergClient,
}

#[allow(clippy::too_many_arguments)]
pub fn app_setup(
    pool: PgPool,
    ro_pool: ReadOnlyPgPool,
    redis_pool: RedisPool,
    search_config: search::SearchConfig,
    clickhouse: &mut Client,
    file_host: Arc<dyn file_hosting::FileHost + Send + Sync>,
    stripe_client: stripe::Client,
    anrok_client: anrok::Client,
    email_queue: EmailQueue,
    gotenberg_client: GotenbergClient,
    enable_background_tasks: bool,
) -> LabrinthConfig {
    info!("Starting labrinth on {}", &ENV.BIND_ADDR);

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

    let scheduler = scheduler::Scheduler::new();

    let limiter = web::Data::new(AsyncRateLimiter::new(
        redis_pool.clone(),
        GCRAParameters::new(300, 300),
    ));

    if enable_background_tasks {
        // The interval in seconds at which the local database is indexed
        // for searching.  Defaults to 1 hour if unset.
        let local_index_interval =
            Duration::from_secs(ENV.LOCAL_INDEX_INTERVAL);
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

        let version_index_interval =
            Duration::from_secs(ENV.VERSION_INDEX_INTERVAL);
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
        let redis_pool_ref = redis_pool.clone();
        scheduler.run(Duration::from_secs(60 * 60 * 6), move || {
            let pool_ref = pool_ref.clone();
            let client_ref = client_ref.clone();
            let redis_ref = redis_pool_ref.clone();
            async move {
                background_task::payouts(pool_ref, client_ref, redis_ref).await;
            }
        });

        let pool_ref = pool.clone();
        let redis_ref = redis_pool.clone();
        let stripe_client_ref = stripe_client.clone();
        let anrok_client_ref = anrok_client.clone();
        actix_rt::spawn(async move {
            loop {
                index_billing(
                    stripe_client_ref.clone(),
                    anrok_client_ref.clone(),
                    pool_ref.clone(),
                    redis_ref.clone(),
                )
                .await;
                tokio::time::sleep(Duration::from_secs(60 * 5)).await;
            }
        });

        let pool_ref = pool.clone();
        let redis_ref = redis_pool.clone();
        let stripe_client_ref = stripe_client.clone();
        let anrok_client_ref = anrok_client.clone();

        actix_rt::spawn(async move {
            loop {
                index_subscriptions(
                    pool_ref.clone(),
                    redis_ref.clone(),
                    stripe_client_ref.clone(),
                    anrok_client_ref.clone(),
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

    let analytics_queue = Arc::new(AnalyticsQueue::new());
    {
        let client_ref = clickhouse.clone();
        let analytics_queue_ref = analytics_queue.clone();
        let pool_ref = pool.clone();
        let redis_ref = redis_pool.clone();
        scheduler.run(Duration::from_secs(15), {
            let redis_ref = redis_ref.clone();
            move || {
                let client_ref = client_ref.clone();
                let analytics_queue_ref = analytics_queue_ref.clone();
                let pool_ref = pool_ref.clone();
                let redis_ref = redis_ref.clone();

                async move {
                    debug!("Indexing analytics queue");
                    let result = analytics_queue_ref
                        .index(client_ref, &redis_ref, &pool_ref)
                        .await;
                    if let Err(e) = result {
                        warn!("Indexing analytics queue failed: {:?}", e);
                    }
                    debug!("Done indexing analytics queue");
                }
            }
        });
    }

    let ip_salt = Pepper {
        pepper: ariadne::ids::Base62Id(ariadne::ids::random_base62(11))
            .to_string(),
    };

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
        ro_pool,
        redis_pool,
        clickhouse: clickhouse.clone(),
        file_host,
        scheduler: Arc::new(scheduler),
        ip_salt,
        search_config,
        session_queue,
        payouts_queue: web::Data::new(PayoutsQueue::new()),
        analytics_queue,
        active_sockets,
        automated_moderation_queue,
        rate_limiter: limiter,
        stripe_client,
        anrok_client,
        gotenberg_client,
        archon_client: web::Data::new(
            ArchonClient::from_env()
                .expect("ARCHON_URL and PYRO_API_KEY must be set"),
        ),
        email_queue: web::Data::new(email_queue),
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
    .app_data(web::Data::new(labrinth_config.ro_pool.clone()))
    .app_data(web::Data::new(labrinth_config.file_host.clone()))
    .app_data(web::Data::new(labrinth_config.search_config.clone()))
    .app_data(web::Data::new(labrinth_config.gotenberg_client.clone()))
    .app_data(labrinth_config.session_queue.clone())
    .app_data(labrinth_config.payouts_queue.clone())
    .app_data(labrinth_config.email_queue.clone())
    .app_data(web::Data::new(labrinth_config.ip_salt.clone()))
    .app_data(web::Data::new(labrinth_config.analytics_queue.clone()))
    .app_data(web::Data::new(labrinth_config.clickhouse.clone()))
    .app_data(labrinth_config.active_sockets.clone())
    .app_data(labrinth_config.automated_moderation_queue.clone())
    .app_data(labrinth_config.archon_client.clone())
    .app_data(web::Data::new(labrinth_config.stripe_client.clone()))
    .app_data(web::Data::new(labrinth_config.anrok_client.clone()))
    .app_data(labrinth_config.rate_limiter.clone())
    .configure(routes::v2::config)
    .configure(routes::v3::config)
    .configure(routes::internal::config)
    .configure(routes::root_config)
    .default_service(web::get().wrap(default_cors()).to(routes::not_found));
}

pub fn utoipa_app_config(
    cfg: &mut utoipa_actix_web::service_config::ServiceConfig,
    _labrinth_config: LabrinthConfig,
) {
    cfg.configure({
        #[cfg(target_os = "linux")]
        {
            |cfg| routes::debug::config(cfg)
        }
        #[cfg(not(target_os = "linux"))]
        {
            |_cfg| ()
        }
    })
    .configure(routes::v3::utoipa_config)
    .configure(routes::internal::utoipa_config);
}
