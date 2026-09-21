use super::ids::*;
use crate::auth::oauth::uris::OAuthRedirectUris;
use crate::models::pats::Scopes;
use crate::{auth::AuthProvider, routes::internal::flows::TempUser};
use chrono::Duration;
use eyre::{Result, WrapErr};
use rand::Rng;
use rand::distributions::Alphanumeric;
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;
use serde_binhum::serde_binhum;
use url::Url;
use webauthn_rs::prelude::{DiscoverableAuthentication, PasskeyRegistration};
use xredis::RedisPool;

const FLOWS_NAMESPACE: &str = "flows:v4";

#[serde_binhum]
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
    RegisterPasskey {
        user_id: DBUserId,
        #[serde_binhum(binary(with = "json_string"))]
        state: PasskeyRegistration,
    },
    AuthenticatePasskey {
        #[serde_binhum(binary(with = "json_string"))]
        state: DiscoverableAuthentication,
    },
}

mod json_string {
    use serde::de::DeserializeOwned;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Serialize,
        S: Serializer,
    {
        let value =
            serde_json::to_string(value).map_err(serde::ser::Error::custom)?;
        value.serialize(serializer)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: DeserializeOwned,
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        serde_json::from_str(&value).map_err(serde::de::Error::custom)
    }
}

impl DBFlow {
    pub async fn insert_with_state(
        &self,
        expires: Duration,
        redis: &RedisPool,
        state: &str,
    ) -> Result<()> {
        let mut redis = redis
            .connect()
            .await
            .wrap_err("connecting to redis to insert flow")?;
        let key = redis.key().entity(FLOWS_NAMESPACE, state);

        redis
            .set_serialized(&key, &self, Some(expires.num_seconds()))
            .await
            .wrap_err("inserting flow into redis")?;
        Ok(())
    }

    pub async fn insert(
        &self,
        expires: Duration,
        redis: &RedisPool,
    ) -> Result<String> {
        let state = ChaCha20Rng::from_entropy()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect::<String>();

        self.insert_with_state(expires, redis, &state)
            .await
            .wrap_err("inserting flow with generated state")?;
        Ok(state)
    }

    pub async fn get(id: &str, redis: &RedisPool) -> Result<Option<DBFlow>> {
        let mut redis = redis
            .connect()
            .await
            .wrap_err("connecting to redis to get flow")?;
        let key = redis.key().entity(FLOWS_NAMESPACE, id);

        redis
            .get_deserialized(&key)
            .await
            .wrap_err("getting flow from redis")
    }

    /// Gets the flow and removes it from the cache, but only removes if the flow was present and the predicate returned true
    /// The predicate should validate that the flow being removed is the correct one, as a security measure
    pub async fn take_if(
        id: &str,
        predicate: impl FnOnce(&DBFlow) -> bool,
        redis: &RedisPool,
    ) -> Result<Option<DBFlow>> {
        let flow = Self::get(id, redis)
            .await
            .wrap_err("getting flow before conditional removal")?;
        if let Some(flow) = flow.as_ref()
            && predicate(flow)
        {
            Self::remove(id, redis)
                .await
                .wrap_err("removing flow after predicate matched")?;
        }
        Ok(flow)
    }

    pub async fn remove(id: &str, redis: &RedisPool) -> Result<Option<()>> {
        let mut redis = redis
            .connect()
            .await
            .wrap_err("connecting to redis to remove flow")?;
        let key = redis.key().entity(FLOWS_NAMESPACE, id);

        redis
            .delete(&key)
            .await
            .wrap_err("removing flow from redis")?;
        Ok(Some(()))
    }
}
