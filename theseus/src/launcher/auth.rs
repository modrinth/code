//! Authentication flow based on Hydra

use crate::hydra;
use crate::util::fetch::FetchSemaphore;

use chrono::{prelude::*, Duration};

use serde::{Deserialize, Serialize};

use crate::api::hydra::stages::{bearer_token, xbl_signin, xsts_token};

// Login information
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Credentials {
    pub id: uuid::Uuid,
    pub username: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires: DateTime<Utc>,
    _ctor_scope: std::marker::PhantomData<()>,
}

impl Credentials {
    pub fn new(
        id: uuid::Uuid,
        username: String,
        access_token: String,
        refresh_token: String,
        expires: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            username,
            access_token,
            refresh_token,
            expires,
            _ctor_scope: std::marker::PhantomData,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expires < Utc::now()
    }
}

pub async fn refresh_credentials(
    credentials: &mut Credentials,
    _semaphore: &FetchSemaphore,
) -> crate::Result<()> {
    let oauth =
        hydra::refresh::refresh(credentials.refresh_token.clone()).await?;

    let xbl_token = xbl_signin::login_xbl(&oauth.access_token).await?;

    // Get xsts token from xbl token
    let xsts_response = xsts_token::fetch_token(&xbl_token.token).await?;

    match xsts_response {
        xsts_token::XSTSResponse::Unauthorized(err) => {
            return Err(crate::ErrorKind::HydraError(format!(
                "Error getting XBox Live token: {}",
                err
            ))
            .as_error())
        }
        xsts_token::XSTSResponse::Success { token: xsts_token } => {
            let (bearer_token, expires_in) =
                bearer_token::fetch_bearer(&xsts_token, &xbl_token.uhs)
                    .await
                    .map_err(|err| {
                        crate::ErrorKind::HydraError(format!(
                            "Error getting bearer token: {}",
                            err
                        ))
                    })?;

            credentials.access_token = bearer_token;
            credentials.refresh_token = oauth.refresh_token;
            credentials.expires = Utc::now() + Duration::seconds(expires_in);
        }
    }

    Ok(())
}
