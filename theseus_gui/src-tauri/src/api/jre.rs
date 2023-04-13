use std::path::{Path, PathBuf};

use crate::api::Result;
use theseus::prelude::JavaVersion;
use theseus::prelude::*;

use super::TheseusSerializableError;

/// Get all JREs that exist on the system
#[tauri::command]
pub async fn jre_get_all_jre() -> Result<Vec<JavaVersion>> {
    Ok(jre::get_all_jre()?)
}

// Finds the isntallation of Java 7, if it exists
#[tauri::command]
pub async fn jre_find_jre_8_jres() -> Result<Vec<JavaVersion>> {
    Ok(jre::find_java8_jres()?)
}

// finds the installation of Java 17, if it exists
#[tauri::command]
pub async fn jre_find_jre_17_jres() -> Result<Vec<JavaVersion>> {
    Ok(jre::find_java17_jres()?)
}

// Finds the highest version of Java 18+, if it exists
#[tauri::command]
pub async fn jre_find_jre_18plus_jres() -> Result<Vec<JavaVersion>> {
    Ok(jre::find_java18plus_jres()?)
}

// Autodetect Java globals, by searching the users computer.
// Returns a *NEW* JavaGlobals that can be put into Settings
#[tauri::command]
pub async fn jre_autodetect_java_globals() -> Result<JavaGlobals> {
    Ok(jre::autodetect_java_globals()?)
}

// Gets key for the optimal JRE to use, for a given profile Profile
// The key can be used in the hashmap contained by JavaGlobals in Settings (if it exists)
#[tauri::command]
pub async fn jre_get_optimal_jre_key(profile: Profile) -> Result<String> {
    Ok(jre::get_optimal_jre_key(&profile).await?)
}

// Gets key for the optimal JRE to use, for a given profile path
// The key can be used in the hashmap contained by JavaGlobals in Settings (if it exists)
#[tauri::command]
pub async fn jre_get_optimal_jre_key_by_path(path: &Path) -> Result<String> {
    let profile = profile::get(path).await?.ok_or_else(|| {
        TheseusSerializableError::NoProfileFound(path.display().to_string())
    })?;
    Ok(jre::get_optimal_jre_key(&profile).await?)
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
pub async fn jre_get_jre(path : PathBuf) -> Result<Option<JavaVersion>> {
    jre::check_jre(path).await.map_err(|e| e.into())
}