use actix_web::{App, HttpServer};
use actix_web_prom::PrometheusMetricsBuilder;
use env_logger::Env;
use labrinth::database::redis::RedisPool;
use labrinth::file_hosting::S3Host;
use labrinth::search;
use labrinth::util::ratelimit::RateLimit;
use labrinth::{check_env_vars, clickhouse, database, file_hosting, queue};
use log::{error, info, warn};
use std::sync::Arc;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

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
        error!("某些环境变量丢失！");
    }

    // DSN is from SENTRY_DSN env variable.
    // Has no effect if not set.
    let sentry = sentry::init(sentry::ClientOptions {
        release: sentry::release_name!(),
        traces_sample_rate: 0.1,
        ..Default::default()
    });
    if sentry.is_enabled() {
        info!("启用 Sentry 集成");
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    info!(
        "启动 Labrinth 于 {}",
        dotenvy::var("BIND_ADDR").unwrap()
    );

    // database::check_for_migrations()
    //     .await
    //     .expect("An error occurred while running migrations.");

    // Database Connector
    let pool = database::connect()
        .await
        .expect("数据库连接失败");

    // Redis connector
    info!("初始化 Redis 连接");
    let redis_pool = RedisPool::new(None);

    info!("Redis 连接已建立");
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
                    &dotenvy::var("S3_URL").unwrap(),
                    &dotenvy::var("S3_ACCESS_TOKEN").unwrap(),
                    &dotenvy::var("S3_SECRET").unwrap(),
                )
                    .unwrap(),
            ),
            "local" => Arc::new(file_hosting::MockHost::new()),
            _ => panic!("指定了无效的存储后端。启动中止！"),
        };

    info!("初始化 clickhouse 连接");
    let mut clickhouse = clickhouse::init_client().await.unwrap();
    info!("开始连接 maxmind 数据库");
    let maxmind_reader =
        Arc::new(queue::maxmind::MaxMindIndexer::new().await.unwrap());
    println!("maxmind_reader: 正常");
    let prometheus = PrometheusMetricsBuilder::new("labrinth")
        .endpoint("/metrics")
        .build()
        .expect("创建 prometheus 指标中间件失败");
    println!("prometheus: 正常");
    let search_config = search::SearchConfig::new(None);
    println!("search_config: 正常");
    let labrinth_config = labrinth::app_setup(
        pool.clone(),
        redis_pool.clone(),
        search_config.clone(),
        &mut clickhouse,
        file_host.clone(),
        maxmind_reader.clone(),
    );

    info!("启动 Actix HTTP 服务器！");

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