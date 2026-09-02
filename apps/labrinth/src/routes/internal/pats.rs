use crate::database;
use crate::database::models::generate_pat_id;
use crate::util::error::Context as _;

use crate::auth::get_user_from_headers;
use crate::routes::ApiError;

use actix_web::web::{self, Data};
use actix_web::{HttpRequest, HttpResponse, delete, get, patch, post};
use chrono::{DateTime, Utc};
use rand::Rng;
use rand::distributions::Alphanumeric;
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;
use xredis::RedisPool;

use crate::database::PgPool;
use crate::database::models::notification_item::NotificationBuilder;
use crate::models::notifications::NotificationBody;
use crate::models::pats::{PersonalAccessToken, Scopes};
use crate::queue::session::AuthQueue;
use serde::Deserialize;
use validator::Validate;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_pats);
    cfg.service(create_pat);
    cfg.service(edit_pat);
    cfg.service(delete_pat);
}

/// List personal access tokens.  
#[utoipa::path(
	tag = "personal access tokens",
    get,
	operation_id = "getPats",
	responses(
		(status = 200, description = "List of PATs", body = serde_json::Value),
		(status = 401, description = "Unauthorized")
	),
    security(("bearer_auth" = ["PAT_READ"]))
)]
#[get("/pat")]
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
        Scopes::PAT_READ,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;

    let pat_ids =
        database::models::pat_item::DBPersonalAccessToken::get_user_pats(
            user.id.into(),
            &**pool,
            &redis,
        )
        .await
        .wrap_internal_err("fetching personal access tokens from database")?;
    let pats = database::models::pat_item::DBPersonalAccessToken::get_many_ids(
        &pat_ids, &**pool, &redis,
    )
    .await
    .wrap_internal_err("fetching personal access tokens from database")?;

    Ok(HttpResponse::Ok().json(
        pats.into_iter()
            .map(|x| PersonalAccessToken::from(x, false))
            .collect::<Vec<_>>(),
    ))
}

#[derive(Deserialize, Validate, utoipa::ToSchema)]
pub struct NewPersonalAccessToken {
    pub scopes: Scopes,
    #[validate(length(min = 3, max = 255))]
    pub name: String,
    pub expires: DateTime<Utc>,
}

/// Create a personal access token.  
#[utoipa::path(
	tag = "personal access tokens",
    post,
	operation_id = "createPat",
	responses(
		(status = 200, description = "PAT created", body = serde_json::Value),
		(status = 400, description = "Invalid input"),
		(status = 401, description = "Unauthorized")
	),
    security(("bearer_auth" = ["PAT_CREATE"]))
)]
#[post("/pat")]
pub async fn create_pat(
    req: HttpRequest,
    info: web::Json<NewPersonalAccessToken>,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    info.0
        .validate()
        .map_err(|err| eyre::eyre!(err))
        .wrap_request_err("validating request")?;

    if info.scopes.is_restricted() {
        return Err(ApiError::Request(eyre::eyre!(
            "Invalid scopes requested!",
        )));
    }
    if info.expires < Utc::now() {
        return Err(ApiError::Request(eyre::eyre!(
            "Expire date must be in the future!",
        )));
    }

    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PAT_CREATE,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;

    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("starting database transaction")?;

    let id = generate_pat_id(&mut transaction)
        .await
        .wrap_internal_err("generating pat ID")?;

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
    .await
    .wrap_internal_err("inserting database records for `create_pat`")?;

    NotificationBuilder {
        body: NotificationBody::PatCreated {
            token_name: name.clone(),
        },
    }
    .insert(user.id.into(), &mut transaction, &redis)
    .await
    .wrap_internal_err("inserting database records for `create_pat`")?;
    transaction
        .commit()
        .await
        .wrap_internal_err("committing database transaction")?;

    database::models::pat_item::DBPersonalAccessToken::clear_cache(
        vec![(None, None, Some(user.id.into()))],
        &redis,
    )
    .await
    .wrap_internal_err("clearing cached data from Redis")?;

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

#[derive(Deserialize, Validate, utoipa::ToSchema)]
pub struct ModifyPersonalAccessToken {
    pub scopes: Option<Scopes>,
    #[validate(length(min = 3, max = 255))]
    pub name: Option<String>,
    pub expires: Option<DateTime<Utc>>,
}

/// Update a personal access token.  
#[utoipa::path(
	tag = "personal access tokens",
    patch,
    operation_id = "editPat",
    params(
        ("id" = String, Path, description = "The PAT ID")
    ),
    responses(
        (status = 204, description = "PAT updated"),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = ["PAT_WRITE"]))
)]
#[patch("/pat/{id}")]
pub async fn edit_pat(
    req: HttpRequest,
    id: web::Path<(String,)>,
    info: web::Json<ModifyPersonalAccessToken>,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    info.0
        .validate()
        .map_err(|err| eyre::eyre!(err))
        .wrap_request_err("validating request")?;

    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PAT_WRITE,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;

    let id = id.into_inner().0;
    let pat = database::models::pat_item::DBPersonalAccessToken::get(
        &id, &**pool, &redis,
    )
    .await
    .wrap_internal_err("fetching personal access token from database")?;

    if let Some(pat) = pat
        && pat.user_id == user.id.into()
    {
        let mut transaction = pool
            .begin()
            .await
            .wrap_internal_err("starting database transaction")?;

        if let Some(scopes) = &info.scopes {
            if scopes.is_restricted() {
                return Err(ApiError::Request(eyre::eyre!(
                    "Invalid scopes requested!",
                )));
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
            .execute(&mut transaction)
            .await
            .wrap_internal_err("querying database for `edit_pat`")?;
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
            .execute(&mut transaction)
            .await
            .wrap_internal_err("querying database for `edit_pat`")?;
        }
        if let Some(expires) = &info.expires {
            if expires < &Utc::now() {
                return Err(ApiError::Request(eyre::eyre!(
                    "Expire date must be in the future!",
                )));
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
            .execute(&mut transaction)
            .await
            .wrap_internal_err("querying database for `edit_pat`")?;
        }

        transaction
            .commit()
            .await
            .wrap_internal_err("committing database transaction")?;
        database::models::pat_item::DBPersonalAccessToken::clear_cache(
            vec![(Some(pat.id), Some(pat.access_token), Some(pat.user_id))],
            &redis,
        )
        .await
        .wrap_internal_err("clearing cached data from Redis")?;
    }

    Ok(HttpResponse::NoContent().finish())
}

/// Delete a personal access token.  
#[utoipa::path(
	tag = "personal access tokens",
    delete,
    operation_id = "deletePat",
    params(
        ("id" = String, Path, description = "The PAT ID")
    ),
    responses(
        (status = 204, description = "PAT deleted"),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = ["PAT_DELETE"]))
)]
#[delete("/pat/{id}")]
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
        Scopes::PAT_DELETE,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;
    let id = id.into_inner().0;
    let pat = database::models::pat_item::DBPersonalAccessToken::get(
        &id, &**pool, &redis,
    )
    .await
    .wrap_internal_err("fetching personal access token from database")?;

    if let Some(pat) = pat
        && pat.user_id == user.id.into()
    {
        let mut transaction = pool
            .begin()
            .await
            .wrap_internal_err("starting database transaction")?;
        database::models::pat_item::DBPersonalAccessToken::remove(
            pat.id,
            &mut transaction,
        )
        .await
        .wrap_internal_err("deleting personal access token from database")?;
        transaction
            .commit()
            .await
            .wrap_internal_err("committing database transaction")?;
        database::models::pat_item::DBPersonalAccessToken::clear_cache(
            vec![(Some(pat.id), Some(pat.access_token), Some(pat.user_id))],
            &redis,
        )
        .await
        .wrap_internal_err("clearing cached data from Redis")?;
    }

    Ok(HttpResponse::NoContent().finish())
}
