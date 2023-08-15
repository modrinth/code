use serde_json::json;

const MCSERVICES_AUTH_URL: &str =
    "https://api.minecraftservices.com/launcher/login";

pub async fn fetch_bearer(token: &str, uhs: &str) -> crate::Result<String> {
    let client = reqwest::Client::new();
    let body = client
        .post(MCSERVICES_AUTH_URL)
        .json(&json!({
            "xtoken": format!("XBL3.0 x={};{}", uhs, token),
            "platform": "PC_LAUNCHER"
        }))
        .send()
        .await?
        .text()
        .await?;

    serde_json::from_str::<serde_json::Value>(&body)?
        .get("access_token")
        .and_then(serde_json::Value::as_str)
        .map(String::from)
        .ok_or(
            crate::ErrorKind::HydraError(format!(
                "Response didn't contain valid bearer token. body: {body}"
            ))
            .into(),
        )
}
