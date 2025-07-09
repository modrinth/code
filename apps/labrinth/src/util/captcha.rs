use crate::routes::ApiError;
use crate::util::env::parse_var;
use actix_web::HttpRequest;
use serde::Deserialize;
use std::collections::HashMap;

pub async fn check_hcaptcha(
    req: &HttpRequest,
    challenge: &str,
) -> Result<bool, ApiError> {
    let secret = dotenvy::var("HCAPTCHA_SECRET")?;

    if secret.is_empty() || secret == "none" {
        tracing::info!("hCaptcha secret not set, skipping check");
        return Ok(true);
    }

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

    let ip_addr = ip_addr.ok_or(ApiError::Turnstile)?;

    let client = reqwest::Client::new();

    #[derive(Deserialize)]
    struct Response {
        success: bool,
    }

    let mut form = HashMap::new();

    form.insert("response", challenge);
    form.insert("secret", &*secret);
    form.insert("remoteip", ip_addr);

    let val: Response = client
        .post("https://api.hcaptcha.com/siteverify")
        .form(&form)
        .send()
        .await
        .map_err(|_| ApiError::Turnstile)?
        .json()
        .await
        .map_err(|_| ApiError::Turnstile)?;

    Ok(val.success)
}
