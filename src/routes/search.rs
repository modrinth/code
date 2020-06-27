use actix_web::{get, post, web, web::Data, HttpResponse};
use handlebars::*;
use meilisearch_sdk::{client::*, document::*, search::*};
use serde::{Deserialize, Serialize};
use meilisearch_sdk::settings::Settings;
use futures::stream::StreamExt;
use bson::Bson;
use std::collections::{HashMap, VecDeque};
use std::error::Error;

use crate::database::*;
use futures_timer::Delay;
use futures::TryFutureExt;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Attachment {
    url: String,
    is_default: bool,
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
#[serde(rename_all = "camelCase")]
struct CurseVersion {
    game_version: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CurseForgeMod {
    id: i32,
    name: String,
    authors: Vec<Author>,
    attachments: Vec<Attachment>,
    website_url: String,
    summary: String,
    download_count: f32,
    categories: Vec<Category>,
    game_version_latest_files: Vec<CurseVersion>,
    date_created: String,
    date_modified: String,
    game_slug: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    date_created: String,
    created: i64,
    date_modified: String,
    updated: i64,
    latest_version: String,
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
    #[serde(rename = "q")]
    query: Option<String>,
    #[serde(rename = "f")]
    filters: Option<String>,
    #[serde(rename = "v")]
    version: Option<String>,
    #[serde(rename = "o")]
    offset: Option<String>,
    #[serde(rename = "s")]
    index: Option<String>,
}

#[post("search")]
pub async fn search_post(
    web::Query(info): web::Query<SearchRequest>,
    hb: Data<Handlebars<'_>>,
) -> HttpResponse {
    let results = search(&info);
    let data = json!({
        "query": info,
        "results": results,
    });
    let body = hb.render("search-results", &data).unwrap();

    HttpResponse::Ok().body(body)
}

#[get("search")]
pub async fn search_get(
    web::Query(info): web::Query<SearchRequest>,
    hb: Data<Handlebars<'_>>,
) -> HttpResponse {
    let results = search(&info);

    let data = json!({
        "query": info,
        "results": results,
    });

    let body = hb.render("search", &data).unwrap();

    HttpResponse::Ok().body(body)
}

fn search(info: &SearchRequest) -> Vec<SearchMod> {
    let client = Client::new("http://localhost:7700", "");

    let search_query: &str;
    let mut filters = String::new();
    let mut offset = 0;
    let mut index = "relevance";

    match info.query.as_ref() {
        Some(q) => search_query = q,
        None => search_query = "{}{}{}",
    }

    if let Some(f) = info.filters.as_ref() {
        filters = f.clone();
    }

    if let Some(v) = info.version.as_ref() {
        if filters.is_empty() {
            filters = v.clone();
        } else {
            filters = format!("({}) AND ({})", filters, v);
        }
    }

    if let Some(o) = info.offset.as_ref() {
        offset = o.parse().unwrap();
    }

    if let Some(s) = info.index.as_ref() {
        index = s;
    }

    let mut query = Query::new(search_query).with_limit(10).with_offset(offset);

    if !filters.is_empty() {
        query = query.with_filters(&filters);
    }

    client.get_index(format!("{}_mods", index).as_ref()).unwrap()
        .search::<SearchMod>(&query).unwrap().hits
}

/*
TODO This method needs a lot of refactoring. Here's a list of changes that need to be made:
 - Move Curseforge Indexing to another method/module
 - Get rid of the 4 indexes (when MeiliSearch updates) and replace it with different rules
 - Remove code fragment duplicates
 */

pub async fn index_mods(db: mongodb::Client) -> Result<(), Box<dyn Error>>{
    let mut docs_to_add: Vec<SearchMod> = vec![];

    docs_to_add.append(&mut index_database(db.clone()).await?);
    //docs_to_add.append(&mut index_curseforge(1, 400000).await?);

    //Write Indexes
    //Relevance Index
    let client = Client::new("http://localhost:7700", "");

    let mut relevance_index = client.get_or_create("relevance_mods").unwrap();

    let mut relevance_rules = default_rules();
    relevance_rules.push_back("desc(downloads)".to_string());

    relevance_index.set_settings(&default_settings().with_ranking_rules(relevance_rules.into())).unwrap();
    relevance_index.add_documents(docs_to_add.clone(), Some("mod_id")).unwrap();

    //Downloads Index
    let mut downloads_index = client.get_or_create("downloads_mods").unwrap();

    let mut downloads_rules = default_rules();
    downloads_rules.push_front("desc(downloads)".to_string());

    downloads_index.set_settings(&default_settings().with_ranking_rules(downloads_rules.into())).unwrap();
    downloads_index.add_documents(docs_to_add.clone(), Some("mod_id")).unwrap();

    //Updated Index
    let mut updated_index = client.get_or_create("updated_mods").unwrap();

    let mut updated_rules = default_rules();
    updated_rules.push_front("desc(updated)".to_string());

    updated_index.set_settings(&default_settings().with_ranking_rules(updated_rules.into())).unwrap();
    updated_index.add_documents(docs_to_add.clone(), Some("mod_id")).unwrap();

    //Created Index
    let mut newest_index = client.get_or_create("newest_mods").unwrap();

    let mut newest_rules = default_rules();
    newest_rules.push_back("desc(created)".to_string());

    newest_index.set_settings(&default_settings().with_ranking_rules(newest_rules.into())).unwrap();
    newest_index.add_documents(docs_to_add.clone(), Some("mod_id")).unwrap();

    Ok(())
}

async fn index_database(client: mongodb::Client) -> Result<Vec<SearchMod>,  Box<dyn Error>> {
    info!("Indexing database mods!");

    let mut docs_to_add: Vec<SearchMod> = vec![];

    let db = client.database("fabricate");

    let mods = db.collection("mods");
    let versions = db.collection("versions");

    let mut results = mods.find(None, None).await?;

    while let Some(unparsed_result) = results.next().await {
        let result : Mod = bson::from_bson(Bson::from(unparsed_result?))?;

        let mut mod_versions = versions.find(doc!{ "mod_id": result.id}, None).await?;

        let mut mod_game_versions = vec![];

        while let Some(unparsed_version) = mod_versions.next().await {
            let mut version : Version = bson::from_bson(Bson::from(unparsed_version?))?;
            mod_game_versions.append(&mut version.game_versions);
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
            icon_url: result.icon_path,
            author_url: "".to_string(),
            date_created: "".to_string(),
            created: 0,
            date_modified: "".to_string(),
            updated: 0,
            latest_version: "".to_string(),
            empty: String::from("{}{}{}"),
        });
    }
    
    Ok(docs_to_add)
}

async fn index_curseforge(start_index: i32, end_index: i32) ->  Result<Vec<SearchMod>,  Box<dyn Error>>{
    info!("Indexing curseforge mods!");

    let mut docs_to_add: Vec<SearchMod> = vec![];

    let res = reqwest::Client::new().post("https://addons-ecs.forgesvc.net/api/v2/addon")
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(format!("{:?}", (start_index..end_index).collect::<Vec<_>>()))
        .send().await?;

    let text = &res.text().await?;
    let curseforge_mods : Vec<CurseForgeMod> = serde_json::from_str(text)?;

    let mut max_index = 0;

    for curseforge_mod in curseforge_mods {
        max_index = curseforge_mod.id;
        if curseforge_mod.game_slug != "minecraft" || !curseforge_mod.website_url.contains("/mc-mods/") { continue; }

        let mut mod_game_versions = vec![];

        let mut using_forge = false;
        let mut using_fabric = false;

        for version in curseforge_mod.game_version_latest_files {
            let version_number: String = version
                .game_version
                .chars()
                .skip(2)
                .take(version.game_version.len())
                .collect();

            if version_number.parse::<f32>()? < 14.0 {
                using_forge = true;
            }

            mod_game_versions.push(version.game_version);
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
                "Energy, Fluid, and Item Transport" => { mod_categories.push(String::from("technology")) }
                "Food" => mod_categories.push(String::from("food")),
                "Farming" => mod_categories.push(String::from("food")),
                "Energy" => mod_categories.push(String::from("technology")),
                "Redstone" => mod_categories.push(String::from("technology")),
                "Genetics" => mod_categories.push(String::from("technology")),
                "Magic" => mod_categories.push(String::from("magic")),
                "Storage" => mod_categories.push(String::from("storage")),
                "API and Library" => mod_categories.push(String::from("library")),
                "Adventure and RPG" => mod_categories.push(String::from("adventure")),
                "Map and Information" => mod_categories.push(String::from("utility")),
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
                "Armor, Tools, and Weapons" => mod_categories.push(String::from("equipment")),
                "Server Utility" => mod_categories.push(String::from("utility")),
                "Fabric" => mod_categories.push(String::from("fabric")),
                _ => {}
            }
        }

        if mod_categories.contains(&"fabric".to_owned()) {
            using_fabric = true;
        }

        mod_categories.sort();
        mod_categories.dedup();
        mod_categories.truncate(3);

        if using_forge {
            mod_categories.push(String::from("forge"));
        }
        if using_fabric {
            mod_categories.push(String::from("fabric"));
        }

        let mut mod_attachments = curseforge_mod.attachments;
        mod_attachments.retain(|x| x.is_default);

        if mod_attachments.is_empty() {
            mod_attachments.push(Attachment {
                url: "".to_string(),
                is_default: true,
            })
        }

        let latest_version = if !mod_game_versions.is_empty() {
            mod_game_versions[0].to_string()
        } else {
            "None".to_string()
        };

        docs_to_add.push(SearchMod {
            mod_id: -curseforge_mod.id,
            author: (&curseforge_mod.authors[0].name).to_string(),
            title: curseforge_mod.name,
            description: curseforge_mod.summary.chars().take(150).collect(),
            keywords: mod_categories,
            versions: mod_game_versions.clone(),
            downloads: curseforge_mod.download_count as i32,
            page_url: curseforge_mod.website_url,
            icon_url: (mod_attachments[0].url).to_string(),
            author_url: (&curseforge_mod.authors[0].url).to_string(),
            date_created: curseforge_mod.date_created.chars().take(10).collect(),
            created: curseforge_mod.date_created.chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse()?,
            date_modified: curseforge_mod.date_modified.chars().take(10).collect(),
            updated: curseforge_mod.date_modified.chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse()?,
            latest_version,
            empty: String::from("{}{}{}"),
        })
    }

    //TODO Reindex every hour for new mods.
    Ok(docs_to_add)
}

fn default_rules() -> VecDeque<String> {
    vec![
        "typo".to_string(),
        "words".to_string(),
        "proximity".to_string(),
        "attribute".to_string(),
        "wordsPosition".to_string(),
        "exactness".to_string(),
    ].into()
}

fn default_settings() -> Settings {
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
        "date_created".to_string(),
        "created".to_string(),
        "date_modified".to_string(),
        "updated".to_string(),
        "latest_version".to_string(),
        "empty".to_string(),
    ];

    let searchable_attributes = vec![
        "title".to_string(),
        "description".to_string(),
        "keywords".to_string(),
        "versions".to_string(),
        "author".to_string(),
        "empty".to_string(),
    ];

    Settings::new()
        .with_displayed_attributes(displayed_attributes.clone())
        .with_searchable_attributes(searchable_attributes.clone())
        .with_accept_new_fields(true)
        .with_stop_words(vec![])
        .with_synonyms(HashMap::new())
}
