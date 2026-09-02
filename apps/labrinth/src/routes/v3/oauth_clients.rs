use crate::util::error::ApiContext as _;
use crate::util::error::Context as _;
use std::{collections::HashSet, fmt::Display};
use xredis::RedisPool;

use super::ApiError;
use crate::database::{PgPool, PgTransaction};
use crate::file_hosting::FileHostPublicity;
use crate::models::ids::OAuthClientId;
use crate::util::img::{delete_old_images, upload_image_optimized};
use crate::{
    auth::{checks::ValidateAuthorized, get_user_from_headers},
    database::models::{
        DBOAuthClientId, DBUser, DatabaseError, generate_oauth_client_id,
        generate_oauth_redirect_id,
        oauth_client_authorization_item::DBOAuthClientAuthorization,
        oauth_client_item::{DBOAuthClient, DBOAuthRedirectUri},
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
    util::routes::read_limited_from_payload,
};
use actix_web::{HttpRequest, HttpResponse, delete, get, patch, post, web};
use ariadne::ids::base62_impl::parse_base62;
use chrono::Utc;
use itertools::Itertools;
use rand::{Rng, SeedableRng, distributions::Alphanumeric};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use validator::Validate;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        web::scope("/oauth")
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

#[utoipa::path(
	context_path = "/oauth",
	tag = "oauth clients", responses((status = OK))
)]
#[get("/user/{id}/oauth_apps")]
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
        Scopes::SESSION_ACCESS,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;

    let target_user = DBUser::get(&info.into_inner(), &**pool, &redis)
        .await
        .wrap_internal_err("fetching user from database")?;

    if let Some(target_user) = target_user {
        if target_user.id != current_user.id.into()
            && !current_user.role.is_admin()
        {
            return Err(ApiError::Auth(eyre::eyre!(
                "You do not have permission to see the OAuth clients of this user!",
            )));
        }

        let clients =
            DBOAuthClient::get_all_user_clients(target_user.id, &**pool)
                .await
                .wrap_internal_err("fetching OAuth clients from database")?;

        let response = clients
            .into_iter()
            .map(models::oauth_clients::OAuthClient::from)
            .collect_vec();

        Ok(HttpResponse::Ok().json(response))
    } else {
        Err(ApiError::NotFound(eyre::eyre!("resource not found")))
    }
}

/// Get an OAuth client.  
#[utoipa::path(
	context_path = "/oauth",
	tag = "oauth clients",
	params(("id" = OAuthClientId, Path)),
	responses((status = OK, body = models::oauth_clients::OAuthClient)),
)]
#[get("/app/{id}")]
pub async fn get_client(
    id: web::Path<OAuthClientId>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let clients = get_clients_inner(&[id.into_inner()], pool)
        .await
        .wrap_api_err("fetching OAuth client")?;
    if let Some(client) = clients.into_iter().next() {
        Ok(HttpResponse::Ok().json(client))
    } else {
        Err(ApiError::NotFound(eyre::eyre!("resource not found")))
    }
}

/// List OAuth clients.  
#[utoipa::path(
	context_path = "/oauth",
	tag = "oauth clients",
	params(("ids" = Vec<String>, Query)),
	responses((status = OK, body = Vec<models::oauth_clients::OAuthClient>)),
)]
#[get("/apps")]
pub async fn get_clients(
    info: web::Query<GetOAuthClientsRequest>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let ids: Vec<_> = info
        .ids
        .iter()
        .map(|id| parse_base62(id).map(OAuthClientId))
        .collect::<Result<_, _>>()
        .wrap_request_err("parsing OAuth client IDs")?;

    let clients = get_clients_inner(&ids, pool)
        .await
        .wrap_api_err("fetching clients inner")?;

    Ok(HttpResponse::Ok().json(clients))
}

#[derive(Deserialize, Validate, utoipa::ToSchema)]
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

/// Create an OAuth client.  
#[utoipa::path(
	context_path = "/oauth",
	tag = "oauth clients",
	responses((status = OK, body = OAuthClientCreationResult)),
)]
#[post("/app")]
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
        Scopes::SESSION_ACCESS,
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

/// Delete an OAuth client.  
#[utoipa::path(
	context_path = "/oauth",
	tag = "oauth clients",
	params(("id" = OAuthClientId, Path)),
	responses((status = NO_CONTENT))
)]
#[delete("/app/{id}")]
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
        Scopes::SESSION_ACCESS,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;

    let client = DBOAuthClient::get(client_id.into_inner().into(), &**pool)
        .await
        .wrap_internal_err("fetching OAuth client from database")?;
    if let Some(client) = client {
        client
            .validate_authorized(Some(&current_user))
            .wrap_api_err("validating authorized")?;
        DBOAuthClient::remove(client.id, &**pool)
            .await
            .wrap_internal_err("validating oauth client")?;

        Ok(HttpResponse::NoContent().body(""))
    } else {
        Err(ApiError::NotFound(eyre::eyre!("resource not found")))
    }
}

#[derive(Serialize, Deserialize, Validate, utoipa::ToSchema)]
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

/// Update an OAuth client.  
#[utoipa::path(
	context_path = "/oauth",
	tag = "oauth clients",
	params(("id" = OAuthClientId, Path)),
	responses((status = NO_CONTENT))
)]
#[patch("/app/{id}")]
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
        Scopes::SESSION_ACCESS,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;

    client_updates
        .validate()
        .map_err(|err| eyre::eyre!(err))
        .wrap_request_err("validating request")?;

    if let Some(existing_client) =
        DBOAuthClient::get(client_id.into_inner().into(), &**pool)
            .await
            .wrap_internal_err("fetching OAuth client from database")?
    {
        existing_client
            .validate_authorized(Some(&current_user))
            .wrap_api_err("authorizing OAuth client update")?;

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

        let mut transaction = pool
            .begin()
            .await
            .wrap_internal_err("starting database transaction")?;
        updated_client
            .update_editable_fields(&mut transaction)
            .await
            .wrap_internal_err(
                "updating database records for `oauth_client_edit`",
            )?;

        if let Some(redirects) = redirect_uris {
            edit_redirects(redirects, &existing_client, &mut transaction)
                .await
                .wrap_internal_err("updating redirects in database")?;
        }

        transaction
            .commit()
            .await
            .wrap_internal_err("committing database transaction")?;

        Ok(HttpResponse::Ok().body(""))
    } else {
        Err(ApiError::NotFound(eyre::eyre!("resource not found")))
    }
}

#[derive(Serialize, Deserialize)]
pub struct Extension {
    pub ext: String,
}

/// Update an OAuth client icon.  
#[utoipa::path(
	context_path = "/oauth",
	tag = "oauth clients",
	params(
		("id" = OAuthClientId, Path),
		("ext" = String, Query)
	),
	request_body(content = Vec<u8>, content_type = "application/octet-stream"),
	responses((status = NO_CONTENT))
)]
#[patch("/app/{id}/icon")]
#[allow(clippy::too_many_arguments)]
pub async fn oauth_client_icon_edit(
    web::Query(ext): web::Query<Extension>,
    req: HttpRequest,
    client_id: web::Path<OAuthClientId>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<dyn FileHost>,
    mut payload: web::Payload,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;

    let client = DBOAuthClient::get((*client_id).into(), &**pool)
        .await
        .wrap_internal_err("fetching OAuth client from database")?
        .wrap_request_err_with(|| {
            "the specified client does not exist!".to_string()
        })?;

    client
        .validate_authorized(Some(&user))
        .wrap_api_err("validating authorized")?;

    delete_old_images(
        client.icon_url.clone(),
        client.raw_icon_url.clone(),
        FileHostPublicity::Public,
        &**file_host,
    )
    .await
    .wrap_api_err("deleting old images")?;

    let bytes = read_limited_from_payload(
        &mut payload,
        262144,
        "Icons must be smaller than 256KiB",
    )
    .await
    .wrap_api_err("executing `read_limited_from_payload`")?;
    let upload_result = upload_image_optimized(
        &format!("data/{client_id}"),
        FileHostPublicity::Public,
        bytes.freeze(),
        &ext.ext,
        Some(96),
        Some(1.0),
        &**file_host,
    )
    .await
    .wrap_api_err("uploading image")?;

    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("starting database transaction")?;

    let mut editable_client = client.clone();
    editable_client.icon_url = Some(upload_result.url);
    editable_client.raw_icon_url = Some(upload_result.raw_url);

    editable_client
        .update_editable_fields(&mut transaction)
        .await
        .wrap_internal_err(
            "updating database records for `oauth_client_icon_edit`",
        )?;

    transaction
        .commit()
        .await
        .wrap_internal_err("committing database transaction")?;

    Ok(HttpResponse::NoContent().body(""))
}

/// Delete an OAuth client icon.  
#[utoipa::path(
	context_path = "/oauth",
	tag = "oauth clients",
	params(("id" = OAuthClientId, Path)),
	responses((status = NO_CONTENT))
)]
#[delete("/app/{id}/icon")]
pub async fn oauth_client_icon_delete(
    req: HttpRequest,
    client_id: web::Path<OAuthClientId>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<dyn FileHost>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;

    let client = DBOAuthClient::get((*client_id).into(), &**pool)
        .await
        .wrap_internal_err("fetching OAuth client from database")?
        .wrap_request_err_with(|| {
            "the specified client does not exist!".to_string()
        })?;
    client
        .validate_authorized(Some(&user))
        .wrap_api_err("validating authorized")?;

    delete_old_images(
        client.icon_url.clone(),
        client.raw_icon_url.clone(),
        FileHostPublicity::Public,
        &**file_host,
    )
    .await
    .wrap_api_err("deleting old images")?;

    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("starting database transaction")?;

    let mut editable_client = client.clone();
    editable_client.icon_url = None;
    editable_client.raw_icon_url = None;

    editable_client
        .update_editable_fields(&mut transaction)
        .await
        .wrap_internal_err(
            "updating database records for `oauth_client_icon_delete`",
        )?;
    transaction
        .commit()
        .await
        .wrap_internal_err("committing database transaction")?;

    Ok(HttpResponse::NoContent().body(""))
}

/// List OAuth authorizations.  
#[utoipa::path(
	context_path = "/oauth",
	tag = "oauth clients",
	responses((status = OK, body = Vec<models::oauth_clients::OAuthClientAuthorization>)),
)]
#[get("/authorizations")]
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
        Scopes::SESSION_ACCESS,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;

    let authorizations = DBOAuthClientAuthorization::get_all_for_user(
        current_user.id.into(),
        &**pool,
    )
    .await
    .wrap_internal_err("fetching OAuth client authorizations from database")?;

    let mapped: Vec<models::oauth_clients::OAuthClientAuthorization> =
        authorizations.into_iter().map(|a| a.into()).collect_vec();

    Ok(HttpResponse::Ok().json(mapped))
}

/// Revoke OAuth authorization.  
#[utoipa::path(
	context_path = "/oauth",
	tag = "oauth clients",
	params(("client_id" = OAuthClientId, Query)),
	responses((status = NO_CONTENT))
)]
#[delete("/authorizations")]
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
        Scopes::SESSION_ACCESS,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;

    DBOAuthClientAuthorization::remove(
        info.client_id.into(),
        current_user.id.into(),
        &**pool,
    )
    .await
    .wrap_internal_err("deleting oauth client authorization from database")?;

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
    transaction: &mut PgTransaction<'_>,
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
    transaction: &mut PgTransaction<'_>,
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
    DBOAuthClient::insert_redirect_uris(&redirects_to_add, &mut *transaction)
        .await?;

    let mut redirects_to_remove = existing_client.redirect_uris.clone();
    redirects_to_remove.retain(|r| !updated_redirects.contains(&r.uri));
    DBOAuthClient::remove_redirect_uris(
        redirects_to_remove.iter().map(|r| r.id),
        &mut *transaction,
    )
    .await?;

    Ok(())
}

pub async fn get_clients_inner(
    ids: &[OAuthClientId],
    pool: web::Data<PgPool>,
) -> Result<Vec<models::oauth_clients::OAuthClient>, ApiError> {
    let ids: Vec<DBOAuthClientId> = ids.iter().map(|i| (*i).into()).collect();
    let clients = DBOAuthClient::get_many(&ids, &**pool)
        .await
        .wrap_internal_err("fetching OAuth clients from database")?;

    Ok(clients.into_iter().map(|c| c.into()).collect_vec())
}
