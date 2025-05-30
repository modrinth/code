use crate::auth::{AuthenticationError, get_user_from_headers};
use crate::database::models::DBUserId;
use crate::database::models::session_item::DBSession;
use crate::database::models::session_item::SessionBuilder;
use crate::database::redis::RedisPool;
use crate::models::pats::Scopes;
use crate::models::sessions::Session;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::env::parse_var;
use actix_web::http::header::AUTHORIZATION;
use actix_web::web::{Data, ServiceConfig, scope};
use actix_web::{HttpRequest, HttpResponse, delete, get, post, web};
use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use sqlx::PgPool;
use woothee::parser::Parser;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("session")
            .service(list)
            .service(delete)
            .service(refresh),
    );
}

pub struct SessionMetadata {
    pub city: Option<String>,
    pub country: Option<String>,
    pub ip: String,

    pub os: Option<String>,
    pub platform: Option<String>,
    pub user_agent: String,
}

pub async fn get_session_metadata(
    req: &HttpRequest,
) -> Result<SessionMetadata, AuthenticationError> {
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
        .unwrap_or("No user agent");

    let parser = Parser::new();
    let info = parser.parse(user_agent);
    let os = if let Some(info) = info {
        Some((info.os, info.name))
    } else {
        None
    };

    Ok(SessionMetadata {
        os: os.map(|x| x.0.to_string()),
        platform: os.map(|x| x.1.to_string()),
        city: city.map(|x| x.to_string()),
        country: country.map(|x| x.to_string()),
        ip: ip_addr
            .ok_or_else(|| AuthenticationError::InvalidCredentials)?
            .to_string(),
        user_agent: user_agent.to_string(),
    })
}

pub async fn issue_session(
    req: HttpRequest,
    user_id: DBUserId,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    redis: &RedisPool,
) -> Result<DBSession, AuthenticationError> {
    let metadata = get_session_metadata(&req).await?;

    let session = ChaCha20Rng::from_entropy()
        .sample_iter(&Alphanumeric)
        .take(60)
        .map(char::from)
        .collect::<String>();

    let session = format!("mra_{session}");

    let id = SessionBuilder {
        session,
        user_id,
        os: metadata.os,
        platform: metadata.platform,
        city: metadata.city,
        country: metadata.country,
        ip: metadata.ip,
        user_agent: metadata.user_agent,
    }
    .insert(transaction)
    .await?;

    let session = DBSession::get_id(id, &mut **transaction, redis)
        .await?
        .ok_or_else(|| AuthenticationError::InvalidCredentials)?;

    DBSession::clear_cache(
        vec![(
            Some(session.id),
            Some(session.session.clone()),
            Some(session.user_id),
        )],
        redis,
    )
    .await?;

    Ok(session)
}

#[get("list")]
pub async fn list(
    req: HttpRequest,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::SESSION_READ]),
    )
    .await?
    .1;

    let session = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|x| x.to_str().ok())
        .ok_or_else(|| AuthenticationError::InvalidCredentials)?;

    let session_ids =
        DBSession::get_user_sessions(current_user.id.into(), &**pool, &redis)
            .await?;
    let sessions = DBSession::get_many_ids(&session_ids, &**pool, &redis)
        .await?
        .into_iter()
        .filter(|x| x.expires > Utc::now())
        .map(|x| Session::from(x, false, Some(session)))
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(sessions))
}

#[delete("{id}")]
pub async fn delete(
    info: web::Path<(String,)>,
    req: HttpRequest,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::SESSION_DELETE]),
    )
    .await?
    .1;

    let session = DBSession::get(info.into_inner().0, &**pool, &redis).await?;

    if let Some(session) = session {
        if session.user_id == current_user.id.into() {
            let mut transaction = pool.begin().await?;
            DBSession::remove(session.id, &mut transaction).await?;
            transaction.commit().await?;
            DBSession::clear_cache(
                vec![(
                    Some(session.id),
                    Some(session.session),
                    Some(session.user_id),
                )],
                &redis,
            )
            .await?;
        }
    }

    Ok(HttpResponse::NoContent().body(""))
}

#[post("refresh")]
pub async fn refresh(
    req: HttpRequest,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let current_user =
        get_user_from_headers(&req, &**pool, &redis, &session_queue, None)
            .await?
            .1;
    let session = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|x| x.to_str().ok())
        .ok_or_else(|| {
            ApiError::Authentication(AuthenticationError::InvalidCredentials)
        })?;

    let session = DBSession::get(session, &**pool, &redis).await?;

    if let Some(session) = session {
        if current_user.id != session.user_id.into()
            || session.refresh_expires < Utc::now()
        {
            return Err(ApiError::Authentication(
                AuthenticationError::InvalidCredentials,
            ));
        }

        let mut transaction = pool.begin().await?;

        DBSession::remove(session.id, &mut transaction).await?;
        let new_session =
            issue_session(req, session.user_id, &mut transaction, &redis)
                .await?;
        transaction.commit().await?;
        DBSession::clear_cache(
            vec![(
                Some(session.id),
                Some(session.session),
                Some(session.user_id),
            )],
            &redis,
        )
        .await?;

        Ok(HttpResponse::Ok().json(Session::from(new_session, true, None)))
    } else {
        Err(ApiError::Authentication(
            AuthenticationError::InvalidCredentials,
        ))
    }
}
