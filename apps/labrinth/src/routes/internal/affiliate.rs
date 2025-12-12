use crate::{
    auth::get_user_from_headers,
    database::{
        models::{DBAffiliateCode, DBAffiliateCodeId, DBUser, DBUserId},
        redis::RedisPool,
    },
    models::{
        ids::AffiliateCodeId, pats::Scopes, users::Badges,
        v3::affiliate_code::AffiliateCode,
    },
    queue::session::AuthQueue,
    util::error::Context,
};
use actix_web::{HttpRequest, delete, get, patch, put, web};
use ariadne::ids::UserId;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::routes::ApiError;

pub fn config(cfg: &mut utoipa_actix_web::service_config::ServiceConfig) {
    cfg.service(ingest_click)
        .service(get_all)
        .service(create)
        .service(get)
        .service(delete)
        .service(patch);
}

#[utoipa::path()]
#[get("/ingest-click")]
async fn ingest_click(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<(), ApiError> {
    // TODO: insert into clickhouse - see `clickhouse/mod.rs` unfinished code
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
