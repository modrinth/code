use actix_web::{get, post, web, HttpResponse};
use handlebars::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct CreatedMod {
    name: String,
    description: String,
    body: String,
}

#[get("createmod")]
pub async fn mod_create_get(hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    let data = json!({
        "name": "Handlebars"
    });
    let body = hb.render("mod-create", &data).unwrap();

    HttpResponse::Ok().body(body)
}

#[post("createmod")]
pub async fn mod_create_post(hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    let data = json!({
        "name": "Handlebars"
    });
    let body = hb.render("mod-create", &data).unwrap();

    HttpResponse::Ok().body(body)
}