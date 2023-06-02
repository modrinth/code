use crate::api::Result;
use std::path::{Path, PathBuf};
use theseus::prelude::*;

#[tauri::command]
pub async fn pack_install_version_id(
    project_id: String,
    version_id: String,
    pack_title: String,
    pack_icon: Option<String>,
) -> Result<PathBuf> {
    let res = pack::install_pack_from_version_id(
        project_id, version_id, pack_title, pack_icon,
    )
    .await?;
    Ok(res)
}

#[tauri::command]
pub async fn pack_install_file(path: &Path) -> Result<PathBuf> {
    let res = pack::install_pack_from_file(path.to_path_buf()).await?;
    Ok(res)
}
