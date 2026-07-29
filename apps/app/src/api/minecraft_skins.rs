use crate::api::Result;

use std::path::Path;
use theseus::minecraft_skins::{
    self, Bytes, Cape, MinecraftSkinVariant, Skin, UrlOrBlob,
};

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("minecraft-skins")
        .invoke_handler(tauri::generate_handler![
            get_available_capes,
            get_available_skins,
            add_and_equip_custom_skin,
            equip_skin,
            remove_custom_skin,
            save_custom_skin,
            set_custom_skin_order,
            unequip_skin,
            flush_pending_skin_change,
            flush_pending_skin_change_for_profile,
            normalize_skin_texture,
            get_dragged_skin_data,
        ])
        .build()
}

/// `invoke('plugin:minecraft-skins|get_available_capes')`
///
/// See also: [minecraft_skins::get_available_capes]
#[tauri::command]
pub async fn get_available_capes() -> Result<Vec<Cape>> {
    Ok(minecraft_skins::get_available_capes().await?)
}

/// `invoke('plugin:minecraft-skins|get_available_skins')`
///
/// See also: [minecraft_skins::get_available_skins]
#[tauri::command]
pub async fn get_available_skins() -> Result<Vec<Skin>> {
    Ok(minecraft_skins::get_available_skins().await?)
}

/// `invoke('plugin:minecraft-skins|add_and_equip_custom_skin', texture_blob, variant, cape)`
///
/// See also: [minecraft_skins::add_and_equip_custom_skin]
#[tauri::command]
pub async fn add_and_equip_custom_skin(
    texture_blob: Bytes,
    variant: MinecraftSkinVariant,
    cape: Option<Cape>,
) -> Result<Skin> {
    Ok(
        minecraft_skins::add_and_equip_custom_skin(texture_blob, variant, cape)
            .await?,
    )
}

/// `invoke('plugin:minecraft-skins|equip_skin', skin)`
///
/// See also: [minecraft_skins::equip_skin]
#[tauri::command]
pub async fn equip_skin(skin: Skin) -> Result<()> {
    Ok(minecraft_skins::equip_skin(skin).await?)
}

/// `invoke('plugin:minecraft-skins|remove_custom_skin', skin)`
///
/// See also: [minecraft_skins::remove_custom_skin]
#[tauri::command]
pub async fn remove_custom_skin(skin: Skin) -> Result<()> {
    Ok(minecraft_skins::remove_custom_skin(skin).await?)
}

/// `invoke('plugin:minecraft-skins|save_custom_skin', skin, texture_blob, variant, cape, replace_texture)`
///
/// See also: [minecraft_skins::save_custom_skin]
#[tauri::command]
pub async fn save_custom_skin(
    skin: Skin,
    texture_blob: Bytes,
    variant: MinecraftSkinVariant,
    cape: Option<Cape>,
    replace_texture: bool,
) -> Result<Skin> {
    Ok(minecraft_skins::save_custom_skin(
        skin,
        texture_blob,
        variant,
        cape,
        replace_texture,
    )
    .await?)
}

/// `invoke('plugin:minecraft-skins|set_custom_skin_order', texture_keys)`
///
/// See also: [minecraft_skins::set_custom_skin_order]
#[tauri::command]
pub async fn set_custom_skin_order(texture_keys: Vec<String>) -> Result<()> {
    Ok(minecraft_skins::set_custom_skin_order(texture_keys).await?)
}

/// `invoke('plugin:minecraft-skins|unequip_skin')`
///
/// See also: [minecraft_skins::unequip_skin]
#[tauri::command]
pub async fn unequip_skin() -> Result<()> {
    Ok(minecraft_skins::unequip_skin().await?)
}

/// `invoke('plugin:minecraft-skins|flush_pending_skin_change')`
///
/// See also: [minecraft_skins::flush_pending_skin_change]
#[tauri::command]
pub async fn flush_pending_skin_change() -> Result<()> {
    Ok(minecraft_skins::flush_pending_skin_change().await?)
}

/// `invoke('plugin:minecraft-skins|flush_pending_skin_change_for_profile', profile_id)`
///
/// See also: [minecraft_skins::flush_pending_skin_change_for_profile]
#[tauri::command]
pub async fn flush_pending_skin_change_for_profile(
    profile_id: uuid::Uuid,
) -> Result<()> {
    Ok(
        minecraft_skins::flush_pending_skin_change_for_profile(profile_id)
            .await?,
    )
}

/// `invoke('plugin:minecraft-skins|normalize_skin_texture')`
///
/// See also: [minecraft_skins::normalize_skin_texture]
#[tauri::command]
pub async fn normalize_skin_texture(texture: UrlOrBlob) -> Result<Bytes> {
    Ok(minecraft_skins::normalize_skin_texture(&texture).await?)
}

/// `invoke('plugin:minecraft-skins|get_dragged_skin_data', path)`
///
/// See also: [minecraft_skins::get_dragged_skin_data]
#[tauri::command]
pub async fn get_dragged_skin_data(path: String) -> Result<Bytes> {
    let path = Path::new(&path);
    Ok(minecraft_skins::get_dragged_skin_data(path).await?)
}
