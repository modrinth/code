//! Login route for Hydra, redirects to the Microsoft login page before going to the redirect route
use serde::{Deserialize, Serialize};

use super::{MICROSOFT_CLIENT_ID, REDIRECT_URL, REQUESTED_SCOPES};

#[derive(Serialize, Deserialize)]
pub struct AuthInit {
    pub verification_uri: String,
}

pub async fn init() -> AuthInit {
    let url = format!(
        "https://login.live.com/oauth20_authorize.srf?client_id={MICROSOFT_CLIENT_ID}&response_type=code&redirect_uri={}&scope={}&prompt=select_account&cobrandid=8058f65d-ce06-4c30-9559-473c9275a65d",
        urlencoding::encode(REDIRECT_URL),
        urlencoding::encode(REQUESTED_SCOPES),
    );

    AuthInit {
        verification_uri: url,
    }
}
