use actix_web::{get, web, HttpResponse};
use handlebars::*;

#[get("modeditor")]
pub async fn mod_editor_get(hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    let data = json!({
        "name": "Handlebars"
    });
    let body = hb.render("mod-page", &data).unwrap();

    HttpResponse::Ok().body(body)
}
