use crate::State;
use crate::util::fetch::fetch_json;
use reqwest::Method;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchUser {
    pub id: String,
    pub username: String,
    pub avatar_url: Option<String>,
}

#[tracing::instrument]
pub async fn search_user(query: &str) -> crate::Result<Vec<SearchUser>> {
    let state = State::get().await?;
    let query = urlencoding::encode(query);

    fetch_json(
        Method::GET,
        &format!(
            "{}users/search?query={}",
            env!("MODRINTH_API_URL_V3"),
            query
        ),
        None,
        None,
        Some("/v3/users/search"),
        &state.api_semaphore,
        &state.pool,
    )
    .await
}
