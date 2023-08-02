//! Fetch player info for display
use serde::Deserialize;

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
            name: String::from("???"),
        }
    }
}

pub async fn fetch_info(token: &str) -> Result<PlayerInfo, reqwest::Error> {
    let client = reqwest::Client::new();
    let resp = client
        .get(PROFILE_URL)
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(resp)
}
