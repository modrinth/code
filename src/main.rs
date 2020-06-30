use crate::search::indexing::index_mods;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use env_logger::Env;
use log::info;
use std::env;
use std::fs::File;

mod database;
mod models;
mod routes;
mod search;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::from_env(Env::default().default_filter_or("info")).init();
    dotenv::dotenv().ok();

    let client = database::connect().await.unwrap();

    // Get executable path
    let mut exe_path = env::current_exe()?.parent().unwrap().to_path_buf();
    // Create the path to the index lock file
    exe_path.push("index.v1.lock");

    //Indexing mods if not already done
    if env::args().any(|x| x == "regen") {
        // User forced regen of indexing
        info!("Forced regeneration of indexes!");
        index_mods(client).await.unwrap();
    } else if exe_path.exists() {
        // The indexes were not created, or the version was upgraded
        info!("Indexing of mods for first time...");
        index_mods(client).await.unwrap();
        // Create the lock file
        File::create(exe_path)?;
    }

    info!("Starting Actix HTTP server!");

    //Init App
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .service(routes::index_get)
            .service(routes::mod_search)
            .default_service(web::get().to(routes::not_found))
    })
    .bind("127.0.0.1:".to_string() + &dotenv::var("PORT").unwrap())?
    .run()
    .await
}
