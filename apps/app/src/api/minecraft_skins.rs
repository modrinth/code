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
            set_default_cape,
            equip_skin,
            remove_custom_skin,
            unequip_skin,
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

/// `invoke('plugin:minecraft-skins|add_and_equip_custom_skin', texture_blob, variant, cape_override)`
///
/// See also: [minecraft_skins::add_and_equip_custom_skin]
#[tauri::command]
pub async fn add_and_equip_custom_skin(
    texture_blob: Bytes,
    variant: MinecraftSkinVariant,
    cape_override: Option<Cape>,
) -> Result<()> {
    Ok(minecraft_skins::add_and_equip_custom_skin(
        texture_blob,
        variant,
        cape_override,
    )
    .await?)
}

/// `invoke('plugin:minecraft-skins|set_default_cape', cape)`
///
/// See also: [minecraft_skins::set_default_cape]
#[tauri::command]
pub async fn set_default_cape(cape: Option<Cape>) -> Result<()> {
    Ok(minecraft_skins::set_default_cape(cape).await?)
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

/// `invoke('plugin:minecraft-skins|unequip_skin')`
///
/// See also: [minecraft_skins::unequip_skin]
#[tauri::command]
pub async fn unequip_skin() -> Result<()> {
    Ok(minecraft_skins::unequip_skin().await?)
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
