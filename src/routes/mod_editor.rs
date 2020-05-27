use actix_web::{web, HttpResponse, get};
use handlebars::*;

#[get("modeditor")]
pub async fn mod_editor_get(hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    let data = json!({
        "name": "Handlebars"
    });
    let body = hb.render("mod_editor", &data).unwrap();

    HttpResponse::Ok().body(body)
}