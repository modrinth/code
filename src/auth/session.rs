use crate::auth::AuthenticationError;
use crate::database::models::session_item::{Session, SessionBuilder};
use crate::database::models::UserId;
use crate::util::env::parse_var;
use actix_web::HttpRequest;
use rand::distributions::Alphanumeric;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use woothee::parser::Parser;

pub async fn issue_session(
    req: HttpRequest,
    user_id: UserId,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    redis: &deadpool_redis::Pool,
) -> Result<Session, AuthenticationError> {
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

    let country = req
        .headers()
        .get("cf-ipcountry")
        .and_then(|x| x.to_str().ok());
    let city = req.headers().get("cf-ipcity").and_then(|x| x.to_str().ok());

    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|x| x.to_str().ok())
        .ok_or_else(|| AuthenticationError::InvalidCredentials)?;

    let parser = Parser::new();
    let info = parser.parse(user_agent);
    let os = if let Some(info) = info {
        Some((info.os, info.name))
    } else {
        None
    };

    let session = ChaCha20Rng::from_entropy()
        .sample_iter(&Alphanumeric)
        .take(60)
        .map(char::from)
        .collect::<String>();

    let session = format!("mra_{session}");

    let id = SessionBuilder {
        session,
        user_id,
        os: os.map(|x| x.0.to_string()),
        platform: os.map(|x| x.1.to_string()),
        city: city.map(|x| x.to_string()),
        country: country.map(|x| x.to_string()),
        ip: ip_addr
            .ok_or_else(|| AuthenticationError::InvalidCredentials)?
            .to_string(),
        user_agent: user_agent.to_string(),
    }
    .insert(transaction)
    .await?;

    let session = Session::get_id(id, &mut *transaction, redis)
        .await?
        .ok_or_else(|| AuthenticationError::InvalidCredentials)?;

    Ok(session)
}

// TODO: List user sessions route
// TODO: Delete User Session Route / logout
// TODO: Refresh session route
