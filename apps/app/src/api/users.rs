use crate::api::Result;
use serde_json::Value;
use theseus::users::SearchUser;

#[tauri::command]
pub async fn search_user(query: &str) -> Result<Vec<SearchUser>> {
    Ok(theseus::users::search_user(query).await?)
}

#[tauri::command]
pub async fn get_user_profile(user_id: &str) -> Result<Value> {
    Ok(theseus::users::get_user_profile(user_id).await?)
}

#[tauri::command]
pub async fn get_user_projects(user_id: &str) -> Result<Value> {
    Ok(theseus::users::get_user_projects(user_id).await?)
}

#[tauri::command]
pub async fn get_user_organizations(user_id: &str) -> Result<Value> {
    Ok(theseus::users::get_user_organizations(user_id).await?)
}

#[tauri::command]
pub async fn get_user_collections(user_id: &str) -> Result<Value> {
    Ok(theseus::users::get_user_collections(user_id).await?)
}

#[tauri::command]
pub async fn patch_user(user_id: &str, patch: Value) -> Result<()> {
    Ok(theseus::users::patch_user(user_id, patch).await?)
}

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("users")
        .invoke_handler(tauri::generate_handler![
            search_user,
            get_user_profile,
            get_user_projects,
            get_user_organizations,
            get_user_collections,
            patch_user,
        ])
        .build()
}
