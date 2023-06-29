use crate::api::Result;
use std::path::PathBuf;
use theseus::{
    pack::{
        install::install_pack,
        install_from::{CreatePackLocation, CreatePackProfile},
    },
    prelude::*,
};

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("pack")
        .invoke_handler(tauri::generate_handler![
            pack_install,
            pack_get_profile_from_pack,
        ])
        .build()
}

#[tauri::command]
pub async fn pack_install(
    location: CreatePackLocation,
    profile: PathBuf,
) -> Result<PathBuf> {
    Ok(install_pack(location, profile).await?)
}

#[tauri::command]
pub fn pack_get_profile_from_pack(
    location: CreatePackLocation,
) -> Result<CreatePackProfile> {
    Ok(pack::install_from::get_profile_from_pack(location))
}
