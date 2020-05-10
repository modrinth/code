#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate tantivy;

use actix_web::{web, web::Data, web::Query, App, HttpRequest, HttpResponse, HttpServer, Responder, get, post};
use handlebars::Handlebars;
use serde_derive::Deserialize;

use tantivy::collector::TopDocs;
use tantivy::query::{QueryParser};
use tantivy::schema::*;
use tantivy::{Index, IndexReader};
use tantivy::ReloadPolicy;
use tempdir::TempDir;

#[derive(Deserialize)]
pub struct SearchRequest {
    q: Option<String>,
}

#[post("search")]
async fn search_post(Query(info): Query<SearchRequest>, reader: Data<IndexReader>, parser: Data<QueryParser<>>, schema: Data<Schema<>>) -> HttpResponse {
    let results = handle_search(Query(info), reader, parser, schema);

    let mut data = "{ \"results\": [".to_owned();

    for result in &results {
        data.push_str(&result);
        data.push_str(",");
    }

    if &results.len() > &(0 as usize) {
        data.pop();
    }

    data.push_str("] }");

    HttpResponse::Ok().body(data)
}

#[get("search")]
async fn search(Query(info): Query<SearchRequest>, hb: Data<Handlebars<'_>>, reader: Data<IndexReader>, parser: Data<QueryParser<>>, schema: Data<Schema<>>) -> HttpResponse {
    let results = handle_search(Query(info), reader, parser, schema);

    let data = json!({
    "results": results,
    });

    let body = hb.render("search", &data).unwrap();

    HttpResponse::Ok().body(body)
}

fn handle_search(Query(info): Query<SearchRequest>, reader: Data<IndexReader>, parser: Data<QueryParser<>>, schema: Data<Schema<>>) -> Vec<String>{
    let mut search_query : String = "".to_string();

    if let Some(q) = info.q {
        search_query = q;
    }

    let searcher = reader.searcher();

    let mut results = vec![];

    if let Ok(query) = parser.parse_query(&search_query) {
        if let Ok(top_docs) = searcher.search(&query, &TopDocs::with_limit(10)) {
            for (_score, doc_address) in top_docs {
                if let Ok(retrieved_doc) = searcher.doc(doc_address) {
                    results.push(schema.to_json(&retrieved_doc));
                }
            }
        }
    }

    return results;
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
async fn main() -> tantivy::Result<()> {
    //Handlebars
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".html", "./static/templates")
        .unwrap();
    let handlebars_ref = web::Data::new(handlebars);

    //Search
    let index_path = TempDir::new("search_index")?;

    let mut schema_builder = Schema::builder();

    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("keywords", TEXT | STORED);
    schema_builder.add_text_field("description", TEXT | STORED);
    schema_builder.add_text_field("body", TEXT);

    let schema = schema_builder.build();
    let schema_ref = web::Data::new(schema.clone());

    let title = schema.get_field("title").unwrap();
    let keywords = schema.get_field("keywords").unwrap();
    let description = schema.get_field("description").unwrap();
    let body = schema.get_field("body").unwrap();

    let site_index = Index::create_in_dir(&index_path, schema.clone())?;
    let mut index_writer = site_index.writer(50_000_000)?;

    index_writer.add_document(doc!(
    title => "Magic",
    keywords => "Magic Fun Adventure",
    description => "A magic mod for magical purposes!",
    body => "A cool magic mod made by your mom :)",
    ));

    index_writer.add_document(doc!(
    title => "Technology",
    keywords => "Technology Fun Adventure",
    description => "A tech mod for tech purposes!",
    body => "A tech mod made by your mom :)",
    ));

    index_writer.commit()?;

    let reader = site_index.reader_builder().reload_policy(ReloadPolicy::OnCommit).try_into()?;
    let reader_ref = web::Data::new(reader);

    let query_parser =  QueryParser::for_index(&site_index, vec![title, body, keywords, description]);
    let query_parser_ref = web::Data::new(query_parser);

    //Init App
    HttpServer::new(move || {
        App::new()
            .app_data(handlebars_ref.clone())
            .app_data(reader_ref.clone())
            .app_data(query_parser_ref.clone())
            .app_data(schema_ref.clone())
            .service(index)
            .service(search)
            .service(search_post)
    })
        .bind("127.0.0.1:8000")?
        .run()
        .await?;

    Ok(())
}

