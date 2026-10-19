use crate::env::ENV;
use crate::routes::ApiError;
use crate::util::error::Context as _;
use actix_web::HttpRequest;
use serde::Deserialize;
use std::collections::HashMap;

pub async fn check_turnstile(
    req: &HttpRequest,
    challenge: &str,
) -> Result<bool, ApiError> {
    let secret = &ENV.TURNSTILE_SECRET;

    if secret.is_empty() || secret == "none" {
        tracing::info!("Turnstile secret not set, skipping check");
        return Ok(true);
    }

    let conn_info = req.connection_info().clone();
    let ip_addr = if ENV.CLOUDFLARE_INTEGRATION {
        if let Some(header) = req.headers().get("CF-Connecting-IP") {
            header.to_str().ok()
        } else {
            conn_info.peer_addr()
        }
    } else {
        conn_info.peer_addr()
    };

    let ip_addr = ip_addr.wrap_request_err("captcha validation failed")?;

    let client = reqwest::Client::new();

    #[derive(Deserialize)]
    struct Response {
        success: bool,
    }

    let mut form = HashMap::new();

    form.insert("response", challenge);
    form.insert("secret", secret);
    form.insert("remoteip", ip_addr);

    let val: Response = client
        .post("https://challenges.cloudflare.com/turnstile/v0/siteverify")
        .form(&form)
        .send()
        .await
        .wrap_request_err("captcha validation failed")?
        .json()
        .await
        .map_err(|err| eyre::eyre!(err))
        .wrap_request_err("captcha validation failed")?;

    Ok(val.success)
}
