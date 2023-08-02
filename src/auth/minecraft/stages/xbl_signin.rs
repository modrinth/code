//! Signin for XBox Live

use crate::auth::AuthenticationError;
use serde_json::json;

const XBL_AUTH_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";

// Deserialization
pub struct XBLLogin {
    pub token: String,
    pub uhs: String,
}

// Impl
pub async fn login_xbl(token: &str) -> Result<XBLLogin, AuthenticationError> {
    let client = reqwest::Client::new();
    let body = client
        .post(XBL_AUTH_URL)
        .header(reqwest::header::ACCEPT, "application/json")
        .header("x-xbl-contract-version", "1")
        .json(&json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": format!("d={token}")
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"
        }))
        .send()
        .await?
        .text()
        .await?;

    let json = serde_json::from_str::<serde_json::Value>(&body)?;
    let token = Some(&json)
        .and_then(|it| it.get("Token")?.as_str().map(String::from))
        .ok_or(AuthenticationError::Custom(
            "XBL response didn't contain valid token".to_string(),
        ))?;
    let uhs = Some(&json)
        .and_then(|it| {
            it.get("DisplayClaims")?
                .get("xui")?
                .get(0)?
                .get("uhs")?
                .as_str()
                .map(String::from)
        })
        .ok_or(AuthenticationError::Custom(
            "XBL response didn't contain valid user hash".to_string(),
        ))?;

    Ok(XBLLogin { token, uhs })
}
