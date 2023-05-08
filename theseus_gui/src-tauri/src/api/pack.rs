use crate::api::Result;
use std::path::{Path, PathBuf};
use theseus::prelude::*;

// Creates a pack from a version ID (returns a path to the created profile)
// invoke('pack_install_version_id', version_id)
#[tauri::command]
pub async fn pack_install_version_id(
    version_id: String,
    pack_title: Option<String>,
) -> Result<PathBuf> {
    let res =
        pack::install_pack_from_version_id(version_id, pack_title).await?;
    Ok(res)
}

#[tauri::command]
pub async fn pack_install_file(path: &Path) -> Result<PathBuf> {
    let res = pack::install_pack_from_file(path.to_path_buf()).await?;
    Ok(res)
}
