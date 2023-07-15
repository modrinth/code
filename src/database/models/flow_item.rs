use super::ids::*;
use crate::auth::flows::AuthProvider;
use crate::database::models::DatabaseError;
use chrono::Duration;
use rand::distributions::Alphanumeric;
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha20Rng;
use redis::cmd;
use serde::{Deserialize, Serialize};

const FLOWS_NAMESPACE: &str = "flows";

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Flow {
    OAuth {
        user_id: Option<UserId>,
        url: String,
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
}

impl Flow {
    pub async fn insert(
        &self,
        expires: Duration,
        redis: &deadpool_redis::Pool,
    ) -> Result<String, DatabaseError> {
        let mut redis = redis.get().await?;

        let flow = ChaCha20Rng::from_entropy()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect::<String>();

        cmd("SET")
            .arg(format!("{}:{}", FLOWS_NAMESPACE, flow))
            .arg(serde_json::to_string(&self)?)
            .arg("EX")
            .arg(expires.num_seconds())
            .query_async::<_, ()>(&mut redis)
            .await?;

        Ok(flow)
    }

    pub async fn get(
        id: &str,
        redis: &deadpool_redis::Pool,
    ) -> Result<Option<Flow>, DatabaseError> {
        let mut redis = redis.get().await?;

        let res = cmd("GET")
            .arg(format!("{}:{}", FLOWS_NAMESPACE, id))
            .query_async::<_, Option<String>>(&mut redis)
            .await?;

        Ok(res.and_then(|x| serde_json::from_str(&x).ok()))
    }

    pub async fn remove(
        id: &str,
        redis: &deadpool_redis::Pool,
    ) -> Result<Option<()>, DatabaseError> {
        let mut redis = redis.get().await?;
        let mut cmd = cmd("DEL");
        cmd.arg(format!("{}:{}", FLOWS_NAMESPACE, id));
        cmd.query_async::<_, ()>(&mut redis).await?;

        Ok(Some(()))
    }
}
