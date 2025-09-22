use crate::{
    auth::get_user_from_headers,
    database::{
        models::{DBAffiliateCode, DBAffiliateCodeId, DBUser, DBUserId},
        redis::RedisPool,
    },
    models::{
        ids::AffiliateCodeId,
        pats::Scopes,
        users::Badges,
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
            .route("/admin", web::get().to(admin_get_all))
            .route("/admin", web::put().to(admin_create))
            .route("/admin/{id}", web::get().to(admin_get))
            .route("/admin/{id}", web::delete().to(admin_delete))
            .route("/self", web::get().to(self_get_all))
            .route("/self", web::put().to(self_patch))
            .route("/self/{id}", web::delete().to(self_delete)),
    );
}

#[derive(Serialize)]
struct AdminGetAllResponse(Vec<AdminAffiliateCode>);

async fn admin_get_all(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<Json<AdminGetAllResponse>, ApiError> {
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

    Ok(Json(AdminGetAllResponse(codes)))
}

#[derive(Serialize, Deserialize)]
struct AdminCreateRequest {
    affiliate: UserId,
    source_name: String,
}

#[derive(Serialize)]
struct AdminCreateResponse(AdminAffiliateCode);

async fn admin_create(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    body: web::Json<AdminCreateRequest>,
) -> Result<Json<AdminCreateResponse>, ApiError> {
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
        source_name: body.source_name.clone(),
    };
    code.insert(&mut *transaction).await?;

    transaction.commit().await?;

    Ok(Json(AdminCreateResponse(AdminAffiliateCode::from(code))))
}

#[derive(Serialize)]
struct AdminGetResponse(AdminAffiliateCode);

async fn admin_get(
    req: HttpRequest,
    path: web::Path<(AffiliateCodeId,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<Json<AdminGetResponse>, ApiError> {
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
        Ok(Json(AdminGetResponse(model)))
    } else {
        Err(ApiError::NotFound)
    }
}

async fn admin_delete(
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
struct SelfGetAllResponse(Vec<AffiliateCode>);

async fn self_get_all(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<Json<SelfGetAllResponse>, ApiError> {
    let (_, user) = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await?;

    if !user.badges.contains(Badges::AFFILIATE) {
        return Err(ApiError::CustomAuthentication(
            "You do not have permission to view your affiliate codes!"
                .to_string(),
        ));
    }

    let codes =
        DBAffiliateCode::get_by_affiliate(DBUserId::from(user.id), &**pool)
            .await?;

    let codes = codes
        .into_iter()
        .map(AffiliateCode::from)
        .collect::<Vec<_>>();

    Ok(Json(SelfGetAllResponse(codes)))
}

#[derive(Deserialize)]
struct SelfPatchRequest {
    id: AffiliateCodeId,
    source_name: String,
}

async fn self_patch(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    body: web::Json<SelfPatchRequest>,
) -> Result<HttpResponse, ApiError> {
    let (_, user) = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await?;

    if !user.badges.contains(Badges::AFFILIATE) {
        return Err(ApiError::CustomAuthentication(
            "You do not have permission to update your affiliate codes!"
                .to_string(),
        ));
    }

    let affiliate_code_id = DBAffiliateCodeId::from(body.id);

    let existing_code = DBAffiliateCode::get_by_id(affiliate_code_id, &**pool)
        .await?
        .ok_or(ApiError::NotFound)?;

    if existing_code.affiliate != DBUserId::from(user.id) {
        return Err(ApiError::NotFound);
    }

    DBAffiliateCode::update_source_name(
        affiliate_code_id,
        &body.source_name,
        &**pool,
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

async fn self_delete(
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

    if !user.badges.contains(Badges::AFFILIATE) {
        return Err(ApiError::CustomAuthentication(
            "You do not have permission to delete your affiliate codes!"
                .to_string(),
        ));
    }

    let (affiliate_code_id,) = path.into_inner();
    let affiliate_code_id = DBAffiliateCodeId::from(affiliate_code_id);

    let code = DBAffiliateCode::get_by_id(affiliate_code_id, &**pool)
        .await?
        .ok_or(ApiError::NotFound)?;

    if code.affiliate != DBUserId::from(user.id) {
        return Err(ApiError::NotFound);
    }

    let result = DBAffiliateCode::remove(affiliate_code_id, &**pool).await?;
    if result.is_some() {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(ApiError::NotFound)
    }
}
