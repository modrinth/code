use super::ids::*;
use crate::auth::flows::AuthProvider;
use crate::database::models::DatabaseError;
use crate::database::redis::RedisPool;
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
}

impl Flow {
    pub async fn insert(
        &self,
        expires: Duration,
        redis: &RedisPool,
    ) -> Result<String, DatabaseError> {
        let flow = ChaCha20Rng::from_entropy()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect::<String>();

        redis
            .set(
                FLOWS_NAMESPACE,
                &flow,
                serde_json::to_string(&self)?,
                Some(expires.num_seconds()),
            )
            .await?;
        Ok(flow)
    }

    pub async fn get(id: &str, redis: &RedisPool) -> Result<Option<Flow>, DatabaseError> {
        let res = redis.get::<String, _>(FLOWS_NAMESPACE, id).await?;
        Ok(res.and_then(|x| serde_json::from_str(&x).ok()))
    }

    pub async fn remove(id: &str, redis: &RedisPool) -> Result<Option<()>, DatabaseError> {
        redis.delete(FLOWS_NAMESPACE, id).await?;
        Ok(Some(()))
    }
}
