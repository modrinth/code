use crate::{
    auth::get_user_from_headers,
    database::{
        models::{DBAffiliateCode, DBAffiliateCodeId, DBUser, DBUserId},
        redis::RedisPool,
    },
    models::{pats::Scopes, v3::affiliate_code::AffiliateCode},
    queue::session::AuthQueue,
};
use actix_web::{HttpRequest, HttpResponse, web};
use ariadne::ids::{UserId, base62_impl::parse_base62};
use chrono::Utc;
use serde::Deserialize;
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

#[derive(Deserialize)]
pub struct CreateCode {
    pub affiliate: UserId,
}

pub async fn code_get_all(
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
        .map(|code| AffiliateCode::from(code, true))
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(codes))
}

pub async fn code_create(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    body: web::Json<CreateCode>,
) -> Result<HttpResponse, ApiError> {
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

    Ok(HttpResponse::Ok().json(AffiliateCode::from(code, true)))
}

pub async fn code_get(
    req: HttpRequest,
    path: web::Path<(String,)>,
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
            "You do not have permission to read an affiliate code!".to_string(),
        ));
    }

    let (affiliate_code_id,) = path.into_inner();
    let affiliate_code_id =
        DBAffiliateCodeId(parse_base62(&affiliate_code_id)? as i64);

    if let Some(model) =
        DBAffiliateCode::get_by_id(affiliate_code_id, &**pool).await?
    {
        let model = AffiliateCode::from(model, true);
        Ok(HttpResponse::Ok().json(model))
    } else {
        Err(ApiError::NotFound)
    }
}

pub async fn code_delete(
    req: HttpRequest,
    path: web::Path<(String,)>,
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
    let affiliate_code_id =
        DBAffiliateCodeId(parse_base62(&affiliate_code_id)? as i64);

    let mut transaction = pool.begin().await?;

    let result =
        DBAffiliateCode::remove(affiliate_code_id, &mut *transaction).await?;

    transaction.commit().await?;

    if result.is_some() {
        Ok(HttpResponse::NoContent().body(""))
    } else {
        Err(ApiError::NotFound)
    }
}

async fn self_get(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("fetching user");

    let (_, user) = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS, // TODO: what token scope?
    )
    .await?;

    tracing::info!("uid = {:?}", user.id);

    let codes =
        DBAffiliateCode::get_by_affiliate(DBUserId::from(user.id), &**pool)
            .await?;

    tracing::info!("codes = {:?}", codes);

    let codes = codes
        .into_iter()
        .map(|data| AffiliateCode::from(data, false))
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(codes))
}
