//! Refresh token route
use super::stages;
use crate::parse_var;
use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct Body {
    refresh_token: String,
}

#[post("refresh")]
pub async fn route(body: web::Json<Body>) -> Result<HttpResponse, super::Error> {
    let public_url = parse_var::<String>("SELF_ADDR").unwrap_or(format!(
        "http://{}",
        parse_var::<String>("BIND_ADDR").unwrap()
    ));
    let client_id = parse_var::<String>("MICROSOFT_CLIENT_ID").unwrap();
    let client_secret = parse_var::<String>("MICROSOFT_CLIENT_SECRET").unwrap();

    let access_token = stages::access_token::refresh_token(
        &public_url,
        &body.refresh_token,
        &client_id,
        &client_secret,
    )
    .await
    .map_err(|_| super::Error {
        code: StatusCode::INTERNAL_SERVER_ERROR,
        reason: "Error with OAuth token exchange".to_string(),
    })?;

    let stages::xbl_signin::XBLLogin {
        token: xbl_token,
        uhs,
    } = stages::xbl_signin::login_xbl(&access_token.access_token)
        .await
        .map_err(|_| super::Error {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            reason: "Error with XBox Live token exchange".to_string(),
        })?;

    let xsts_response = stages::xsts_token::fetch_token(&xbl_token)
        .await
        .map_err(|_| super::Error {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            reason: "Error with XSTS token exchange".to_string(),
        })?;

    match xsts_response {
        stages::xsts_token::XSTSResponse::Unauthorized(err) => Err(super::Error {
            code: StatusCode::UNAUTHORIZED,
            reason: format!("Error getting XBox Live token: {err}"),
        }),
        stages::xsts_token::XSTSResponse::Success { token: xsts_token } => {
            let bearer_token = stages::bearer_token::fetch_bearer(&xsts_token, &uhs)
                .await
                .map_err(|_| super::Error {
                    code: StatusCode::INTERNAL_SERVER_ERROR,
                    reason: "Error with Bearer token flow".to_string(),
                })?;

            Ok(HttpResponse::Ok().json(&json!({
                "token": bearer_token,
                "refresh_token": &access_token.refresh_token,
                "expires_after": 86400
            })))
        }
    }
}
