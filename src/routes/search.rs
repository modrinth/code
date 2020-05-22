extern crate diesel;

use actix_web::{web, web::Data, HttpRequest, HttpResponse, get, post};
use handlebars::*;
use meilisearch_sdk::{document::*, indexes::*, client::*, search::*};
use serde::{Serialize, Deserialize};
use actix_web::client;

use crate::database::*;
use diesel::prelude::*;
use actix_web::client::Connector;
use meilisearch_sdk::settings::Settings;
use meilisearch_sdk::progress::SettingsUpdate;

#[derive(Serialize, Deserialize, Debug)]
struct Attachment {
    url: String,
    isDefault: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct Category {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Author {
    name: String,
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct CurseVersion {
    gameVersion: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct CurseForgeMod {
    id: i32,
    name: String,
    authors: Vec<Author>,
    attachments: Vec<Attachment>,
    websiteUrl: String,
    summary: String,
    downloadCount: f32,
    categories: Vec<Category>,
    gameVersionLatestFiles: Vec<CurseVersion>
}

#[derive(Serialize, Deserialize, Debug)]
struct SearchMod {
    mod_id: i32,
    author: String,
    title: String,
    description: String,
    keywords: Vec<String>,
    versions: Vec<String>,
    downloads: i32,
    page_url: String,
    icon_url: String,
    author_url: String,
    empty: String,
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

    match info.q {
        Some(q) => search_query = q,
        None => search_query = "empty".to_string()
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
    let mut docs_to_add : Vec<SearchMod> = vec![];

    for result in results {
        let mod_versions = versions.filter(mod_id.eq(result.id)).load::<Version>(&conn).expect("Error loading versions!");

        let mut mod_game_versions = vec![];

        for version in mod_versions {
            mod_game_versions.extend(version.game_versions.clone())
        }

        docs_to_add.push(SearchMod {
            mod_id: result.id,
            author: result.author,
            title: result.title,
            description: result.description,
            keywords: result.categories,
            versions: mod_game_versions,
            downloads: result.downloads,
            page_url: "".to_string(),
            icon_url: "".to_string(),
            author_url: "".to_string(),
            empty: String::from("empty")
        });
    }

    let mut client = aws::Client::default();

    let mut response = client.get("https://addons-ecs.forgesvc.net/api/v2/addon/search?categoryId=0&gameId=432&index=0&pageSize=25&sectionId=6&sort=5")
        .header("User-Agent", "Actix-web")
        .send().await.unwrap();


    let body = response.body().await.unwrap();
    println!("{:?}", body.len());

    let curseforge_mods : Vec<CurseForgeMod> = serde_json::from_str(std::str::from_utf8(&body).unwrap()).unwrap();

    for curseforge_mod in curseforge_mods {
        let mut mod_game_versions = vec![];
        for version in curseforge_mod.gameVersionLatestFiles {
            mod_game_versions.push(version.gameVersion);
        }

        let mut mod_categories = vec![];
        for category in curseforge_mod.categories {
            match &category.name[..] {
                "World Gen" => mod_categories.push(String::from("worldgen")),
                "Biomes" => mod_categories.push(String::from("worldgen")),
                "Ores and Resources" => mod_categories.push(String::from("worldgen")),
                "Structures" => mod_categories.push(String::from("worldgen")),
                "Dimensions" => mod_categories.push(String::from("worldgen")),
                "Mobs" => mod_categories.push(String::from("worldgen")),
                "Technology" => mod_categories.push(String::from("technology")),
                "Processing" => mod_categories.push(String::from("technology")),
                "Player Transport" => mod_categories.push(String::from("technology")),
                "Energy, Fluid, and Item Transport" => mod_categories.push(String::from("technology")),
                "Farming" => mod_categories.push(String::from("technology")),
                "Energy" => mod_categories.push(String::from("technology")),
                "Redstone" => mod_categories.push(String::from("technology")),
                "Genetics" => mod_categories.push(String::from("technology")),
                "Magic" => mod_categories.push(String::from("magic")),
                "Storage" => mod_categories.push(String::from("technology")),
                "API and Library" => mod_categories.push(String::from("library")),
                "Adventure and RPG" => mod_categories.push(String::from("adventure")),
                "Map and Information" => mod_categories.push(String::from("adventure")),
                "Cosmetic" => mod_categories.push(String::from("decoration")),
                "Addons" => mod_categories.push(String::from("misc")),
                "Thermal Expansion" => mod_categories.push(String::from("misc")),
                "Tinker's Construct" => mod_categories.push(String::from("misc")),
                "Industrial Craft" => mod_categories.push(String::from("misc")),
                "Thaumcraft" => mod_categories.push(String::from("misc")),
                "Buildcraft" => mod_categories.push(String::from("misc")),
                "Forestry" => mod_categories.push(String::from("misc")),
                "Blood Magic" => mod_categories.push(String::from("misc")),
                "Lucky Blocks" => mod_categories.push(String::from("misc")),
                "Applied Energistics 2" => mod_categories.push(String::from("misc")),
                "CraftTweaker" => mod_categories.push(String::from("misc")),
                "Miscellaneous" => mod_categories.push(String::from("misc")),
                "Armor, Tools, and Weapons" => mod_categories.push(String::from("technology")),
                "Server Utility" => mod_categories.push(String::from("utility")),
                "Fabric" => mod_categories.push(String::from("fabric")),
                _ => {}
            }
        }

        let mut mod_attachments = curseforge_mod.attachments;
        mod_attachments.retain(|x| x.isDefault);
        docs_to_add.push(SearchMod {
            mod_id: curseforge_mod.id,
            author: (&curseforge_mod.authors[0].name).to_string(),
            title: curseforge_mod.name,
            description: curseforge_mod.summary,
            keywords: mod_categories,
            versions: mod_game_versions,
            downloads: curseforge_mod.downloadCount as i32,
            page_url: curseforge_mod.websiteUrl,
            icon_url: (mod_attachments[0].url).to_string(),
            author_url: (&curseforge_mod.authors[0].url).to_string(),
            empty: String::from("empty")
        })
    }

    println!("{:?}", docs_to_add);
    mods_index.add_documents(docs_to_add, Some("mod_id")).unwrap();

    //Write Settings
    let settings = mods_index.get_settings().unwrap();

    let mut ranking_rules = vec![
        "typo".to_string(),
        "words".to_string(),
        "proximity".to_string(),
        "attribute".to_string(),
        "wordsPosition".to_string(),
        "exactness".to_string(),
        "desc(downloads)".to_string()
    ];

    let displayed_attributes = vec![
        "mod_id".to_string(),
        "author".to_string(),
        "title".to_string(),
        "description".to_string(),
        "keywords".to_string(),
        "versions".to_string(),
        "downloads".to_string(),
        "page_url".to_string(),
        "icon_url".to_string(),
        "author_url".to_string(),
        "empty".to_string(),
    ];

    let searchable_attributes = vec![
        "author".to_string(),
        "title".to_string(),
        "description".to_string(),
        "keywords".to_string(),
        "versions".to_string(),
        "empty".to_string(),
    ];

    let write_settings = Settings::new()
        .with_displayed_attributes(displayed_attributes)
        .with_searchable_attributes(searchable_attributes)
        .with_accept_new_fields(settings.accept_new_fields.unwrap())
        .with_stop_words(settings.stop_words.unwrap())
        .with_synonyms(settings.synonyms.unwrap())
        .with_ranking_rules(ranking_rules);

    mods_index.set_settings(&write_settings).unwrap();
}