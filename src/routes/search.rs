extern crate diesel;

use actix_web::{web, web::Data, HttpRequest, HttpResponse, get, post};
use handlebars::*;
use meilisearch_sdk::{document::*, indexes::*, client::*, search::*};
use serde::{Serialize, Deserialize};
use actix_web::client;

use crate::database::*;
use diesel::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
struct SearchMod {
    mod_id: i32,
    title: String,
    description: String,
    keywords: Vec<String>,
    versions: Vec<String>,
}

impl Document for SearchMod {
    type UIDType = i32;

    fn get_uid(&self) -> &Self::UIDType {
        &self.mod_id
    }
}

#[derive(Serialize, Deserialize)]
pub struct SearchRequest {
    q: Option<String>,
    f: Option<String>,
    v: Option<String>,
}

#[post("search")]
pub async fn search_post(web::Query(info): web::Query<SearchRequest>, hb: Data<Handlebars<'_>>) -> HttpResponse {
    let results = search(web::Query(info));

    let data = json!({
    "results": results,
    });

    let body = hb.render("search_results", &data).unwrap();

    HttpResponse::Ok().body(body)
}

#[get("search")]
pub async fn search_get(web::Query(info): web::Query<SearchRequest>, hb: Data<Handlebars<'_>>) -> HttpResponse {
    let results = search(web::Query(info));

    let data = json!({
    "results": results,
    });

    let body = hb.render("search", &data).unwrap();

    HttpResponse::Ok().body(body)
}

fn search(web::Query(info): web::Query<SearchRequest>) -> Vec<SearchMod> {
    let client =  Client::new("http://localhost:7700", "");

    let mut search_query = "".to_string();
    let mut filters = "".to_string();


    if let Some(q) = info.q {
        search_query = q;
    }

    if let Some(f) = info.f {
        filters = f;
    }

    if let Some(v) = info.v {
        if filters.is_empty() {
            filters = v;
        }
        else {
            filters = format!("({}) AND {}", filters, v);
        }
    }

    let mut query = Query::new(&search_query).with_limit(10);

    if !filters.is_empty() {
        query = Query::new(&search_query).with_limit(10).with_filters(&filters);
    }

    client.get_index("mods").unwrap().search::<SearchMod>(&query).unwrap().hits
}

pub async fn index_mods(conn : PgConnection) {
    use crate::schema::mods::dsl::*;
    use crate::schema::versions::dsl::*;

    let client =  Client::new("http://localhost:7700", "");
    let mut mods_index = client.get_or_create("mods").unwrap();

    let results = mods.load::<Mod>(&conn).expect("Error loading mods!");
    let mut docs_to_add = vec![];

    for result in results {
        let mod_versions = versions.filter(mod_id.eq(result.id)).load::<Version>(&conn).expect("Error loading versions!");

        let mut mod_game_versions = vec![];

        for version in mod_versions {
            mod_game_versions.extend(version.game_versions.clone())
        }

        docs_to_add.push(SearchMod {
            mod_id: result.id,
            title: result.title,
            description: result.description,
            keywords: result.categories,
            versions: mod_game_versions
        });
    }

    let mut client = client::Client::default();

    let mut response = client.get("https://addons-ecs.forgesvc.net/api/v2/addon/search?categoryId=0&gameId=432&index=0&pageSize=100&sectionId=6&sort=5")
        .header("User-Agent", "Actix-web")
        .header("Content-Type", "application/json")
        .send().await.unwrap();

    println!("{:?}", response);

    let body = response.body().await.unwrap();
    println!("Downloaded: {:?} bytes", body);


    mods_index.add_documents(docs_to_add, Some("mod_id")).unwrap();
}