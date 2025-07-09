use crate::api::Result;
use dashmap::DashMap;
use std::path::PathBuf;
use tauri::plugin::TauriPlugin;
use theseus::prelude::JavaVersion;
use theseus::prelude::*;

pub fn init<R: tauri::Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("jre")
        .invoke_handler(tauri::generate_handler![
            get_java_versions,
            set_java_version,
            jre_find_filtered_jres,
            jre_get_jre,
            jre_test_jre,
            jre_auto_install_java,
            jre_get_max_memory,
        ])
        .build()
}

#[tauri::command]
pub async fn get_java_versions() -> Result<DashMap<u32, JavaVersion>> {
    Ok(jre::get_java_versions().await?)
}

#[tauri::command]
pub async fn set_java_version(java_version: JavaVersion) -> Result<()> {
    jre::set_java_version(java_version).await?;
    Ok(())
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
pub async fn jre_get_jre(path: PathBuf) -> Result<JavaVersion> {
    Ok(jre::check_jre(path).await?)
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
