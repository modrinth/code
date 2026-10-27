use crate::env::ENV;
use crate::routes::ApiError;
use crate::util::error::Context as _;
use actix_web::HttpRequest;
use serde::Deserialize;
use std::collections::HashMap;
use xredis::RedisPool;

const USED_TURNSTILE_NAMESPACE: &str = "used_turnstile:v1";
const TURNSTILE_TOKEN_LIFETIME_SECS: i64 = 300;

pub async fn check_turnstile(
    req: &HttpRequest,
    challenge: &str,
    redis: &RedisPool,
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

    if !val.success {
        return Ok(false);
    }

    mark_token_used(challenge, redis).await
}

/// Siteverify has been observed accepting replayed tokens, so single use is
/// enforced here as well. Returns `false` if the token was already consumed.
async fn mark_token_used(
    challenge: &str,
    redis: &RedisPool,
) -> Result<bool, ApiError> {
    let mut conn = redis
        .connect()
        .await
        .wrap_internal_err("connecting to Redis")?;
    let key =
        conn.key()
            .with_slot(USED_TURNSTILE_NAMESPACE, challenge, challenge);

    let newly_set: Option<String> = redis::cmd("SET")
        .arg(&key)
        .arg("")
        .arg("NX")
        .arg("EX")
        .arg(TURNSTILE_TOKEN_LIFETIME_SECS)
        .query_async(&mut conn)
        .await
        .wrap_internal_err("marking captcha token as used")?;

    Ok(newly_set.is_some())
}
