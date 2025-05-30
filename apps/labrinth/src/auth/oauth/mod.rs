use crate::auth::get_user_from_headers;
use crate::auth::oauth::uris::{OAuthRedirectUris, ValidatedRedirectUri};
use crate::auth::validate::extract_authorization_header;
use crate::database::models::flow_item::DBFlow;
use crate::database::models::oauth_client_authorization_item::DBOAuthClientAuthorization;
use crate::database::models::oauth_client_item::DBOAuthClient;
use crate::database::models::oauth_token_item::DBOAuthAccessToken;
use crate::database::models::{
    DBOAuthClientAuthorizationId, generate_oauth_access_token_id,
    generate_oauth_client_authorization_id,
};
use crate::database::redis::RedisPool;
use crate::models;
use crate::models::ids::OAuthClientId;
use crate::models::pats::Scopes;
use crate::queue::session::AuthQueue;
use actix_web::http::header::{CACHE_CONTROL, LOCATION, PRAGMA};
use actix_web::web::{Data, Query, ServiceConfig};
use actix_web::{HttpRequest, HttpResponse, get, post, web};
use chrono::Duration;
use rand::distributions::Alphanumeric;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

use self::errors::{OAuthError, OAuthErrorType};

use super::AuthenticationError;

pub mod errors;
pub mod uris;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(init_oauth)
        .service(accept_client_scopes)
        .service(reject_client_scopes)
        .service(request_token);
}

#[derive(Serialize, Deserialize)]
pub struct OAuthInit {
    pub client_id: OAuthClientId,
    pub redirect_uri: Option<String>,
    pub scope: Option<String>,
    pub state: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct OAuthClientAccessRequest {
    pub flow_id: String,
    pub client_id: OAuthClientId,
    pub client_name: String,
    pub client_icon: Option<String>,
    pub requested_scopes: Scopes,
}

#[get("authorize")]
pub async fn init_oauth(
    req: HttpRequest,
    Query(oauth_info): Query<OAuthInit>,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, OAuthError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::USER_AUTH_WRITE]),
    )
    .await?
    .1;

    let client_id = oauth_info.client_id.into();
    let client = DBOAuthClient::get(client_id, &**pool).await?;

    if let Some(client) = client {
        let redirect_uri = ValidatedRedirectUri::validate(
            &oauth_info.redirect_uri,
            client.redirect_uris.iter().map(|r| r.uri.as_ref()),
            client.id,
        )?;

        let requested_scopes =
            oauth_info
                .scope
                .as_ref()
                .map_or(Ok(client.max_scopes), |s| {
                    Scopes::parse_from_oauth_scopes(s).map_err(|e| {
                        OAuthError::redirect(
                            OAuthErrorType::FailedScopeParse(e),
                            &oauth_info.state,
                            &redirect_uri,
                        )
                    })
                })?;

        if !client.max_scopes.contains(requested_scopes) {
            return Err(OAuthError::redirect(
                OAuthErrorType::ScopesTooBroad,
                &oauth_info.state,
                &redirect_uri,
            ));
        }

        let existing_authorization =
            DBOAuthClientAuthorization::get(client.id, user.id.into(), &**pool)
                .await
                .map_err(|e| {
                    OAuthError::redirect(e, &oauth_info.state, &redirect_uri)
                })?;
        let redirect_uris = OAuthRedirectUris {
            original: oauth_info.redirect_uri.clone(),
            validated: redirect_uri.clone(),
        };
        match existing_authorization {
            Some(existing_authorization)
                if existing_authorization.scopes.contains(requested_scopes) =>
            {
                init_oauth_code_flow(
                    user.id.into(),
                    client.id.into(),
                    existing_authorization.id,
                    requested_scopes,
                    redirect_uris,
                    oauth_info.state,
                    &redis,
                )
                .await
            }
            _ => {
                let flow_id = DBFlow::InitOAuthAppApproval {
                    user_id: user.id.into(),
                    client_id: client.id,
                    existing_authorization_id: existing_authorization
                        .map(|a| a.id),
                    scopes: requested_scopes,
                    redirect_uris,
                    state: oauth_info.state.clone(),
                }
                .insert(Duration::minutes(30), &redis)
                .await
                .map_err(|e| {
                    OAuthError::redirect(e, &oauth_info.state, &redirect_uri)
                })?;

                let access_request = OAuthClientAccessRequest {
                    client_id: client.id.into(),
                    client_name: client.name,
                    client_icon: client.icon_url,
                    flow_id,
                    requested_scopes,
                };
                Ok(HttpResponse::Ok().json(access_request))
            }
        }
    } else {
        Err(OAuthError::error(OAuthErrorType::InvalidClientId(
            client_id,
        )))
    }
}

#[derive(Serialize, Deserialize)]
pub struct RespondToOAuthClientScopes {
    pub flow: String,
}

#[post("accept")]
pub async fn accept_client_scopes(
    req: HttpRequest,
    accept_body: web::Json<RespondToOAuthClientScopes>,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, OAuthError> {
    accept_or_reject_client_scopes(
        true,
        req,
        accept_body,
        pool,
        redis,
        session_queue,
    )
    .await
}

#[post("reject")]
pub async fn reject_client_scopes(
    req: HttpRequest,
    body: web::Json<RespondToOAuthClientScopes>,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, OAuthError> {
    accept_or_reject_client_scopes(false, req, body, pool, redis, session_queue)
        .await
}

#[derive(Serialize, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub code: String,
    pub redirect_uri: Option<String>,
    pub client_id: models::ids::OAuthClientId,
}

#[derive(Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[post("token")]
/// Params should be in the urlencoded request body
/// And client secret should be in the HTTP basic authorization header
/// Per IETF RFC6749 Section 4.1.3 (https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.3)
pub async fn request_token(
    req: HttpRequest,
    req_params: web::Form<TokenRequest>,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
) -> Result<HttpResponse, OAuthError> {
    let req_client_id = req_params.client_id;
    let client = DBOAuthClient::get(req_client_id.into(), &**pool).await?;
    if let Some(client) = client {
        authenticate_client_token_request(&req, &client)?;

        // Ensure auth code is single use
        // per IETF RFC6749 Section 10.5 (https://datatracker.ietf.org/doc/html/rfc6749#section-10.5)
        let flow = DBFlow::take_if(
            &req_params.code,
            |f| matches!(f, DBFlow::OAuthAuthorizationCodeSupplied { .. }),
            &redis,
        )
        .await?;
        if let Some(DBFlow::OAuthAuthorizationCodeSupplied {
            user_id,
            client_id,
            authorization_id,
            scopes,
            original_redirect_uri,
        }) = flow
        {
            // https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.3
            if req_client_id != client_id.into() {
                return Err(OAuthError::error(
                    OAuthErrorType::UnauthorizedClient,
                ));
            }

            if original_redirect_uri != req_params.redirect_uri {
                return Err(OAuthError::error(
                    OAuthErrorType::RedirectUriChanged(
                        req_params.redirect_uri.clone(),
                    ),
                ));
            }

            if req_params.grant_type != "authorization_code" {
                return Err(OAuthError::error(
                    OAuthErrorType::OnlySupportsAuthorizationCodeGrant(
                        req_params.grant_type.clone(),
                    ),
                ));
            }

            let scopes = scopes - Scopes::restricted();

            let mut transaction = pool.begin().await?;
            let token_id =
                generate_oauth_access_token_id(&mut transaction).await?;
            let token = generate_access_token();
            let token_hash = DBOAuthAccessToken::hash_token(&token);
            let time_until_expiration = DBOAuthAccessToken {
                id: token_id,
                authorization_id,
                token_hash,
                scopes,
                created: Default::default(),
                expires: Default::default(),
                last_used: None,
                client_id,
                user_id,
            }
            .insert(&mut *transaction)
            .await?;

            transaction.commit().await?;

            // IETF RFC6749 Section 5.1 (https://datatracker.ietf.org/doc/html/rfc6749#section-5.1)
            Ok(HttpResponse::Ok()
                .append_header((CACHE_CONTROL, "no-store"))
                .append_header((PRAGMA, "no-cache"))
                .json(TokenResponse {
                    access_token: token,
                    token_type: "Bearer".to_string(),
                    expires_in: time_until_expiration.num_seconds(),
                }))
        } else {
            Err(OAuthError::error(OAuthErrorType::InvalidAuthCode))
        }
    } else {
        Err(OAuthError::error(OAuthErrorType::InvalidClientId(
            req_client_id.into(),
        )))
    }
}

pub async fn accept_or_reject_client_scopes(
    accept: bool,
    req: HttpRequest,
    body: web::Json<RespondToOAuthClientScopes>,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, OAuthError> {
    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Some(&[Scopes::SESSION_ACCESS]),
    )
    .await?
    .1;

    let flow = DBFlow::take_if(
        &body.flow,
        |f| matches!(f, DBFlow::InitOAuthAppApproval { .. }),
        &redis,
    )
    .await?;
    if let Some(DBFlow::InitOAuthAppApproval {
        user_id,
        client_id,
        existing_authorization_id,
        scopes,
        redirect_uris,
        state,
    }) = flow
    {
        if current_user.id != user_id.into() {
            return Err(OAuthError::error(
                AuthenticationError::InvalidCredentials,
            ));
        }

        if accept {
            let mut transaction = pool.begin().await?;

            let auth_id = match existing_authorization_id {
                Some(id) => id,
                None => {
                    generate_oauth_client_authorization_id(&mut transaction)
                        .await?
                }
            };
            DBOAuthClientAuthorization::upsert(
                auth_id,
                client_id,
                user_id,
                scopes,
                &mut transaction,
            )
            .await?;

            transaction.commit().await?;

            init_oauth_code_flow(
                user_id,
                client_id.into(),
                auth_id,
                scopes,
                redirect_uris,
                state,
                &redis,
            )
            .await
        } else {
            Err(OAuthError::redirect(
                OAuthErrorType::AccessDenied,
                &state,
                &redirect_uris.validated,
            ))
        }
    } else {
        Err(OAuthError::error(OAuthErrorType::InvalidAcceptFlowId))
    }
}

fn authenticate_client_token_request(
    req: &HttpRequest,
    client: &DBOAuthClient,
) -> Result<(), OAuthError> {
    let client_secret = extract_authorization_header(req)?;
    let hashed_client_secret = DBOAuthClient::hash_secret(client_secret);
    if client.secret_hash != hashed_client_secret {
        Err(OAuthError::error(
            OAuthErrorType::ClientAuthenticationFailed,
        ))
    } else {
        Ok(())
    }
}

fn generate_access_token() -> String {
    let random = ChaCha20Rng::from_entropy()
        .sample_iter(&Alphanumeric)
        .take(60)
        .map(char::from)
        .collect::<String>();
    format!("mro_{random}")
}

async fn init_oauth_code_flow(
    user_id: crate::database::models::DBUserId,
    client_id: OAuthClientId,
    authorization_id: DBOAuthClientAuthorizationId,
    scopes: Scopes,
    redirect_uris: OAuthRedirectUris,
    state: Option<String>,
    redis: &RedisPool,
) -> Result<HttpResponse, OAuthError> {
    let code = DBFlow::OAuthAuthorizationCodeSupplied {
        user_id,
        client_id: client_id.into(),
        authorization_id,
        scopes,
        original_redirect_uri: redirect_uris.original.clone(),
    }
    .insert(Duration::minutes(10), redis)
    .await
    .map_err(|e| {
        OAuthError::redirect(e, &state, &redirect_uris.validated.clone())
    })?;

    let mut redirect_params = vec![format!("code={code}")];
    if let Some(state) = state {
        redirect_params.push(format!("state={state}"));
    }

    let redirect_uri =
        append_params_to_uri(&redirect_uris.validated.0, &redirect_params);

    // IETF RFC 6749 Section 4.1.2 (https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.2)
    Ok(HttpResponse::Ok()
        .append_header((LOCATION, redirect_uri.clone()))
        .body(redirect_uri))
}

fn append_params_to_uri(uri: &str, params: &[impl AsRef<str>]) -> String {
    let mut uri = uri.to_string();
    let mut connector = if uri.contains('?') { "&" } else { "?" };
    for param in params {
        uri.push_str(&format!("{}{}", connector, param.as_ref()));
        connector = "&";
    }

    uri
}
