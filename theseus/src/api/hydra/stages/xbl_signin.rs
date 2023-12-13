use serde_json::json;

use crate::util::fetch::REQWEST_CLIENT;

use super::auth_retry;

const XBL_AUTH_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";

// Deserialization
pub struct XBLLogin {
    pub token: String,
    pub uhs: String,
}

// Impl
#[tracing::instrument]
pub async fn login_xbl(token: &str) -> crate::Result<XBLLogin> {
    let response = auth_retry(|| {
        REQWEST_CLIENT
            .post(XBL_AUTH_URL)
            .header(reqwest::header::ACCEPT, "application/json")
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
    })
    .await?;
    let body = response.text().await?;

    let json = serde_json::from_str::<serde_json::Value>(&body)?;
    let token = Some(&json)
        .and_then(|it| it.get("Token")?.as_str().map(String::from))
        .ok_or(crate::ErrorKind::HydraError(
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
        .ok_or(
            crate::ErrorKind::HydraError(
                "XBL response didn't contain valid user hash".to_string(),
            )
            .as_error(),
        )?;

    Ok(XBLLogin { token, uhs })
}
