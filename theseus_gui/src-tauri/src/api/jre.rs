use std::path::Path;

use crate::api::Result;
use theseus::prelude::*;

use super::TheseusSerializableError;
use theseus::prelude::JavaVersion;

/// Get all JREs that exist on the system
#[tauri::command]
pub async fn jre_get_all_jre() -> Result<Vec<JavaVersion>> {
    Ok(jre::get_all_jre()?)
}

// Finds the isntallation of Java 7, if it exists
#[tauri::command]
pub async fn jre_find_jre_8() -> Result<Option<JavaVersion>> {
    Ok(jre::find_jre_8()?)
}

// Finds the highest version of Java 17+, if it exists
#[tauri::command]
pub async fn jre_find_jre_17plus() -> Result<Option<JavaVersion>> {
    Ok(jre::find_jre_17plus()?)
}

/// From a Path to a profile, returns the JavaVersion of the optimal JRE to use
/// Returns an error if the profile is not managed by Theseus, or if the optimal JRE could not be detected
#[tauri::command]
pub async fn jre_detect_optimal_jre(path: &Path) -> Result<JavaVersion> {
    let profile = profile::get(path).await?;
    if let Some(profile) = profile {
        Ok(jre::detect_optimal_jre(&profile).await?)
    } else {
        Err(TheseusSerializableError::NoProfileFound(
            path.display().to_string(),
        )
        .into())
    }
}

/// Get all allowed JREs for a given game version that exist on the system
#[tauri::command]
pub async fn jre_get_all_allowable_jre(
    path: &Path,
) -> Result<Vec<JavaVersion>> {
    let profile = profile::get(path).await?;
    if let Some(profile) = profile {
        Ok(jre::get_all_allowable_jre(&profile).await?)
    } else {
        Err(TheseusSerializableError::NoProfileFound(
            path.display().to_string(),
        )
        .into())
    }
}
