//! TaxPhobia news API integration.

use serde::{Deserialize, Serialize};
use tauri::{plugin::TauriPlugin, Runtime};

const DEFAULT_TAXPHOBIA_API_URL: &str = "https://taxphobia.top/api";
const TAXPHOBIA_API_KEY_ENV: &str = "TAXPHOBIA_API_KEY";
const TAXPHOBIA_API_URL_ENV: &str = "TAXPHOBIA_API_URL";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
	pub id: String,
	pub title: String,
	pub content: String,
	pub excerpt: Option<String>,
	pub date: String,
	#[serde(rename = "type")]
	pub news_type: String,
	pub pinned: bool,
	pub image: Option<String>,
	pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsResponse {
	pub success: bool,
	pub news: Vec<NewsItem>,
}

#[tauri::command]
pub async fn get_taxphobia_news() -> Result<NewsResponse, String> {
	let api_key = std::env::var(TAXPHOBIA_API_KEY_ENV).map_err(|_| {
		format!(
			"Missing required environment variable: {}",
			TAXPHOBIA_API_KEY_ENV
		)
	})?;

	let api_url = std::env::var(TAXPHOBIA_API_URL_ENV)
		.unwrap_or_else(|_| DEFAULT_TAXPHOBIA_API_URL.to_string());

	let client = reqwest::Client::builder()
		.timeout(std::time::Duration::from_secs(10))
		.build()
		.map_err(|e| format!("Failed to build HTTP client: {e}"))?;

	let response = client
		.get(format!("{api_url}/v1.php"))
		.query(&[("endpoint", "news")])
		.header("X-Tax-Auth-Key", api_key)
		.header("User-Agent", pteron::launcher_user_agent())
		.send()
		.await
		.map_err(|e| format!("Failed to fetch news: {e}"))?;

	if !response.status().is_success() {
		let status = response.status();
		let error_text = response.text().await.unwrap_or_default();
		return Err(format!("API returned error: {status} - {error_text}"));
	}

	let body: serde_json::Value = response
		.json()
		.await
		.map_err(|e| format!("Failed to parse response: {e}"))?;

	if !body["success"].as_bool().unwrap_or(false) {
		return Err(format!(
			"API returned success: false - {}",
			body.get("error").map(|e| e.to_string()).unwrap_or_default()
		));
	}

	let mut news: Vec<NewsItem> = serde_json::from_value(body["news"].clone())
		.map_err(|e| format!("Failed to parse news: {e}"))?;

	news.sort_by(|a, b| b.pinned.cmp(&a.pinned));

	Ok(NewsResponse {
		success: true,
		news,
	})
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
	tauri::plugin::Builder::new("taxphobia-news")
		.invoke_handler(tauri::generate_handler![get_taxphobia_news])
		.build()
}
