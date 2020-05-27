use actix_web::{web, HttpResponse, get};
use handlebars::*;

#[get("/")]
pub async fn index_get(hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    let data = json!({
        "name": "Handlebars"
    });
    let body = hb.render("index", &data).unwrap();

    HttpResponse::Ok().body(body)
}