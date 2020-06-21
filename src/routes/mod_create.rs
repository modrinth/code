use actix_web::{get, web, HttpResponse};
use handlebars::*;

#[get("createmod")]
pub async fn mod_create_get(hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    let data = json!({
        "name": "Handlebars"
    });
    let body = hb.render("mod-create", &data).unwrap();

    HttpResponse::Ok().body(body)
}