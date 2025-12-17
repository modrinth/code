use std::{collections::HashMap, net::Ipv4Addr, sync::Arc};

use crate::{
    auth::get_user_from_headers,
    database::{
        models::{DBAffiliateCode, DBAffiliateCodeId, DBUser, DBUserId},
        redis::RedisPool,
    },
    models::{
        analytics::AffiliateCodeClick, ids::AffiliateCodeId, pats::Scopes,
        users::Badges, v3::affiliate_code::AffiliateCode,
    },
    queue::{analytics::AnalyticsQueue, session::AuthQueue},
    routes::analytics::FILTERED_HEADERS,
    util::{
        date::get_current_tenths_of_ms, env::parse_strings_from_var,
        error::Context,
    },
};
use actix_web::{HttpRequest, delete, get, patch, post, put, web};
use ariadne::ids::UserId;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::trace;
use url::Url;

use crate::routes::ApiError;

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(ingest_click)
        .service(get_all)
        .service(create)
        .service(get)
        .service(delete)
        .service(patch);
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct IngestClick {
    pub url: Url,
    pub affiliate_code_id: AffiliateCodeId,
}

#[utoipa::path]
#[post("/ingest-click")]
async fn ingest_click(
    req: HttpRequest,
    web::Json(ingest_click): web::Json<IngestClick>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    analytics_queue: web::Data<Arc<AnalyticsQueue>>,
) -> Result<(), ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::empty(),
    )
    .await
    .map(|(_, user)| user)
    .ok();
    let conn_info = req.connection_info().peer_addr().map(|x| x.to_string());

    let url = ingest_click.url;
    let domain = url.host_str().ok_or_else(|| {
        ApiError::InvalidInput("invalid page view URL specified!".to_string())
    })?;
    let url_origin = url.origin().ascii_serialization();

    let is_valid_url_origin =
        parse_strings_from_var("ANALYTICS_ALLOWED_ORIGINS")
            .unwrap_or_default()
            .iter()
            .any(|origin| origin == "*" || url_origin == *origin);

    if !is_valid_url_origin {
        return Err(ApiError::InvalidInput(
            "invalid page view URL specified!".to_string(),
        ));
    }

    let exists = sqlx::query!(
        "
        SELECT 1 AS exists FROM affiliate_codes WHERE id = $1
        ",
        DBAffiliateCodeId::from(ingest_click.affiliate_code_id) as _
    )
    .fetch_optional(&**pool)
    .await
    .wrap_internal_err("failed to check if code exists")?;
    if exists.is_none() {
        // don't allow enumerating affiliate codes
        return Ok(());
    }

    let headers = req
        .headers()
        .into_iter()
        .map(|(key, val)| {
            (
                key.to_string().to_lowercase(),
                val.to_str().unwrap_or_default().to_string(),
            )
        })
        .collect::<HashMap<String, String>>();

    let ip = crate::util::ip::convert_to_ip_v6(
        if let Some(header) = headers.get("cf-connecting-ip") {
            header
        } else {
            conn_info.as_deref().unwrap_or_default()
        },
    )
    .unwrap_or_else(|_| Ipv4Addr::new(127, 0, 0, 1).to_ipv6_mapped());

    let click = AffiliateCodeClick {
        recorded: get_current_tenths_of_ms(),
        domain: domain.to_string(),
        user_id: user.map(|user| user.id.0).unwrap_or_default(),
        affiliate_code_id: ingest_click.affiliate_code_id.0,
        ip,
        country: headers
            .get("cf-ipcountry")
            .map(|x| x.to_string())
            .unwrap_or_default(),
        user_agent: headers.get("user-agent").cloned().unwrap_or_default(),
        headers: headers
            .into_iter()
            .filter(|x| !FILTERED_HEADERS.contains(&&*x.0))
            .collect(),
    };

    trace!("Ingested affiliate code click {click:?}");
    analytics_queue.add_affiliate_code_click(click);

    Ok(())
}

#[utoipa::path(
    responses((status = OK, body = inline(Vec<AffiliateCode>)))
)]
#[get("")]
async fn get_all(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<Vec<AffiliateCode>>, ApiError> {
    let (_, user) = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await?;

    if user.role.is_admin() {
        let codes = DBAffiliateCode::get_all(&**pool)
            .await
            .wrap_internal_err("failed to get all affiliate codes")?;
        let codes = codes
            .into_iter()
            .map(|code| AffiliateCode::from(code, true))
            .collect::<Vec<_>>();
        Ok(web::Json(codes))
    } else if user.badges.contains(Badges::AFFILIATE) {
        let codes =
            DBAffiliateCode::get_by_affiliate(DBUserId::from(user.id), &**pool)
                .await
                .wrap_internal_err("failed to get all affiliate codes")?;
        let codes = codes
            .into_iter()
            .map(|code| AffiliateCode::from(code, false))
            .collect::<Vec<_>>();
        Ok(web::Json(codes))
    } else {
        Err(ApiError::CustomAuthentication(
            "You do not have permission to view affiliate codes!".to_string(),
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateRequest {
    pub affiliate: Option<UserId>,
    pub source_name: String,
}

#[utoipa::path(
    responses((status = OK, body = inline(AffiliateCode)))
)]
#[put("")]
async fn create(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    body: web::Json<CreateRequest>,
) -> Result<web::Json<AffiliateCode>, ApiError> {
    let (_, creator) = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await?;

    let is_admin = creator.role.is_admin();
    let is_affiliate = creator.badges.contains(Badges::AFFILIATE);

    if !is_admin && !is_affiliate {
        return Err(ApiError::CustomAuthentication(
            "You do not have permission to create an affiliate code!"
                .to_string(),
        ));
    }

    let creator_id = DBUserId::from(creator.id);
    let affiliate_id = if is_admin {
        if let Some(affiliate) = body.affiliate {
            DBUserId::from(affiliate)
        } else {
            creator_id
        }
    } else {
        creator_id
    };

    if affiliate_id != creator_id {
        let Some(_affiliate_user) =
            DBUser::get_id(affiliate_id, &**pool, &redis).await?
        else {
            return Err(ApiError::CustomAuthentication(
                "Affiliate user not found!".to_string(),
            ));
        };
    }

    let mut transaction = pool.begin().await?;

    let affiliate_code_id =
        crate::database::models::generate_affiliate_code_id(&mut transaction)
            .await?;

    let code = DBAffiliateCode {
        id: affiliate_code_id,
        created_at: Utc::now(),
        created_by: creator_id,
        affiliate: affiliate_id,
        source_name: body.source_name.clone(),
    };
    code.insert(&mut *transaction)
        .await
        .wrap_internal_err("failed to insert affiliate code")?;

    transaction
        .commit()
        .await
        .wrap_internal_err("failed to commit transaction")?;

    Ok(web::Json(AffiliateCode::from(code, is_admin)))
}

#[utoipa::path(
    responses((status = OK, body = inline(AffiliateCode)))
)]
#[get("/{id}")]
async fn get(
    req: HttpRequest,
    path: web::Path<(AffiliateCodeId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<AffiliateCode>, ApiError> {
    let (_, user) = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await?;

    let (affiliate_code_id,) = path.into_inner();
    let affiliate_code_id = DBAffiliateCodeId::from(affiliate_code_id);

    if let Some(model) =
        DBAffiliateCode::get_by_id(affiliate_code_id, &**pool).await?
    {
        let is_admin = user.role.is_admin();
        let is_owner = model.affiliate == DBUserId::from(user.id);

        if is_admin || is_owner {
            Ok(web::Json(AffiliateCode::from(model, is_admin)))
        } else {
            Err(ApiError::NotFound)
        }
    } else {
        Err(ApiError::NotFound)
    }
}

#[utoipa::path]
#[delete("/{id}")]
async fn delete(
    req: HttpRequest,
    path: web::Path<(AffiliateCodeId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<(), ApiError> {
    let (_, user) = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await?;

    let (affiliate_code_id,) = path.into_inner();
    let affiliate_code_id = DBAffiliateCodeId::from(affiliate_code_id);

    if let Some(model) =
        DBAffiliateCode::get_by_id(affiliate_code_id, &**pool).await?
    {
        let is_admin = user.role.is_admin();
        let is_owner = model.affiliate == DBUserId::from(user.id);

        if is_admin || is_owner {
            let result =
                DBAffiliateCode::remove(affiliate_code_id, &**pool).await?;
            if result.is_some() {
                Ok(())
            } else {
                Err(ApiError::NotFound)
            }
        } else {
            Err(ApiError::NotFound)
        }
    } else {
        Err(ApiError::NotFound)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PatchRequest {
    pub source_name: String,
}

#[utoipa::path]
#[patch("/{id}")]
async fn patch(
    req: HttpRequest,
    path: web::Path<(AffiliateCodeId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    body: web::Json<PatchRequest>,
) -> Result<(), ApiError> {
    let (_, user) = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await?;

    let (affiliate_code_id,) = path.into_inner();
    let affiliate_code_id = DBAffiliateCodeId::from(affiliate_code_id);

    let existing_code = DBAffiliateCode::get_by_id(affiliate_code_id, &**pool)
        .await?
        .ok_or(ApiError::NotFound)?;

    let is_admin = user.role.is_admin();
    let is_owner = existing_code.affiliate == DBUserId::from(user.id);

    if !is_admin && !is_owner {
        return Err(ApiError::NotFound);
    }

    if !is_admin && !user.badges.contains(Badges::AFFILIATE) {
        return Err(ApiError::CustomAuthentication(
            "You do not have permission to update affiliate codes!".to_string(),
        ));
    }

    DBAffiliateCode::update_source_name(
        affiliate_code_id,
        &body.source_name,
        &**pool,
    )
    .await
    .wrap_internal_err("failed to update affiliate code source name")?;

    Ok(())
}
