use std::{collections::HashSet, fmt::Display, sync::Arc};

use actix_web::{
    delete, get, patch, post,
    web::{self, scope},
    HttpRequest, HttpResponse,
};
use chrono::Utc;
use itertools::Itertools;
use rand::{distributions::Alphanumeric, Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use validator::Validate;

use super::ApiError;
use crate::{
    auth::checks::ValidateAllAuthorized,
    file_hosting::FileHost,
    models::{ids::base62_impl::parse_base62, oauth_clients::DeleteOAuthClientQueryParam},
    util::routes::read_from_payload,
};
use crate::{
    auth::{checks::ValidateAuthorized, get_user_from_headers},
    database::{
        models::{
            generate_oauth_client_id, generate_oauth_redirect_id,
            oauth_client_authorization_item::OAuthClientAuthorization,
            oauth_client_item::{OAuthClient, OAuthRedirectUri},
            DatabaseError, OAuthClientId, User,
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

use crate::database::models::oauth_client_item::OAuthClient as DBOAuthClient;
use crate::models::ids::OAuthClientId as ApiOAuthClientId;

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

    let target_user = User::get(&info.into_inner(), &**pool, &redis).await?;

    if let Some(target_user) = target_user {
        let clients = OAuthClient::get_all_user_clients(target_user.id, &**pool).await?;
        clients
            .iter()
            .validate_all_authorized(Some(&current_user))?;

        let response = clients
            .into_iter()
            .map(models::oauth_clients::OAuthClient::from)
            .collect_vec();

        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[get("app/{id}")]
pub async fn get_client(
    req: HttpRequest,
    id: web::Path<ApiOAuthClientId>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let clients = get_clients_inner(&[id.into_inner()], req, pool, redis, session_queue).await?;
    if let Some(client) = clients.into_iter().next() {
        Ok(HttpResponse::Ok().json(client))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[get("apps")]
pub async fn get_clients(
    req: HttpRequest,
    info: web::Query<GetOAuthClientsRequest>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let ids: Vec<_> = info
        .ids
        .iter()
        .map(|id| parse_base62(id).map(ApiOAuthClientId))
        .collect::<Result<_, _>>()?;

    let clients = get_clients_inner(&ids, req, pool, redis, session_queue).await?;

    Ok(HttpResponse::Ok().json(clients))
}

#[derive(Deserialize, Validate)]
pub struct NewOAuthApp {
    #[validate(
        custom(function = "crate::util::validate::validate_name"),
        length(min = 3, max = 255)
    )]
    pub name: String,

    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 255)
    )]
    pub icon_url: Option<String>,

    #[validate(custom(function = "crate::util::validate::validate_no_restricted_scopes"))]
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
pub async fn oauth_client_create<'a>(
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

    new_oauth_app
        .validate()
        .map_err(|e| CreateError::ValidationError(validation_errors_to_string(e, None)))?;

    let mut transaction = pool.begin().await?;

    let client_id = generate_oauth_client_id(&mut transaction).await?;

    let client_secret = generate_oauth_client_secret();
    let client_secret_hash = DBOAuthClient::hash_secret(&client_secret);

    let redirect_uris =
        create_redirect_uris(&new_oauth_app.redirect_uris, client_id, &mut transaction).await?;

    let client = OAuthClient {
        id: client_id,
        icon_url: new_oauth_app.icon_url.clone(),
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
pub async fn oauth_client_delete<'a>(
    req: HttpRequest,
    client_id: web::Path<ApiOAuthClientId>,
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

    let client = OAuthClient::get(client_id.into_inner().into(), &**pool).await?;
    if let Some(client) = client {
        client.validate_authorized(Some(&current_user))?;
        OAuthClient::remove(client.id, &**pool).await?;

        Ok(HttpResponse::NoContent().body(""))
    } else {
        Ok(HttpResponse::NotFound().body(""))
    }
}

#[derive(Serialize, Deserialize, Validate)]
pub struct OAuthClientEdit {
    #[validate(
        custom(function = "crate::util::validate::validate_name"),
        length(min = 3, max = 255)
    )]
    pub name: Option<String>,

    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 255)
    )]
    pub icon_url: Option<Option<String>>,

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
    client_id: web::Path<ApiOAuthClientId>,
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

    client_updates
        .validate()
        .map_err(|e| ApiError::Validation(validation_errors_to_string(e, None)))?;

    if client_updates.icon_url.is_none()
        && client_updates.name.is_none()
        && client_updates.max_scopes.is_none()
    {
        return Err(ApiError::InvalidInput("No changes provided".to_string()));
    }

    if let Some(existing_client) = OAuthClient::get(client_id.into_inner().into(), &**pool).await? {
        existing_client.validate_authorized(Some(&current_user))?;

        let mut updated_client = existing_client.clone();
        let OAuthClientEdit {
            name,
            icon_url,
            max_scopes,
            redirect_uris,
            url,
            description,
        } = client_updates.into_inner();
        if let Some(name) = name {
            updated_client.name = name;
        }

        if let Some(icon_url) = icon_url {
            updated_client.icon_url = icon_url;
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
            edit_redirects(redirects, &existing_client, &mut transaction).await?;
        }

        transaction.commit().await?;

        Ok(HttpResponse::Ok().body(""))
    } else {
        Ok(HttpResponse::NotFound().body(""))
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
    client_id: web::Path<ApiOAuthClientId>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    mut payload: web::Payload,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    if let Some(content_type) = crate::util::ext::get_image_content_type(&ext.ext) {
        let cdn_url = dotenvy::var("CDN_URL")?;
        let user = get_user_from_headers(
            &req,
            &**pool,
            &redis,
            &session_queue,
            Some(&[Scopes::SESSION_ACCESS]),
        )
        .await?
        .1;

        let client = OAuthClient::get((*client_id).into(), &**pool)
            .await?
            .ok_or_else(|| {
                ApiError::InvalidInput("The specified client does not exist!".to_string())
            })?;

        client.validate_authorized(Some(&user))?;

        if let Some(ref icon) = client.icon_url {
            let name = icon.split(&format!("{cdn_url}/")).nth(1);

            if let Some(icon_path) = name {
                file_host.delete_file_version("", icon_path).await?;
            }
        }

        let bytes =
            read_from_payload(&mut payload, 262144, "Icons must be smaller than 256KiB").await?;
        let hash = sha1::Sha1::from(&bytes).hexdigest();
        let upload_data = file_host
            .upload_file(
                content_type,
                &format!("data/{}/{}.{}", client_id, hash, ext.ext),
                bytes.freeze(),
            )
            .await?;

        let mut transaction = pool.begin().await?;

        let mut editable_client = client.clone();
        editable_client.icon_url = Some(format!("{}/{}", cdn_url, upload_data.file_name));

        editable_client
            .update_editable_fields(&mut *transaction)
            .await?;

        transaction.commit().await?;

        Ok(HttpResponse::NoContent().body(""))
    } else {
        Err(ApiError::InvalidInput(format!(
            "Invalid format for project icon: {}",
            ext.ext
        )))
    }
}

#[delete("app/{id}/icon")]
pub async fn oauth_client_icon_delete(
    req: HttpRequest,
    client_id: web::Path<ApiOAuthClientId>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<Arc<dyn FileHost + Send + Sync>>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let cdn_url = dotenvy::var("CDN_URL")?;
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::SESSION_ACCESS]),
    )
    .await?
    .1;

    let client = OAuthClient::get((*client_id).into(), &**pool)
        .await?
        .ok_or_else(|| {
            ApiError::InvalidInput("The specified client does not exist!".to_string())
        })?;
    client.validate_authorized(Some(&user))?;

    if let Some(ref icon) = client.icon_url {
        let name = icon.split(&format!("{cdn_url}/")).nth(1);

        if let Some(icon_path) = name {
            file_host.delete_file_version("", icon_path).await?;
        }
    }

    let mut transaction = pool.begin().await?;

    let mut editable_client = client.clone();
    editable_client.icon_url = None;

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

    let authorizations =
        OAuthClientAuthorization::get_all_for_user(current_user.id.into(), &**pool).await?;

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
    println!("Inside revoke_oauth_authorization");
    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::SESSION_ACCESS]),
    )
    .await?
    .1;

    OAuthClientAuthorization::remove(info.client_id.into(), current_user.id.into(), &**pool)
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
    client_id: OAuthClientId,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<Vec<OAuthRedirectUri>, DatabaseError> {
    let mut redirect_uris = vec![];
    for uri in uri_strings.into_iter() {
        let id = generate_oauth_redirect_id(transaction).await?;
        redirect_uris.push(OAuthRedirectUri {
            id,
            client_id,
            uri: uri.to_string(),
        });
    }

    Ok(redirect_uris)
}

async fn edit_redirects(
    redirects: Vec<String>,
    existing_client: &OAuthClient,
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
    OAuthClient::insert_redirect_uris(&redirects_to_add, &mut **transaction).await?;

    let mut redirects_to_remove = existing_client.redirect_uris.clone();
    redirects_to_remove.retain(|r| !updated_redirects.contains(&r.uri));
    OAuthClient::remove_redirect_uris(redirects_to_remove.iter().map(|r| r.id), &mut **transaction)
        .await?;

    Ok(())
}

pub async fn get_clients_inner(
    ids: &[ApiOAuthClientId],
    req: HttpRequest,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<Vec<models::oauth_clients::OAuthClient>, ApiError> {
    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::SESSION_ACCESS]),
    )
    .await?
    .1;

    let ids: Vec<OAuthClientId> = ids.iter().map(|i| (*i).into()).collect();
    let clients = OAuthClient::get_many(&ids, &**pool).await?;
    clients
        .iter()
        .validate_all_authorized(Some(&current_user))?;

    Ok(clients.into_iter().map(|c| c.into()).collect_vec())
}
