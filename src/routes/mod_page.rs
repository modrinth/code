use actix_web::{get, web, HttpResponse};
use handlebars::*;

#[get("mod/testmod")]
pub async fn mod_page_get(hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    let data = json!({
        "name": "Handlebars"
    });
    let body = hb.render("mod-page", &data).unwrap();

    HttpResponse::Ok().body(body)
}
