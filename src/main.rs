#[macro_use]
extern crate serde_json;

use actix_web::{web, web::Data, App, HttpRequest, HttpResponse, HttpServer, Responder, get, post};
use handlebars::Handlebars;
use meilisearch_sdk::{document::*, indexes::*, client::*, search::*};
use serde::{Serialize, Deserialize};
use actix_files as fs;

#[derive(Serialize, Deserialize, Debug)]
struct Mod {
    mod_id: usize,
    title: String,
    description: String,
}

impl Document for Mod {
    type UIDType = usize;

    fn get_uid(&self) -> &Self::UIDType {
        &self.mod_id
    }
}


#[derive(Deserialize)]
pub struct SearchRequest {
    q: Option<String>,
    f: Option<String>,
}

#[post("search")]
async fn search_post(web::Query(info): web::Query<SearchRequest>) -> HttpResponse {
    let results = search(web::Query(info));

    let data = json!({
    "results": results,
    });

    HttpResponse::Ok().body(data)
}

#[get("search")]
async fn search_get(web::Query(info): web::Query<SearchRequest>, hb: Data<Handlebars<'_>>) -> HttpResponse {
    let results = search(web::Query(info));

    let data = json!({
    "results": results,
    });

    let body = hb.render("search", &data).unwrap();

    HttpResponse::Ok().body(body)
}

fn search(web::Query(info): web::Query<SearchRequest>) -> Vec<Mod> {
    let client =  Client::new("http://localhost:7700", "");

    let mut search_query = "".to_string();
    let mut filters = "".to_string();

    if let Some(q) = info.q {
        search_query = q;
    }

    if let Some(f) = info.f {
        filters = f;
    }

    let mut query = Query::new(&search_query).with_limit(10);

    if !filters.is_empty() {
        query = Query::new(&search_query).with_limit(10).with_filters(&filters);
    }

    client.get_index("mods").unwrap().search::<Mod>(&query).unwrap().hits
}

#[get("/")]
async fn index(hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    let data = json!({
        "name": "Handlebars"
    });
    let body = hb.render("index", &data).unwrap();

    HttpResponse::Ok().body(body)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    //Handlebars
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".hbs", "./templates")
        .unwrap();
    let handlebars_ref = web::Data::new(handlebars);

    //Search

    let client =  Client::new("http://localhost:7700", "");
    let mut mods = client.get_or_create("mods").unwrap();

    mods.add_documents(vec![
        Mod {
            mod_id: 0,
            title: String::from("Magic Mod"),
            description: String::from("An illustrious magic mod for magical wizards"),
        },
        Mod {
            mod_id: 1,
            title: String::from("Tech Mod"),
            description: String::from("An technological mod for complete NERDS"),
        },
        Mod {
            mod_id: 2,
            title: String::from("Hood Mod"),
            description: String::from("A hood mod to roleplay as if you were a real street person. Some adventure stuff too"),
        },
        Mod {
            mod_id: 3,
            title: String::from("Adventure Mod"),
            description: String::from("An epic gamer adventure mod for epic adventure gamers"),
        },
    ], Some("mod_id")).unwrap();

    //Init App
    HttpServer::new(move || {
        App::new()
            .app_data(handlebars_ref.clone())
            .service(fs::Files::new("/static", "./static").show_files_listing())
            .service(index)
            .service(search_get)
            .service(search_post)
    })
        .bind("127.0.0.1:8000")?
        .run()
        .await
}

