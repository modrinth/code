use crate::api::Result;
use theseus::{prelude::*, shared_profile::SharedProfile};

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("profile_share")
        .invoke_handler(tauri::generate_handler![
            profile_share_get_all,
            profile_share_install,
            profile_share_create,
            profile_share_inbound_sync,
            profile_share_outbound_sync,
            profile_share_generate_share_link,
            profile_share_accept_share_link
        ])
        .build()
}

// invoke('plugin:profile_share|profile_share_get_all',profile)
#[tauri::command]
pub async fn profile_share_get_all(
) -> Result<Vec<SharedProfile>> {
    let res = shared_profile::get_all()
    .await?;
    Ok(res)
}

#[tauri::command]
pub async fn profile_share_install(
    profile: SharedProfile,
) -> Result<ProfilePathId> {
    let res = shared_profile::install(profile)
    .await?;
    Ok(res)
}

#[tauri::command]
pub async fn profile_share_create(
    path: ProfilePathId
) -> Result<()> {    
    shared_profile::create(path)
    .await?;
    Ok(())
}   

#[tauri::command]
pub async fn profile_share_inbound_sync(
    path: ProfilePathId
) -> Result<()> {
    shared_profile::inbound_sync(path)
    .await?;
    Ok(())
}

#[tauri::command]
pub async fn profile_share_outbound_sync(
    path : ProfilePathId
) -> Result<()> {
    shared_profile::outbound_sync(path).await?;
    Ok(())
}

#[tauri::command]
pub async fn profile_share_generate_share_link(
    path : ProfilePathId
) -> Result<String> {
    let res = shared_profile::generate_share_link(path).await?;
    Ok(res)
}

#[tauri::command]
pub async fn profile_share_accept_share_link(
    link : String
) -> Result<()> {
    shared_profile::accept_share_link(link).await?;
    Ok(())
}