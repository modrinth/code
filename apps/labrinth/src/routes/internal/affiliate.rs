use crate::{
    auth::get_user_from_headers,
    database::{
        models::{DBAffiliateCode, DBAffiliateCodeId, DBUser, DBUserId},
        redis::RedisPool,
    },
    models::{
        ids::AffiliateCodeId,
        pats::Scopes,
        v3::affiliate_code::{AdminAffiliateCode, AffiliateCode},
    },
    queue::session::AuthQueue,
};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{self, Json},
};
use ariadne::ids::UserId;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::routes::ApiError;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("affiliate")
            .route("/code", web::get().to(code_get_all))
            .route("/code", web::put().to(code_create))
            .route("/code/{id}", web::get().to(code_get))
            .route("/code/{id}", web::delete().to(code_delete))
            .route("/self", web::get().to(self_get)),
    );
}

#[derive(Serialize)]
struct CodeGetAllResponse(Vec<AdminAffiliateCode>);

async fn code_get_all(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<Json<CodeGetAllResponse>, ApiError> {
    let (_, user) = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await?;

    if !user.role.is_admin() {
        return Err(ApiError::CustomAuthentication(
            "You do not have permission to read all affiliate codes!"
                .to_string(),
        ));
    }

    let codes = DBAffiliateCode::get_all(&**pool).await?;
    let codes = codes
        .into_iter()
        .map(AdminAffiliateCode::from)
        .collect::<Vec<_>>();

    Ok(Json(CodeGetAllResponse(codes)))
}

#[derive(Serialize, Deserialize)]
struct CodeCreateRequest {
    affiliate: UserId,
}

#[derive(Serialize)]
struct CodeCreateResponse(AdminAffiliateCode);

async fn code_create(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    body: web::Json<CodeCreateRequest>,
) -> Result<Json<CodeCreateResponse>, ApiError> {
    let (_, creator) = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await?;

    if !creator.role.is_admin() {
        return Err(ApiError::CustomAuthentication(
            "You do not have permission to create an affiliate code!"
                .to_string(),
        ));
    }

    let creator_id = DBUserId::from(creator.id);
    let affiliate_id = DBUserId::from(body.affiliate);
    let Some(_affiliate_user) =
        DBUser::get_id(affiliate_id, &**pool, &redis).await?
    else {
        return Err(ApiError::CustomAuthentication(
            "Affiliate user not found!".to_string(),
        ));
    };

    let mut transaction = pool.begin().await?;

    let affiliate_code_id =
        crate::database::models::generate_affiliate_code_id(&mut transaction)
            .await?;

    let code = DBAffiliateCode {
        id: affiliate_code_id,
        created_at: Utc::now(),
        created_by: creator_id,
        affiliate: affiliate_id,
    };
    code.insert(&mut *transaction).await?;

    transaction.commit().await?;

    Ok(Json(CodeCreateResponse(AdminAffiliateCode::from(code))))
}

#[derive(Serialize)]
struct CodeGetResponse(AdminAffiliateCode);

async fn code_get(
    req: HttpRequest,
    path: web::Path<(AffiliateCodeId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<Json<CodeGetResponse>, ApiError> {
    let (_, user) = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await?;

    if !user.role.is_admin() {
        return Err(ApiError::CustomAuthentication(
            "You do not have permission to read an affiliate code!".to_string(),
        ));
    }

    let (affiliate_code_id,) = path.into_inner();
    let affiliate_code_id = DBAffiliateCodeId::from(affiliate_code_id);

    if let Some(model) =
        DBAffiliateCode::get_by_id(affiliate_code_id, &**pool).await?
    {
        let model = AdminAffiliateCode::from(model);
        Ok(Json(CodeGetResponse(model)))
    } else {
        Err(ApiError::NotFound)
    }
}

async fn code_delete(
    req: HttpRequest,
    path: web::Path<(AffiliateCodeId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let (_, user) = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await?;

    if !user.role.is_admin() {
        return Err(ApiError::CustomAuthentication(
            "You do not have permission to delete an affiliate code!"
                .to_string(),
        ));
    }

    let (affiliate_code_id,) = path.into_inner();
    let affiliate_code_id = DBAffiliateCodeId::from(affiliate_code_id);

    let result = DBAffiliateCode::remove(affiliate_code_id, &**pool).await?;

    if result.is_some() {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(ApiError::NotFound)
    }
}

#[derive(Serialize)]
struct SelfGetResponse(Vec<AffiliateCode>);

async fn self_get(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<Json<SelfGetResponse>, ApiError> {
    let (_, user) = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await?;

    let codes =
        DBAffiliateCode::get_by_affiliate(DBUserId::from(user.id), &**pool)
            .await?;

    let codes = codes
        .into_iter()
        .map(AffiliateCode::from)
        .collect::<Vec<_>>();

    Ok(Json(SelfGetResponse(codes)))
}
