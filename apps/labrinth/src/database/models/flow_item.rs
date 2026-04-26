use super::ids::*;
use crate::auth::oauth::uris::OAuthRedirectUris;
use crate::database::models::DatabaseError;
use crate::database::redis::RedisPool;
use crate::models::pats::Scopes;
use crate::{auth::AuthProvider, routes::internal::flows::TempUser};
use chrono::Duration;
use rand::Rng;
use rand::distributions::Alphanumeric;
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;
use serde::{Deserialize, Serialize};
use url::Url;

const FLOWS_NAMESPACE: &str = "flows";

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DBFlow {
    OAuth {
        user_id: Option<DBUserId>,
        url: Url,
        provider: AuthProvider,
        existing_user_id: Option<DBUserId>,
    },
    OAuthPending {
        url: Url,
        provider: AuthProvider,
        user: TempUser,
    },
    Login2FA {
        user_id: DBUserId,
    },
    Initialize2FA {
        user_id: DBUserId,
        secret: String,
    },
    ForgotPassword {
        user_id: DBUserId,
    },
    ConfirmEmail {
        user_id: DBUserId,
        confirm_email: String,
    },
    MinecraftAuth,
    InitOAuthAppApproval {
        user_id: DBUserId,
        client_id: DBOAuthClientId,
        existing_authorization_id: Option<DBOAuthClientAuthorizationId>,
        scopes: Scopes,
        redirect_uris: OAuthRedirectUris,
        state: Option<String>,
    },
    OAuthAuthorizationCodeSupplied {
        user_id: DBUserId,
        client_id: DBOAuthClientId,
        authorization_id: DBOAuthClientAuthorizationId,
        scopes: Scopes,
        original_redirect_uri: Option<String>, // Needed for https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.3
    },
}

impl DBFlow {
    pub async fn insert_with_state(
        &self,
        expires: Duration,
        redis: &RedisPool,
        state: &str,
    ) -> Result<(), DatabaseError> {
        let mut redis = redis.connect().await?;

        redis
            .set_serialized_to_json(
                FLOWS_NAMESPACE,
                &state,
                &self,
                Some(expires.num_seconds()),
            )
            .await?;
        Ok(())
    }

    pub async fn insert(
        &self,
        expires: Duration,
        redis: &RedisPool,
    ) -> Result<String, DatabaseError> {
        let state = ChaCha20Rng::from_entropy()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect::<String>();

        self.insert_with_state(expires, redis, &state).await?;
        Ok(state)
    }

    pub async fn get(
        id: &str,
        redis: &RedisPool,
    ) -> Result<Option<DBFlow>, DatabaseError> {
        let mut redis = redis.connect().await?;

        redis.get_deserialized_from_json(FLOWS_NAMESPACE, id).await
    }

    /// Gets the flow and removes it from the cache, but only removes if the flow was present and the predicate returned true
    /// The predicate should validate that the flow being removed is the correct one, as a security measure
    pub async fn take_if(
        id: &str,
        predicate: impl FnOnce(&DBFlow) -> bool,
        redis: &RedisPool,
    ) -> Result<Option<DBFlow>, DatabaseError> {
        let flow = Self::get(id, redis).await?;
        if let Some(flow) = flow.as_ref()
            && predicate(flow)
        {
            Self::remove(id, redis).await?;
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
