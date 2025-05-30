use crate::database;
use crate::database::models::generate_pat_id;

use crate::auth::get_user_from_headers;
use crate::routes::ApiError;

use crate::database::redis::RedisPool;
use actix_web::web::{self, Data};
use actix_web::{HttpRequest, HttpResponse, delete, get, patch, post};
use chrono::{DateTime, Utc};
use rand::Rng;
use rand::distributions::Alphanumeric;
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;

use crate::models::pats::{PersonalAccessToken, Scopes};
use crate::queue::session::AuthQueue;
use crate::util::validate::validation_errors_to_string;
use serde::Deserialize;
use sqlx::postgres::PgPool;
use validator::Validate;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_pats);
    cfg.service(create_pat);
    cfg.service(edit_pat);
    cfg.service(delete_pat);
}

#[get("pat")]
pub async fn get_pats(
    req: HttpRequest,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PAT_READ]),
    )
    .await?
    .1;

    let pat_ids =
        database::models::pat_item::DBPersonalAccessToken::get_user_pats(
            user.id.into(),
            &**pool,
            &redis,
        )
        .await?;
    let pats = database::models::pat_item::DBPersonalAccessToken::get_many_ids(
        &pat_ids, &**pool, &redis,
    )
    .await?;

    Ok(HttpResponse::Ok().json(
        pats.into_iter()
            .map(|x| PersonalAccessToken::from(x, false))
            .collect::<Vec<_>>(),
    ))
}

#[derive(Deserialize, Validate)]
pub struct NewPersonalAccessToken {
    pub scopes: Scopes,
    #[validate(length(min = 3, max = 255))]
    pub name: String,
    pub expires: DateTime<Utc>,
}

#[post("pat")]
pub async fn create_pat(
    req: HttpRequest,
    info: web::Json<NewPersonalAccessToken>,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    info.0.validate().map_err(|err| {
        ApiError::InvalidInput(validation_errors_to_string(err, None))
    })?;

    if info.scopes.is_restricted() {
        return Err(ApiError::InvalidInput(
            "Invalid scopes requested!".to_string(),
        ));
    }
    if info.expires < Utc::now() {
        return Err(ApiError::InvalidInput(
            "Expire date must be in the future!".to_string(),
        ));
    }

    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PAT_CREATE]),
    )
    .await?
    .1;

    let mut transaction = pool.begin().await?;

    let id = generate_pat_id(&mut transaction).await?;

    let token = ChaCha20Rng::from_entropy()
        .sample_iter(&Alphanumeric)
        .take(60)
        .map(char::from)
        .collect::<String>();
    let token = format!("mrp_{token}");

    let name = info.name.clone();
    database::models::pat_item::DBPersonalAccessToken {
        id,
        name: name.clone(),
        access_token: token.clone(),
        scopes: info.scopes,
        user_id: user.id.into(),
        created: Utc::now(),
        expires: info.expires,
        last_used: None,
    }
    .insert(&mut transaction)
    .await?;

    transaction.commit().await?;
    database::models::pat_item::DBPersonalAccessToken::clear_cache(
        vec![(None, None, Some(user.id.into()))],
        &redis,
    )
    .await?;

    Ok(HttpResponse::Ok().json(PersonalAccessToken {
        id: id.into(),
        name,
        access_token: Some(token),
        scopes: info.scopes,
        user_id: user.id,
        created: Utc::now(),
        expires: info.expires,
        last_used: None,
    }))
}

#[derive(Deserialize, Validate)]
pub struct ModifyPersonalAccessToken {
    pub scopes: Option<Scopes>,
    #[validate(length(min = 3, max = 255))]
    pub name: Option<String>,
    pub expires: Option<DateTime<Utc>>,
}

#[patch("pat/{id}")]
pub async fn edit_pat(
    req: HttpRequest,
    id: web::Path<(String,)>,
    info: web::Json<ModifyPersonalAccessToken>,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    info.0.validate().map_err(|err| {
        ApiError::InvalidInput(validation_errors_to_string(err, None))
    })?;

    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PAT_WRITE]),
    )
    .await?
    .1;

    let id = id.into_inner().0;
    let pat = database::models::pat_item::DBPersonalAccessToken::get(
        &id, &**pool, &redis,
    )
    .await?;

    if let Some(pat) = pat {
        if pat.user_id == user.id.into() {
            let mut transaction = pool.begin().await?;

            if let Some(scopes) = &info.scopes {
                if scopes.is_restricted() {
                    return Err(ApiError::InvalidInput(
                        "Invalid scopes requested!".to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    UPDATE pats
                    SET scopes = $1
                    WHERE id = $2
                    ",
                    scopes.bits() as i64,
                    pat.id.0
                )
                .execute(&mut *transaction)
                .await?;
            }
            if let Some(name) = &info.name {
                sqlx::query!(
                    "
                    UPDATE pats
                    SET name = $1
                    WHERE id = $2
                    ",
                    name,
                    pat.id.0
                )
                .execute(&mut *transaction)
                .await?;
            }
            if let Some(expires) = &info.expires {
                if expires < &Utc::now() {
                    return Err(ApiError::InvalidInput(
                        "Expire date must be in the future!".to_string(),
                    ));
                }

                sqlx::query!(
                    "
                    UPDATE pats
                    SET expires = $1
                    WHERE id = $2
                    ",
                    expires,
                    pat.id.0
                )
                .execute(&mut *transaction)
                .await?;
            }

            transaction.commit().await?;
            database::models::pat_item::DBPersonalAccessToken::clear_cache(
                vec![(Some(pat.id), Some(pat.access_token), Some(pat.user_id))],
                &redis,
            )
            .await?;
        }
    }

    Ok(HttpResponse::NoContent().finish())
}

#[delete("pat/{id}")]
pub async fn delete_pat(
    req: HttpRequest,
    id: web::Path<(String,)>,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::PAT_DELETE]),
    )
    .await?
    .1;
    let id = id.into_inner().0;
    let pat = database::models::pat_item::DBPersonalAccessToken::get(
        &id, &**pool, &redis,
    )
    .await?;

    if let Some(pat) = pat {
        if pat.user_id == user.id.into() {
            let mut transaction = pool.begin().await?;
            database::models::pat_item::DBPersonalAccessToken::remove(
                pat.id,
                &mut transaction,
            )
            .await?;
            transaction.commit().await?;
            database::models::pat_item::DBPersonalAccessToken::clear_cache(
                vec![(Some(pat.id), Some(pat.access_token), Some(pat.user_id))],
                &redis,
            )
            .await?;
        }
    }

    Ok(HttpResponse::NoContent().finish())
}
