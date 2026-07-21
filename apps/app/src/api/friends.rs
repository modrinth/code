use tauri::plugin::TauriPlugin;
use theseus::prelude::{UserFriend, UserStatus};

pub fn init<R: tauri::Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("friends")
        .invoke_handler(tauri::generate_handler![
            friends,
            friend_statuses,
            add_friend,
            remove_friend
        ])
        .build()
}

#[tauri::command]
pub async fn friends(
    invocation_context: theseus::InvocationContext,
) -> crate::api::Result<Vec<UserFriend>> {
    let context = crate::api::operation_context(invocation_context);
    Ok(theseus::friends::friends(&context).await?)
}

#[tauri::command]
pub async fn friend_statuses() -> crate::api::Result<Vec<UserStatus>> {
    Ok(theseus::friends::friend_statuses().await?)
}

#[tauri::command]
pub async fn add_friend(
    user_id: &str,
    invocation_context: theseus::InvocationContext,
) -> crate::api::Result<()> {
    let context = crate::api::operation_context(invocation_context);
    Ok(theseus::friends::add_friend(&context, user_id).await?)
}

#[tauri::command]
pub async fn remove_friend(
    user_id: &str,
    invocation_context: theseus::InvocationContext,
) -> crate::api::Result<()> {
    let context = crate::api::operation_context(invocation_context);
    Ok(theseus::friends::remove_friend(&context, user_id).await?)
}
