use std::path::PathBuf;

use crate::api::Result;
use theseus::prelude::JavaVersion;
use theseus::prelude::*;

/// Get all JREs that exist on the system
#[tauri::command]
pub async fn jre_get_all_jre() -> Result<Vec<JavaVersion>> {
    Ok(jre::get_all_jre().await?)
}

// Finds the isntallation of Java 7, if it exists
#[tauri::command]
pub async fn jre_find_jre_8_jres() -> Result<Vec<JavaVersion>> {
    Ok(jre::find_java8_jres().await?)
}

// finds the installation of Java 17, if it exists
#[tauri::command]
pub async fn jre_find_jre_17_jres() -> Result<Vec<JavaVersion>> {
    Ok(jre::find_java17_jres().await?)
}

// Finds the highest version of Java 18+, if it exists
#[tauri::command]
pub async fn jre_find_jre_18plus_jres() -> Result<Vec<JavaVersion>> {
    Ok(jre::find_java18plus_jres().await?)
}

// Autodetect Java globals, by searching the users computer.
// Returns a *NEW* JavaGlobals that can be put into Settings
#[tauri::command]
pub async fn jre_autodetect_java_globals() -> Result<JavaGlobals> {
    Ok(jre::autodetect_java_globals().await?)
}

// Validates java globals, by checking if the paths exist
// If false, recommend to direct them to reassign, or to re-guess
#[tauri::command]
pub async fn jre_validate_globals() -> Result<bool> {
    Ok(jre::validate_globals().await?)
}

// Validates JRE at a given path
// Returns None if the path is not a valid JRE
#[tauri::command]
pub async fn jre_get_jre(path: PathBuf) -> Result<Option<JavaVersion>> {
    jre::check_jre(path).await.map_err(|e| e.into())
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
