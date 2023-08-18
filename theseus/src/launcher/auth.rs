//! Authentication flow based on Hydra

use crate::hydra;
use crate::util::fetch::FetchSemaphore;

use chrono::{prelude::*, Duration};

use serde::{Deserialize, Serialize};

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
    let res =
        hydra::refresh::refresh(credentials.refresh_token.clone()).await?;

    credentials.access_token = res.access_token;
    credentials.refresh_token = res.refresh_token;
    credentials.expires = Utc::now() + Duration::seconds(res.expires_in);

    Ok(())
}
