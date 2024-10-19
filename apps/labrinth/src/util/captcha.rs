use crate::routes::ApiError;
use crate::util::env::parse_var;
use actix_web::HttpRequest;
use serde::Deserialize;
use serde_json::json;

pub async fn check_turnstile_captcha(
    req: &HttpRequest,
    challenge: &str,
) -> Result<bool, ApiError> {
    let conn_info = req.connection_info().clone();
    let ip_addr = if parse_var("CLOUDFLARE_INTEGRATION").unwrap_or(false) {
        if let Some(header) = req.headers().get("CF-Connecting-IP") {
            header.to_str().ok()
        } else {
            conn_info.peer_addr()
        }
    } else {
        conn_info.peer_addr()
    };

    let client = reqwest::Client::new();

    #[derive(Deserialize)]
    struct Response {
        success: bool,
    }

    let val: Response = client
        .post("https://challenges.cloudflare.com/turnstile/v0/siteverify")
        .json(&json!({
            "secret": dotenvy::var("TURNSTILE_SECRET")?,
            "response": challenge,
            "remoteip": ip_addr,
        }))
        .send()
        .await
        .map_err(|_| ApiError::Turnstile)?
        .json()
        .await
        .map_err(|_| ApiError::Turnstile)?;

    Ok(val.success)
}
