use crate::api::Result;
use std::path::{Path, PathBuf};
use theseus::{prelude::*, window_scoped};

// Creates a pack from a version ID (returns a path to the created profile)
// invoke('pack_install_version_id', version_id)
#[tauri::command]
pub async fn pack_install_version_id(
    window: tauri::Window,
    version_id: String,
) -> Result<PathBuf> {
    let res =
        window_scoped!(window, pack::install_pack_from_version_id(version_id))
            .await?;
    Ok(res)
}

#[tauri::command]
pub async fn pack_install_file(
    window: tauri::Window,
    path: &Path,
) -> Result<PathBuf> {
    let res = window_scoped!(
        window,
        pack::install_pack_from_file(path.to_path_buf())
    )
    .await?;
    Ok(res)
}
