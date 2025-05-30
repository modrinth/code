use std::{collections::HashSet, fmt::Display, sync::Arc};

use super::ApiError;
use crate::{
    auth::{checks::ValidateAuthorized, get_user_from_headers},
    database::{
        models::{
            DBOAuthClientId, DBUser, DatabaseError, generate_oauth_client_id,
            generate_oauth_redirect_id,
            oauth_client_authorization_item::DBOAuthClientAuthorization,
            oauth_client_item::{DBOAuthClient, DBOAuthRedirectUri},
        },
        redis::RedisPool,
    },
    models::{
        self,
        oauth_clients::{GetOAuthClientsRequest, OAuthClientCreationResult},
        pats::Scopes,
    },
    queue::session::AuthQueue,
    routes::v3::project_creation::CreateError,
    util::validate::validation_errors_to_string,
};
use crate::{
    file_hosting::FileHost, models::oauth_clients::DeleteOAuthClientQueryParam,
    util::routes::read_from_payload,
};
use actix_web::{
    HttpRequest, HttpResponse, delete, get, patch, post,
    web::{self, scope},
};
use ariadne::ids::base62_impl::parse_base62;
use chrono::Utc;
use itertools::Itertools;
use rand::{Rng, SeedableRng, distributions::Alphanumeric};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use validator::Validate;

use crate::models::ids::OAuthClientId;
use crate::util::img::{delete_old_images, upload_image_optimized};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        scope("oauth")
            .configure(crate::auth::oauth::config)
            .service(revoke_oauth_authorization)
            .service(oauth_client_create)
            .service(oauth_client_edit)
            .service(oauth_client_delete)
            .service(oauth_client_icon_edit)
            .service(oauth_client_icon_delete)
            .service(get_client)
            .service(get_clients)
            .service(get_user_oauth_authorizations),
    );
}

pub async fn get_user_clients(
    req: HttpRequest,
    info: web::Path<String>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::SESSION_ACCESS]),
    )
    .await?
    .1;

    let target_user = DBUser::get(&info.into_inner(), &**pool, &redis).await?;

    if let Some(target_user) = target_user {
        if target_user.id != current_user.id.into()
            && !current_user.role.is_admin()
        {
            return Err(ApiError::CustomAuthentication(
                "You do not have permission to see the OAuth clients of this user!".to_string(),
            ));
        }

        let clients =
            DBOAuthClient::get_all_user_clients(target_user.id, &**pool)
                .await?;

        let response = clients
            .into_iter()
            .map(models::oauth_clients::OAuthClient::from)
            .collect_vec();

        Ok(HttpResponse::Ok().json(response))
    } else {
        Err(ApiError::NotFound)
    }
}

#[get("app/{id}")]
pub async fn get_client(
    id: web::Path<OAuthClientId>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let clients = get_clients_inner(&[id.into_inner()], pool).await?;
    if let Some(client) = clients.into_iter().next() {
        Ok(HttpResponse::Ok().json(client))
    } else {
        Err(ApiError::NotFound)
    }
}

#[get("apps")]
pub async fn get_clients(
    info: web::Query<GetOAuthClientsRequest>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let ids: Vec<_> = info
        .ids
        .iter()
        .map(|id| parse_base62(id).map(OAuthClientId))
        .collect::<Result<_, _>>()?;

    let clients = get_clients_inner(&ids, pool).await?;

    Ok(HttpResponse::Ok().json(clients))
}

#[derive(Deserialize, Validate)]
pub struct NewOAuthApp {
    #[validate(
        custom(function = "crate::util::validate::validate_name"),
        length(min = 3, max = 255)
    )]
    pub name: String,

    #[validate(custom(
        function = "crate::util::validate::validate_no_restricted_scopes"
    ))]
    pub max_scopes: Scopes,

    pub redirect_uris: Vec<String>,

    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 255)
    )]
    pub url: Option<String>,

    #[validate(length(max = 255))]
    pub description: Option<String>,
}

#[post("app")]
pub async fn oauth_client_create(
    req: HttpRequest,
    new_oauth_app: web::Json<NewOAuthApp>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, CreateError> {
    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::SESSION_ACCESS]),
    )
    .await?
    .1;

    new_oauth_app.validate().map_err(|e| {
        CreateError::ValidationError(validation_errors_to_string(e, None))
    })?;

    let mut transaction = pool.begin().await?;

    let client_id = generate_oauth_client_id(&mut transaction).await?;

    let client_secret = generate_oauth_client_secret();
    let client_secret_hash = DBOAuthClient::hash_secret(&client_secret);

    let redirect_uris = create_redirect_uris(
        &new_oauth_app.redirect_uris,
        client_id,
        &mut transaction,
    )
    .await?;

    let client = DBOAuthClient {
        id: client_id,
        icon_url: None,
        raw_icon_url: None,
        max_scopes: new_oauth_app.max_scopes,
        name: new_oauth_app.name.clone(),
        redirect_uris,
        created: Utc::now(),
        created_by: current_user.id.into(),
        url: new_oauth_app.url.clone(),
        description: new_oauth_app.description.clone(),
        secret_hash: client_secret_hash,
    };
    client.clone().insert(&mut transaction).await?;

    transaction.commit().await?;

    let client = models::oauth_clients::OAuthClient::from(client);

    Ok(HttpResponse::Ok().json(OAuthClientCreationResult {
        client,
        client_secret,
    }))
}

#[delete("app/{id}")]
pub async fn oauth_client_delete(
    req: HttpRequest,
    client_id: web::Path<OAuthClientId>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::SESSION_ACCESS]),
    )
    .await?
    .1;

    let client =
        DBOAuthClient::get(client_id.into_inner().into(), &**pool).await?;
    if let Some(client) = client {
        client.validate_authorized(Some(&current_user))?;
        DBOAuthClient::remove(client.id, &**pool).await?;

        Ok(HttpResponse::NoContent().body(""))
    } else {
        Err(ApiError::NotFound)
    }
}

#[derive(Serialize, Deserialize, Validate)]
pub struct OAuthClientEdit {
    #[validate(
        custom(function = "crate::util::validate::validate_name"),
        length(min = 3, max = 255)
    )]
    pub name: Option<String>,

    #[validate(custom(
        function = "crate::util::validate::validate_no_restricted_scopes"
    ))]
    pub max_scopes: Option<Scopes>,

    #[validate(length(min = 1))]
    pub redirect_uris: Option<Vec<String>>,

    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 255)
    )]
    pub url: Option<Option<String>>,

    #[validate(length(max = 255))]
    pub description: Option<Option<String>>,
}

#[patch("app/{id}")]
pub async fn oauth_client_edit(
    req: HttpRequest,
    client_id: web::Path<OAuthClientId>,
    client_updates: web::Json<OAuthClientEdit>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::SESSION_ACCESS]),
    )
    .await?
    .1;

    client_updates.validate().map_err(|e| {
        ApiError::Validation(validation_errors_to_string(e, None))
    })?;

    if let Some(existing_client) =
        DBOAuthClient::get(client_id.into_inner().into(), &**pool).await?
    {
        existing_client.validate_authorized(Some(&current_user))?;

        let mut updated_client = existing_client.clone();
        let OAuthClientEdit {
            name,
            max_scopes,
            redirect_uris,
            url,
            description,
        } = client_updates.into_inner();
        if let Some(name) = name {
            updated_client.name = name;
        }

        if let Some(max_scopes) = max_scopes {
            updated_client.max_scopes = max_scopes;
        }

        if let Some(url) = url {
            updated_client.url = url;
        }

        if let Some(description) = description {
            updated_client.description = description;
        }

        let mut transaction = pool.begin().await?;
        updated_client
            .update_editable_fields(&mut *transaction)
            .await?;

        if let Some(redirects) = redirect_uris {
            edit_redirects(redirects, &existing_client, &mut transaction)
                .await?;
        }

        transaction.commit().await?;

        Ok(HttpResponse::Ok().body(""))
    } else {
        Err(ApiError::NotFound)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Extension {
    pub ext: String,
}

#[patch("app/{id}/icon")]
#[allow(clippy::too_many_arguments)]
pub async fn oauth_client_icon_edit(
    web::Query(ext): web::Query<Extension>,
    req: HttpRequest,
    client_id: web::Path<OAuthClientId>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    mut payload: web::Payload,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::SESSION_ACCESS]),
    )
    .await?
    .1;

    let client = DBOAuthClient::get((*client_id).into(), &**pool)
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput(
                "The specified client does not exist!".to_string(),
            )
        })?;

    client.validate_authorized(Some(&user))?;

    delete_old_images(
        client.icon_url.clone(),
        client.raw_icon_url.clone(),
        &***file_host,
    )
    .await?;

    let bytes = read_from_payload(
        &mut payload,
        262144,
        "Icons must be smaller than 256KiB",
    )
    .await?;
    let upload_result = upload_image_optimized(
        &format!("data/{client_id}"),
        bytes.freeze(),
        &ext.ext,
        Some(96),
        Some(1.0),
        &***file_host,
    )
    .await?;

    let mut transaction = pool.begin().await?;

    let mut editable_client = client.clone();
    editable_client.icon_url = Some(upload_result.url);
    editable_client.raw_icon_url = Some(upload_result.raw_url);

    editable_client
        .update_editable_fields(&mut *transaction)
        .await?;

    transaction.commit().await?;

    Ok(HttpResponse::NoContent().body(""))
}

#[delete("app/{id}/icon")]
pub async fn oauth_client_icon_delete(
    req: HttpRequest,
    client_id: web::Path<OAuthClientId>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::SESSION_ACCESS]),
    )
    .await?
    .1;

    let client = DBOAuthClient::get((*client_id).into(), &**pool)
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput(
                "The specified client does not exist!".to_string(),
            )
        })?;
    client.validate_authorized(Some(&user))?;

    delete_old_images(
        client.icon_url.clone(),
        client.raw_icon_url.clone(),
        &***file_host,
    )
    .await?;

    let mut transaction = pool.begin().await?;

    let mut editable_client = client.clone();
    editable_client.icon_url = None;
    editable_client.raw_icon_url = None;

    editable_client
        .update_editable_fields(&mut *transaction)
        .await?;
    transaction.commit().await?;

    Ok(HttpResponse::NoContent().body(""))
}

#[get("authorizations")]
pub async fn get_user_oauth_authorizations(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::SESSION_ACCESS]),
    )
    .await?
    .1;

    let authorizations = DBOAuthClientAuthorization::get_all_for_user(
        current_user.id.into(),
        &**pool,
    )
    .await?;

    let mapped: Vec<models::oauth_clients::OAuthClientAuthorization> =
        authorizations.into_iter().map(|a| a.into()).collect_vec();

    Ok(HttpResponse::Ok().json(mapped))
}

#[delete("authorizations")]
pub async fn revoke_oauth_authorization(
    req: HttpRequest,
    info: web::Query<DeleteOAuthClientQueryParam>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::SESSION_ACCESS]),
    )
    .await?
    .1;

    DBOAuthClientAuthorization::remove(
        info.client_id.into(),
        current_user.id.into(),
        &**pool,
    )
    .await?;

    Ok(HttpResponse::Ok().body(""))
}

fn generate_oauth_client_secret() -> String {
    ChaCha20Rng::from_entropy()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect::<String>()
}

async fn create_redirect_uris(
    uri_strings: impl IntoIterator<Item = impl Display>,
    client_id: DBOAuthClientId,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<Vec<DBOAuthRedirectUri>, DatabaseError> {
    let mut redirect_uris = vec![];
    for uri in uri_strings.into_iter() {
        let id = generate_oauth_redirect_id(transaction).await?;
        redirect_uris.push(DBOAuthRedirectUri {
            id,
            client_id,
            uri: uri.to_string(),
        });
    }

    Ok(redirect_uris)
}

async fn edit_redirects(
    redirects: Vec<String>,
    existing_client: &DBOAuthClient,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), DatabaseError> {
    let updated_redirects: HashSet<String> = redirects.into_iter().collect();
    let original_redirects: HashSet<String> = existing_client
        .redirect_uris
        .iter()
        .map(|r| r.uri.to_string())
        .collect();

    let redirects_to_add = create_redirect_uris(
        updated_redirects.difference(&original_redirects),
        existing_client.id,
        &mut *transaction,
    )
    .await?;
    DBOAuthClient::insert_redirect_uris(&redirects_to_add, &mut **transaction)
        .await?;

    let mut redirects_to_remove = existing_client.redirect_uris.clone();
    redirects_to_remove.retain(|r| !updated_redirects.contains(&r.uri));
    DBOAuthClient::remove_redirect_uris(
        redirects_to_remove.iter().map(|r| r.id),
        &mut **transaction,
    )
    .await?;

    Ok(())
}

pub async fn get_clients_inner(
    ids: &[OAuthClientId],
    pool: web::Data<PgPool>,
) -> Result<Vec<models::oauth_clients::OAuthClient>, ApiError> {
    let ids: Vec<DBOAuthClientId> = ids.iter().map(|i| (*i).into()).collect();
    let clients = DBOAuthClient::get_many(&ids, &**pool).await?;

    Ok(clients.into_iter().map(|c| c.into()).collect_vec())
}
