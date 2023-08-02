//! Login route for Hydra, redirects to the Microsoft login page before going to the redirect route
use crate::{auth::minecraft::stages::login_redirect, auth::templates, parse_var};
use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Query {
    pub id: Option<String>,
}

#[derive(Serialize)]
pub struct AuthorizationInit {
    pub url: String,
}

#[get("init")]
pub async fn route(info: web::Query<Query>) -> Result<HttpResponse, templates::ErrorPage> {
    let conn_id = info.0.id.ok_or_else(|| templates::ErrorPage {
        code: StatusCode::BAD_REQUEST,
        message: "No socket ID provided (open a web socket at the / route for one)".to_string(),
    })?;

    let public_url = parse_var::<String>("SELF_ADDR").unwrap_or(format!(
        "http://{}",
        parse_var::<String>("BIND_ADDR").unwrap()
    ));
    let client_id = parse_var::<String>("MICROSOFT_CLIENT_ID").unwrap();

    let url = login_redirect::get_url(&public_url, &conn_id, &client_id);

    Ok(HttpResponse::TemporaryRedirect()
        .append_header(("Location", &*url))
        .json(AuthorizationInit { url }))
}
