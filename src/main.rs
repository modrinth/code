#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate diesel;

use actix_web::{web, web::Data, App, HttpRequest, HttpResponse, HttpServer, Responder, get, post};
use handlebars::*;
use meilisearch_sdk::{document::*, indexes::*, client::*, search::*};
use serde::{Serialize, Deserialize};
use actix_files as fs;

mod schema;
mod routes;
mod helpers;
mod database;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    //Handlebars
    let mut handlebars = Handlebars::new();

    handlebars.register_helper("contains", Box::new(helpers::ContainsHelper));
    handlebars
        .register_templates_directory(".hbs", "./templates")
        .unwrap();

    let handlebars_ref = web::Data::new(handlebars);

    let database = database::connect();
    routes::index_mods(database);

    //Init App
    HttpServer::new(move || {
        App::new()
            .app_data(handlebars_ref.clone())
            .service(fs::Files::new("/static", "./static").show_files_listing())
            .service(routes::index_get)
            .service(routes::search_post)
            .service(routes::search_get)
    })
        .bind("127.0.0.1:8000")?
        .run()
        .await
}

