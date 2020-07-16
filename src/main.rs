use crate::search::indexing::index_mods;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use env_logger::Env;
use log::info;
use std::env;
use std::fs::File;

mod database;
mod file_hosting;
mod models;
mod routes;
mod search;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::from_env(Env::default().default_filter_or("info")).init();
    dotenv::dotenv().ok();

    check_env_vars();

    //Database Connecter
    let client = database::connect()
        .await
        .expect("Database connection failed");
    let client_ref = web::Data::new(client.clone());

    //File Hosting Initializer
    let authorization_data = file_hosting::authorize_account(
        dotenv::var("BACKBLAZE_KEY_ID").unwrap(),
        dotenv::var("BACKBLAZE_KEY").unwrap(),
    )
    .await
    .unwrap();
    let upload_url_data = file_hosting::get_upload_url(
        authorization_data.clone(),
        dotenv::var("BACKBLAZE_BUCKET_ID").unwrap(),
    )
    .await
    .unwrap();

    let authorization_data_ref = web::Data::new(authorization_data);
    let upload_url_data_ref = web::Data::new(upload_url_data);

    // Get executable path
    let mut exe_path = env::current_exe()?.parent().unwrap().to_path_buf();
    // Create the path to the index lock file
    exe_path.push("index.v1.lock");

    //Indexing mods if not already done
    if env::args().any(|x| x == "regen") {
        // User forced regen of indexing
        info!("Forced regeneration of indexes!");
        index_mods(client).await.expect("Mod indexing failed");
    } else if !exe_path.exists() {
        // The indexes were not created, or the version was upgraded
        info!("Indexing of mods for first time...");
        index_mods(client).await.expect("Mod indexing failed");
        // Create the lock file
        File::create(exe_path)?;
    }

    info!("Starting Actix HTTP server!");

    //Init App
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .data(client_ref.clone())
            .data(authorization_data_ref.clone())
            .data(upload_url_data_ref.clone())
            .service(routes::index_get)
            .service(routes::mod_search)
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
            log::warn!(
                "Variable `{}` missing in dotenv or not of type `{}`",
                var,
                std::any::type_name::<T>()
            )
        }
    }
    check_var::<bool>("INDEX_CURSEFORGE");
    check_var::<String>("MONGODB_ADDR");
    check_var::<String>("MEILISEARCH_ADDR");
    check_var::<String>("BIND_ADDR");

    check_var::<String>("BACKBLAZE_KEY_ID");
    check_var::<String>("BACKBLAZE_KEY");
    check_var::<String>("BACKBLAZE_BUCKET_ID");
}
