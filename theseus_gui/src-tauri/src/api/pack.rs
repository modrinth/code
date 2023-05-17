use crate::api::Result;
use std::path::{Path, PathBuf};
use theseus::prelude::*;

#[tauri::command]
#[theseus_macros::debug_pin]
pub async fn pack_install_version_id(
    version_id: String,
    pack_title: String,
    pack_icon: Option<String>,
) -> Result<PathBuf> {
    let res =
        pack::install_pack_from_version_id(version_id, pack_title, pack_icon)
            .await?;
    Ok(res)
}

#[tauri::command]
#[theseus_macros::debug_pin]
pub async fn pack_install_file(path: &Path) -> Result<PathBuf> {
    let res = pack::install_pack_from_file(path.to_path_buf()).await?;
    Ok(res)
}
