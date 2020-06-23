#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate bson;

#[macro_use]
extern crate log;

use actix_files as fs;
use actix_web::{web, App, HttpServer};
use handlebars::*;
use actix_web::middleware::Logger;
use env_logger::Env;
use std::env;

mod database;
mod helpers;
mod routes;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::from_env(Env::default().default_filter_or("info")).init();
    dotenv::dotenv().ok();

    //Handlebars
    let mut handlebars = Handlebars::new();

    helpers::register_helpers(&mut handlebars);
    handlebars
        .register_templates_directory(".hbs", "./templates")
        .unwrap();

    let handlebars_ref = web::Data::new(handlebars);

    let client = database::connect().await.unwrap();
    routes::index_mods(client).await.unwrap();

    info!("Starting Actix HTTP server!");

    //Init App
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .app_data(handlebars_ref.clone())
            .service(fs::Files::new("/static", "./static").show_files_listing())
            .service(routes::index_get)
            .service(routes::search_post)
            .service(routes::search_get)
            .service(routes::mod_page_get)
            .service(routes::mod_create_get)
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
