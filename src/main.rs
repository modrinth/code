use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{http, web, App, HttpServer};
use env_logger::Env;
use gumdrop::Options;
use log::{info, warn};
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
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::from_env(Env::default().default_filter_or("info")).init();

    let config = Config::parse_args_default_or_exit();

    check_env_vars();

    if config.reset_indices {
        info!("Resetting indices");
        search::indexing::reset_indices().await.unwrap();
        return Ok(());
    } else if config.reconfigure_indices {
        info!("Reconfiguring indices");
        search::indexing::reconfigure_indices().await.unwrap();
        return Ok(());
    }

    // Allow manually skipping the initial indexing for quicker iteration
    // and startup times.
    let skip_initial = config.skip_first_index;
    if skip_initial {
        info!("Skipping initial indexing");
    }

    database::check_for_migrations()
        .await
        .expect("An error occurred while running migrations.");

    // Database Connector
    let pool = database::connect()
        .await
        .expect("Database connection failed");

    let backblaze_enabled = dotenv::var("BACKBLAZE_ENABLED")
        .ok()
        .and_then(|s| s.parse::<bool>().ok())
        .unwrap_or(false);

    let file_host: Arc<dyn file_hosting::FileHost + Send + Sync> = if backblaze_enabled {
        Arc::new(
            file_hosting::BackblazeHost::new(
                &dotenv::var("BACKBLAZE_KEY_ID").unwrap(),
                &dotenv::var("BACKBLAZE_KEY").unwrap(),
                &dotenv::var("BACKBLAZE_BUCKET_ID").unwrap(),
            )
            .await,
        )
    } else {
        Arc::new(file_hosting::MockHost::new())
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
    let mut skip = skip_initial;
    scheduler.run(local_index_interval, move || {
        let pool_ref = pool_ref.clone();
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
            let result = index_mods(pool_ref, settings).await;
            if let Err(e) = result {
                warn!("Local mod indexing failed: {:?}", e);
            }
            info!("Done indexing local database");
        }
    });

    let indexing_queue = Arc::new(search::indexing::queue::CreationQueue::new());

    let queue_ref = indexing_queue.clone();
    let mut skip = skip_initial;
    scheduler.run(std::time::Duration::from_secs(15 * 60), move || {
        let queue = queue_ref.clone();
        let local_skip = skip;
        if skip {
            skip = false;
        }
        async move {
            if local_skip {
                return;
            }
            info!("Indexing created mod queue");
            let result = search::indexing::queue::index_queue(&*queue).await;
            if let Err(e) = result {
                warn!("Indexing created mods failed: {:?}", e);
            }
            info!("Done indexing created mod queue");
        }
    });

    if dotenv::var("INDEX_CURSEFORGE")
        .ok()
        .and_then(|b| b.parse::<bool>().ok())
        .unwrap_or(false)
    {
        // The interval in seconds at which curseforge is indexed for
        // searching.  Defaults to 4 hours if unset.
        let external_index_interval = std::time::Duration::from_secs(
            dotenv::var("EXTERNAL_INDEX_INTERVAL")
                .ok()
                .map(|i| i.parse().unwrap())
                .unwrap_or(3600 * 12),
        );

        let pool_ref = pool.clone();
        scheduler.run(external_index_interval, move || {
            info!("Indexing curseforge");
            let pool_ref = pool_ref.clone();
            async move {
                let settings = IndexingSettings {
                    index_local: false,
                    index_external: true,
                };
                let result = index_mods(pool_ref, settings).await;
                if let Err(e) = result {
                    warn!("External mod indexing failed: {:?}", e);
                }
                info!("Done indexing curseforge");
            }
        });
    }

    let allowed_origins = dotenv::var("CORS_ORIGINS")
        .ok()
        .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
        .unwrap_or_else(|| vec![String::from("http://localhost")]);

    info!("Starting Actix HTTP server!");

    // Init App
    HttpServer::new(move || {
        let mut cors = Cors::new()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);
        for allowed_origin in &allowed_origins {
            cors = cors.allowed_origin(allowed_origin);
        }

        App::new()
            .wrap(cors.finish())
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .data(pool.clone())
            .data(file_host.clone())
            .data(indexing_queue.clone())
            .service(routes::index_get)
            .service(
                web::scope("/api/v1/")
                    .configure(routes::auth_config)
                    .configure(routes::tags_config)
                    .configure(routes::mods_config)
                    .configure(routes::users_config),
            )
            .default_service(web::get().to(routes::not_found))
    })
    .bind(dotenv::var("BIND_ADDR").unwrap())?
    .run()
    .await
}

// This is so that env vars not used immediately don't panic at runtime
fn check_env_vars() {
    fn check_var<T: std::str::FromStr>(var: &str) {
        if dotenv::var(var)
            .ok()
            .and_then(|s| s.parse::<T>().ok())
            .is_none()
        {
            warn!(
                "Variable `{}` missing in dotenv or not of type `{}`",
                var,
                std::any::type_name::<T>()
            )
        }
    }

    if dotenv::var("CORS_ORIGINS")
        .ok()
        .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
        .is_none()
    {
        warn!("Variable `CORS_ORIGINS` missing in dotenv or not a json array of strings");
    }

    check_var::<String>("CDN_URL");
    check_var::<String>("DATABASE_URL");
    check_var::<String>("MEILISEARCH_ADDR");
    check_var::<String>("BIND_ADDR");

    if dotenv::var("BACKBLAZE_ENABLED")
        .ok()
        .and_then(|s| s.parse::<bool>().ok())
        .unwrap_or(false)
    {
        check_var::<String>("BACKBLAZE_KEY_ID");
        check_var::<String>("BACKBLAZE_KEY");
        check_var::<String>("BACKBLAZE_BUCKET_ID");
    } else {
        check_var::<String>("MOCK_FILE_PATH");
    }

    check_var::<bool>("INDEX_CURSEFORGE");
    if dotenv::var("INDEX_CURSEFORGE")
        .ok()
        .and_then(|s| s.parse::<bool>().ok())
        .unwrap_or(false)
    {
        check_var::<usize>("EXTERNAL_INDEX_INTERVAL");
        check_var::<usize>("MAX_CURSEFORGE_ID");
    }

    check_var::<usize>("LOCAL_INDEX_INTERVAL");

    check_var::<String>("GITHUB_CLIENT_ID");
    check_var::<String>("GITHUB_CLIENT_SECRET");
}
