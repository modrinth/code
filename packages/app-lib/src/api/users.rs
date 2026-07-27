use crate::State;
use crate::util::fetch::{fetch_advanced, fetch_json};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

#[tracing::instrument]
pub async fn get_user_profile(user_id: &str) -> crate::Result<Value> {
	let state = State::get().await?;
	let user_id = urlencoding::encode(user_id);

	fetch_json(
		Method::GET,
		&format!("{}user/{}", env!("MODRINTH_API_URL_V3"), user_id),
		None,
		None,
		Some("/v3/user/:id"),
		&state.api_semaphore,
		&state.pool,
	)
	.await
}

#[tracing::instrument]
pub async fn get_user_projects(user_id: &str) -> crate::Result<Value> {
	let state = State::get().await?;
	let user_id = urlencoding::encode(user_id);

	fetch_json(
		Method::GET,
		&format!("{}user/{}/projects", env!("MODRINTH_API_URL"), user_id),
		None,
		None,
		Some("/v2/user/:id/projects"),
		&state.api_semaphore,
		&state.pool,
	)
	.await
}

#[tracing::instrument]
pub async fn get_user_organizations(user_id: &str) -> crate::Result<Value> {
	let state = State::get().await?;
	let user_id = urlencoding::encode(user_id);

	fetch_json(
		Method::GET,
		&format!(
			"{}user/{}/organizations",
			env!("MODRINTH_API_URL_V3"),
			user_id
		),
		None,
		None,
		Some("/v3/user/:id/organizations"),
		&state.api_semaphore,
		&state.pool,
	)
	.await
}

#[tracing::instrument]
pub async fn get_user_collections(user_id: &str) -> crate::Result<Value> {
	let state = State::get().await?;
	let user_id = urlencoding::encode(user_id);

	fetch_json(
		Method::GET,
		&format!(
			"{}user/{}/collections",
			env!("MODRINTH_API_URL_V3"),
			user_id
		),
		None,
		None,
		Some("/v3/user/:id/collections"),
		&state.api_semaphore,
		&state.pool,
	)
	.await
}

#[tracing::instrument(skip(patch))]
pub async fn patch_user(user_id: &str, patch: Value) -> crate::Result<()> {
	let state = State::get().await?;
	let user_id = urlencoding::encode(user_id);

	fetch_advanced(
		Method::PATCH,
		&format!("{}user/{}", env!("MODRINTH_API_URL"), user_id),
		None,
		Some(patch),
		None,
		None,
		None,
		Some("/v2/user/:id"),
		&state.api_semaphore,
		&state.pool,
	)
	.await?;

	Ok(())
}
