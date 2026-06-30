use crate::api::Result;
use theseus::users::SearchUser;

#[tauri::command]
pub async fn search_user(query: &str) -> Result<Vec<SearchUser>> {
	Ok(theseus::users::search_user(query).await?)
}

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
	tauri::plugin::Builder::new("users")
		.invoke_handler(tauri::generate_handler![search_user])
		.build()
}
