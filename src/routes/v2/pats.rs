/*!
Current edition of Ory kratos does not support PAT access of data, so this module is how we allow for PAT authentication.


Just as a summary: Don't implement this flow in your application!
*/

use crate::database;
use crate::database::models::generate_pat_id;
use crate::models::ids::base62_impl::{parse_base62, to_base62};

use crate::models::users::UserId;
use crate::routes::ApiError;
use crate::util::auth::get_user_from_headers;
use crate::util::pat::{generate_pat, PersonalAccessToken};

use actix_web::web::{self, Data, Query};
use actix_web::{delete, get, patch, post, HttpRequest, HttpResponse};
use chrono::{Duration, Utc};

use serde::Deserialize;
use sqlx::postgres::PgPool;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_pats);
    cfg.service(create_pat);
    cfg.service(edit_pat);
    cfg.service(delete_pat);
}

#[derive(Deserialize)]
pub struct CreatePersonalAccessToken {
    pub scope: i64, // todo: should be a vec of enum
    pub name: Option<String>,
    pub expire_in_days: i64, // resets expiry to expire_in_days days from now
}

#[derive(Deserialize)]
pub struct ModifyPersonalAccessToken {
    #[serde(default, with = "::serde_with::rust::double_option")]
    pub name: Option<Option<String>>,
    pub expire_in_days: Option<i64>, // resets expiry to expire_in_days days from now
}

// GET /pat
// Get all personal access tokens for the given user. Minos/Kratos cookie must be attached for it to work.
// Does not return the actual access token, only the ID + metadata.
#[get("pat")]
pub async fn get_pats(req: HttpRequest, pool: Data<PgPool>) -> Result<HttpResponse, ApiError> {
    let user: crate::models::users::User = get_user_from_headers(req.headers(), &**pool).await?;
    let db_user_id: database::models::UserId = database::models::UserId::from(user.id);

    let pats = sqlx::query!(
        "
            SELECT id, name, user_id, scope, expires_at
            FROM pats
            WHERE user_id = $1
            ",
        db_user_id.0
    )
    .fetch_all(&**pool)
    .await?;

    let pats = pats
        .into_iter()
        .map(|pat| PersonalAccessToken {
            id: to_base62(pat.id as u64),
            scope: pat.scope,
            name: pat.name,
            expires_at: pat.expires_at,
            access_token: None,
            user_id: UserId(pat.user_id as u64),
        })
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(pats))
}

// POST /pat
// Create a new personal access token for the given user. Minos/Kratos cookie must be attached for it to work.
// All PAT tokens are base62 encoded, and are prefixed with "mod_"
#[post("pat")]
pub async fn create_pat(
    req: HttpRequest,
    Query(info): Query<CreatePersonalAccessToken>, // callback url
    pool: Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let user: crate::models::users::User = get_user_from_headers(req.headers(), &**pool).await?;
    let db_user_id: database::models::UserId = database::models::UserId::from(user.id);

    let mut transaction: sqlx::Transaction<sqlx::Postgres> = pool.begin().await?;

    let pat = generate_pat_id(&mut transaction).await?;
    let access_token = generate_pat(&mut transaction).await?;
    let expiry = Utc::now().naive_utc() + Duration::days(info.expire_in_days);
    if info.expire_in_days <= 0 {
        return Err(ApiError::InvalidInput(
            "'expire_in_days' must be greater than 0".to_string(),
        ));
    }

    sqlx::query!(
        "
            INSERT INTO pats (id, name, access_token, user_id, scope, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ",
        pat.0,
        info.name,
        access_token,
        db_user_id.0,
        info.scope,
        expiry
    )
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(HttpResponse::Ok().json(PersonalAccessToken {
        id: to_base62(pat.0 as u64),
        access_token: Some(access_token),
        name: info.name,
        scope: info.scope,
        user_id: user.id,
        expires_at: expiry,
    }))
}

// PATCH /pat/(id)
// Edit an access token of id "id" for the given user.
// 'None' will mean not edited. Minos/Kratos cookie or PAT must be attached for it to work.
#[patch("pat/{id}")]
pub async fn edit_pat(
    req: HttpRequest,
    id: web::Path<String>,
    Query(info): Query<ModifyPersonalAccessToken>, // callback url
    pool: Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let user: crate::models::users::User = get_user_from_headers(req.headers(), &**pool).await?;
    let pat_id = database::models::PatId(parse_base62(&id)? as i64);
    let db_user_id: database::models::UserId = database::models::UserId::from(user.id);

    if let Some(expire_in_days) = info.expire_in_days {
        if expire_in_days <= 0 {
            return Err(ApiError::InvalidInput(
                "'expire_in_days' must be greater than 0".to_string(),
            ));
        }
    }

    // Get the singular PAT and user combination (failing immediately if it doesn't exist)
    let mut transaction = pool.begin().await?;
    let row = sqlx::query!(
        "
        SELECT id, name, scope, user_id, expires_at FROM pats
        WHERE id = $1 AND user_id = $2
        ",
        pat_id.0,
        db_user_id.0 // included for safety
    )
    .fetch_one(&**pool)
    .await?;

    let pat = PersonalAccessToken {
        id: to_base62(row.id as u64),
        access_token: None,
        user_id: UserId::from(db_user_id),
        name: info.name.unwrap_or(row.name),
        scope: row.scope,
        expires_at: info
            .expire_in_days
            .map(|d| Utc::now().naive_utc() + Duration::days(d))
            .unwrap_or(row.expires_at),
    };

    sqlx::query!(
        "
        UPDATE pats SET
            name = $1,
            expires_at = $2
        WHERE id = $3
        ",
        pat.name,
        pat.expires_at,
        parse_base62(&pat.id)? as i64
    )
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;

    Ok(HttpResponse::Ok().json(pat))
}

// DELETE /pat
// Delete a personal access token for the given user. Minos/Kratos cookie must be attached for it to work.
#[delete("pat/{id}")]
pub async fn delete_pat(
    req: HttpRequest,
    id: web::Path<String>,
    pool: Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let user: crate::models::users::User = get_user_from_headers(req.headers(), &**pool).await?;
    let pat_id = database::models::PatId(parse_base62(&id)? as i64);
    let db_user_id: database::models::UserId = database::models::UserId::from(user.id);

    // Get the singular PAT and user combination (failing immediately if it doesn't exist)
    // This is to prevent users from deleting other users' PATs
    let pat_id = sqlx::query!(
        "
        SELECT id FROM pats
        WHERE id = $1 AND user_id = $2
        ",
        pat_id.0,
        db_user_id.0
    )
    .fetch_one(&**pool)
    .await?
    .id;

    let mut transaction = pool.begin().await?;
    sqlx::query!(
        "
        DELETE FROM pats
        WHERE id = $1
        ",
        pat_id,
    )
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;

    Ok(HttpResponse::Ok().finish())
}
