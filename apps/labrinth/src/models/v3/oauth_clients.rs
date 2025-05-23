use super::pats::Scopes;
use crate::database::models::oauth_client_authorization_item::DBOAuthClientAuthorization;
use crate::database::models::oauth_client_item::DBOAuthClient;
use crate::database::models::oauth_client_item::DBOAuthRedirectUri;
use crate::models::ids::{
    OAuthClientAuthorizationId, OAuthClientId, OAuthRedirectUriId,
};
use ariadne::ids::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[derive(Deserialize, Serialize)]
pub struct OAuthRedirectUri {
    pub id: OAuthRedirectUriId,
    pub client_id: OAuthClientId,
    pub uri: String,
}

#[derive(Serialize, Deserialize)]
pub struct OAuthClientCreationResult {
    #[serde(flatten)]
    pub client: OAuthClient,

    pub client_secret: String,
}

#[derive(Deserialize, Serialize)]
pub struct OAuthClient {
    pub id: OAuthClientId,
    pub name: String,
    pub icon_url: Option<String>,

    // The maximum scopes the client can request for OAuth
    pub max_scopes: Scopes,

    // The valid URIs that can be redirected to during an authorization request
    pub redirect_uris: Vec<OAuthRedirectUri>,

    // The user that created (and thus controls) this client
    pub created_by: UserId,

    // When this client was created
    pub created: DateTime<Utc>,

    // (optional) Metadata about the client
    pub url: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct OAuthClientAuthorization {
    pub id: OAuthClientAuthorizationId,
    pub app_id: OAuthClientId,
    pub user_id: UserId,
    pub scopes: Scopes,
    pub created: DateTime<Utc>,
}

#[serde_as]
#[derive(Deserialize, Serialize)]
pub struct GetOAuthClientsRequest {
    #[serde_as(
        as = "serde_with::StringWithSeparator::<serde_with::formats::CommaSeparator, String>"
    )]
    pub ids: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct DeleteOAuthClientQueryParam {
    pub client_id: OAuthClientId,
}

impl From<DBOAuthClient> for OAuthClient {
    fn from(value: DBOAuthClient) -> Self {
        Self {
            id: value.id.into(),
            name: value.name,
            icon_url: value.icon_url,
            max_scopes: value.max_scopes,
            redirect_uris: value
                .redirect_uris
                .into_iter()
                .map(|r| r.into())
                .collect(),
            created_by: value.created_by.into(),
            created: value.created,
            url: value.url,
            description: value.description,
        }
    }
}

impl From<DBOAuthRedirectUri> for OAuthRedirectUri {
    fn from(value: DBOAuthRedirectUri) -> Self {
        Self {
            id: value.id.into(),
            client_id: value.client_id.into(),
            uri: value.uri,
        }
    }
}

impl From<DBOAuthClientAuthorization> for OAuthClientAuthorization {
    fn from(value: DBOAuthClientAuthorization) -> Self {
        Self {
            id: value.id.into(),
            app_id: value.client_id.into(),
            user_id: value.user_id.into(),
            scopes: value.scopes,
            created: value.created,
        }
    }
}
