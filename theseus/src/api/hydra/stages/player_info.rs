//! Fetch player info for display
use serde::Deserialize;

use crate::util::fetch::REQWEST_CLIENT;

use super::auth_retry;

const PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";

#[derive(Deserialize)]
pub struct PlayerInfo {
    pub id: String,
    pub name: String,
}

impl Default for PlayerInfo {
    fn default() -> Self {
        Self {
            id: "606e2ff0ed7748429d6ce1d3321c7838".to_string(),
            name: String::from("Unknown"),
        }
    }
}

#[tracing::instrument]
pub async fn fetch_info(token: &str) -> crate::Result<PlayerInfo> {
    auth_retry(|| {
        REQWEST_CLIENT
            .get("https://api.minecraftservices.com/entitlements/mcstore")
            .bearer_auth(token)
            .send()
    })
    .await?;

    let response = auth_retry(|| {
        REQWEST_CLIENT
            .get(PROFILE_URL)
            .header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"))
            .send()
    })
    .await?;

    let resp = response.error_for_status()?.json().await?;

    Ok(resp)
}
