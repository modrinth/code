use super::ids::*;
use crate::auth::oauth::uris::OAuthRedirectUris;
use crate::auth::AuthProvider;
use crate::database::models::DatabaseError;
use crate::database::redis::RedisPool;
use crate::models::pats::Scopes;
use chrono::Duration;
use rand::distributions::Alphanumeric;
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};

const FLOWS_NAMESPACE: &str = "flows";

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Flow {
    OAuth {
        user_id: Option<UserId>,
        url: Option<String>,
        provider: AuthProvider,
    },
    Login2FA {
        user_id: UserId,
    },
    Initialize2FA {
        user_id: UserId,
        secret: String,
    },
    ForgotPassword {
        user_id: UserId,
    },
    ConfirmEmail {
        user_id: UserId,
        confirm_email: String,
    },
    MinecraftAuth,
    InitOAuthAppApproval {
        user_id: UserId,
        client_id: OAuthClientId,
        existing_authorization_id: Option<OAuthClientAuthorizationId>,
        scopes: Scopes,
        redirect_uris: OAuthRedirectUris,
        state: Option<String>,
    },
    OAuthAuthorizationCodeSupplied {
        user_id: UserId,
        client_id: OAuthClientId,
        authorization_id: OAuthClientAuthorizationId,
        scopes: Scopes,
        original_redirect_uri: Option<String>, // Needed for https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.3
    },
}

impl Flow {
    pub async fn insert(
        &self,
        expires: Duration,
        redis: &RedisPool,
    ) -> Result<String, DatabaseError> {
        let mut redis = redis.connect().await?;

        let flow = ChaCha20Rng::from_entropy()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect::<String>();

        redis
            .set_serialized_to_json(
                FLOWS_NAMESPACE,
                &flow,
                &self,
                Some(expires.num_seconds()),
            )
            .await?;
        Ok(flow)
    }

    pub async fn get(
        id: &str,
        redis: &RedisPool,
    ) -> Result<Option<Flow>, DatabaseError> {
        let mut redis = redis.connect().await?;

        redis.get_deserialized_from_json(FLOWS_NAMESPACE, id).await
    }

    /// Gets the flow and removes it from the cache, but only removes if the flow was present and the predicate returned true
    /// The predicate should validate that the flow being removed is the correct one, as a security measure
    pub async fn take_if(
        id: &str,
        predicate: impl FnOnce(&Flow) -> bool,
        redis: &RedisPool,
    ) -> Result<Option<Flow>, DatabaseError> {
        let flow = Self::get(id, redis).await?;
        if let Some(flow) = flow.as_ref() {
            if predicate(flow) {
                Self::remove(id, redis).await?;
            }
        }
        Ok(flow)
    }

    pub async fn remove(
        id: &str,
        redis: &RedisPool,
    ) -> Result<Option<()>, DatabaseError> {
        let mut redis = redis.connect().await?;

        redis.delete(FLOWS_NAMESPACE, id).await?;
        Ok(Some(()))
    }
}
