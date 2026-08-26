//! Eligibility checks for the Contributor/Translator/Proofreader profile badges.

use actix_web::web::Query;
use actix_web::{HttpRequest, HttpResponse, get, post, web};
use chrono::Duration;
use eyre::eyre;
use serde::{Deserialize, Serialize};
use xredis::RedisPool;

use crate::auth::get_user_from_headers;
use crate::auth::validate::get_user_record_from_bearer_token;
use crate::database::PgPool;
use crate::database::models::flow_item::DBFlow;
use crate::database::models::{DBUser, DBUserId};
use crate::env::ENV;
use crate::models::pats::Scopes;
use crate::models::users::Badges;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::error::Context;
use crate::util::{crowdin, github_contributor};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(check_contributor)
        .service(crowdin_init)
        .service(crowdin_callback);
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct BadgeCheckResponse {
    pub badges: Badges,
}

/// Re-checks the calling user's linked GitHub account for the Contributor badge.
#[utoipa::path(tag = "badges", responses((status = OK)))]
#[post("/contributor/check")]
pub async fn check_contributor(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let (_, user) = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::USER_READ,
    )
    .await
    .wrap_auth_err("authenticating API request")?;

    let db_user = DBUser::get_id(user.id.into(), &**pool, &redis)
        .await
        .wrap_internal_err("fetching user from database")?
        .ok_or_else(|| ApiError::NotFound(eyre!("resource not found")))?;

    let mut badges = db_user.badges;

    if !badges.contains(Badges::CONTRIBUTOR) {
        let Some(github_id) = db_user.github_id else {
            return Err(ApiError::Request(eyre!(
                "a GitHub account must be linked before checking contributor status",
            )));
        };

        let eligible = github_contributor::is_eligible_contributor(github_id)
            .await
            .wrap_internal_err("checking GitHub contributor status")?;

        if eligible {
            badges |= Badges::CONTRIBUTOR;

            sqlx::query!(
                "UPDATE users SET badges = $1 WHERE id = $2",
                badges.bits() as i64,
                db_user.id as DBUserId,
            )
            .execute(&**pool)
            .await
            .wrap_internal_err("updating user badges")?;

            DBUser::clear_caches(
                &[(db_user.id, Some(db_user.username))],
                &redis,
            )
            .await
            .wrap_internal_err("clearing cached data from Redis")?;
        }
    }

    Ok(HttpResponse::Ok().json(BadgeCheckResponse { badges }))
}

#[derive(Deserialize)]
pub struct CrowdinInitQuery {
    /// A first-party session token (`mra_...`).
    pub token: String,
}

fn crowdin_redirect_uri() -> String {
    format!("{}/_internal/badges/crowdin/callback", &ENV.SELF_ADDR)
}

/// Starts the on-demand Crowdin verification flow for the calling user.
#[utoipa::path(tag = "badges", responses((status = TEMPORARY_REDIRECT)))]
#[get("/crowdin/init")]
pub async fn crowdin_init(
    req: HttpRequest,
    Query(info): Query<CrowdinInitQuery>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    if !info.token.starts_with("mra_") {
        return Err(ApiError::Auth(eyre!("invalid session token")));
    }

    let (_, user) = get_user_record_from_bearer_token(
        &req,
        Some(&info.token),
        &**pool,
        &redis,
        &session_queue,
        false,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .ok_or_else(|| ApiError::Auth(eyre!("invalid session token")))?;

    let user_id: DBUserId = user.id.into();

    let state = DBFlow::CrowdinVerify { user_id }
        .insert(Duration::minutes(10), &redis)
        .await
        .wrap_internal_err("creating Crowdin verification flow")?;

    let url = crowdin::authorize_url(&state, &crowdin_redirect_uri());

    Ok(HttpResponse::TemporaryRedirect()
        .append_header(("Location", url.as_str()))
        .json(serde_json::json!({ "url": url })))
}

#[derive(Deserialize)]
pub struct CrowdinCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

/// Finishes the on-demand Crowdin verification flow and awards the
/// Translator/Proofreader badges if earned.
#[utoipa::path(tag = "badges", responses((status = TEMPORARY_REDIRECT)))]
#[get("/crowdin/callback")]
pub async fn crowdin_callback(
    Query(query): Query<CrowdinCallbackQuery>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let redirect_target = format!("{}/settings/account", &ENV.SITE_URL);

    let Some(state) = query.state else {
        return Err(ApiError::Request(eyre!("missing OAuth state")));
    };

    let flow = DBFlow::take_if(
        &state,
        |flow| matches!(flow, DBFlow::CrowdinVerify { .. }),
        &redis,
    )
    .await
    .wrap_internal_err("fetching Crowdin verification flow")?;

    let Some(DBFlow::CrowdinVerify { user_id }) = flow else {
        return Err(ApiError::Request(eyre!(
            "invalid or expired Crowdin verification flow",
        )));
    };

    if query.error.is_some() {
        let location = format!("{redirect_target}?crowdin_verified=denied");
        return Ok(HttpResponse::TemporaryRedirect()
            .append_header(("Location", location.as_str()))
            .finish());
    }

    let Some(code) = query.code else {
        return Err(ApiError::Request(eyre!("missing OAuth code")));
    };

    let access_token =
        crowdin::exchange_code(&code, &crowdin_redirect_uri())
            .await
            .wrap_internal_err("exchanging Crowdin OAuth code")?;

    let crowdin_user_id = crowdin::fetch_own_user_id(&access_token)
        .await
        .wrap_internal_err("fetching Crowdin user profile")?;

    let stats = crowdin::fetch_contribution_stats(crowdin_user_id)
        .await
        .wrap_internal_err("fetching Crowdin contribution stats")?;

    // Fall back to raw stats only if the role lookup itself fails.
    let is_proofreader = match crowdin::has_proofreader_role(crowdin_user_id)
        .await
    {
        Ok(is_proofreader) => is_proofreader,
        Err(_) => stats.approved > 0,
    };

    let db_user = DBUser::get_id(user_id, &**pool, &redis)
        .await
        .wrap_internal_err("fetching user from database")?
        .ok_or_else(|| ApiError::NotFound(eyre!("resource not found")))?;

    let mut badges = db_user.badges;
    let mut updated = false;

    if stats.has_contribution() && !badges.contains(Badges::TRANSLATOR) {
        badges |= Badges::TRANSLATOR;
        updated = true;
    }
    if is_proofreader && !badges.contains(Badges::PROOFREADER) {
        badges |= Badges::PROOFREADER;
        updated = true;
    }

    if updated {
        sqlx::query!(
            "UPDATE users SET badges = $1 WHERE id = $2",
            badges.bits() as i64,
            db_user.id as DBUserId,
        )
        .execute(&**pool)
        .await
        .wrap_internal_err("updating user badges")?;

        DBUser::clear_caches(&[(db_user.id, Some(db_user.username))], &redis)
            .await
            .wrap_internal_err("clearing cached data from Redis")?;
    }

    let location = format!("{redirect_target}?crowdin_verified=1");
    Ok(HttpResponse::TemporaryRedirect()
        .append_header(("Location", location.as_str()))
        .finish())
}
