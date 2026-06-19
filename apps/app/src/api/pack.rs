use crate::api::Result;

use theseus::{
    pack::{
        install_from::{CreatePackLocation, CreatePackInstance},
        install_mrpack::install_zipped_mrpack,
    },
    prelude::*,
};

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("pack")
        .invoke_handler(tauri::generate_handler![
            pack_install,
            pack_get_instance_from_pack,
        ])
        .build()
}

#[tauri::command]
pub async fn pack_install(
    location: CreatePackLocation,
    instance_id: String,
) -> Result<String> {
    Ok(install_zipped_mrpack(location, instance_id).await?)
}

#[tauri::command]
pub async fn pack_get_instance_from_pack(
    location: CreatePackLocation,
) -> Result<CreatePackInstance> {
    Ok(pack::install_from::get_instance_from_pack(location).await?)
}
