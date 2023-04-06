use std::path::Path;

use crate::api::Result;
use theseus::prelude::*;
use theseus::prelude::JavaVersion;

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

// Finds the highest version of Java 17+, if it exists
#[tauri::command]
pub async fn jre_find_jre_17plus_jres() -> Result<Vec<JavaVersion>> {
    Ok(jre::find_java17plus_jres()?)
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
    let profile = profile::get(path).await?.ok_or_else(|| TheseusSerializableError::NoProfileFound(path.display().to_string()))?;
    Ok(jre::get_optimal_jre_key(&profile).await?)
}