use std::path::PathBuf;

use crate::api::Result;
use tauri::plugin::TauriPlugin;
use theseus::prelude::JavaVersion;
use theseus::prelude::*;

pub fn init<R: tauri::Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("jre")
        .invoke_handler(tauri::generate_handler![
            jre_find_filtered_jres,
            jre_get_jre,
            jre_test_jre,
            jre_auto_install_java,
            jre_get_max_memory,
        ])
        .build()
}

// Finds the installation of Java 8, if it exists
#[tauri::command]
pub async fn jre_find_filtered_jres(
    version: Option<u32>,
) -> Result<Vec<JavaVersion>> {
    Ok(jre::find_filtered_jres(version).await?)
}

// Validates JRE at a given path
// Returns None if the path is not a valid JRE
#[tauri::command]
pub async fn jre_get_jre(path: PathBuf) -> Result<Option<JavaVersion>> {
    jre::check_jre(path).await.map_err(|e| e.into())
}

// Tests JRE of a certain version
#[tauri::command]
pub async fn jre_test_jre(path: PathBuf, major_version: u32) -> Result<bool> {
    Ok(jre::test_jre(path, major_version).await?)
}

// Auto installs java for the given java version
#[tauri::command]
pub async fn jre_auto_install_java(java_version: u32) -> Result<PathBuf> {
    Ok(jre::auto_install_java(java_version).await?)
}

// Gets the maximum memory a system has available.
#[tauri::command]
pub async fn jre_get_max_memory() -> Result<u64> {
    Ok(jre::get_max_memory().await?)
}
