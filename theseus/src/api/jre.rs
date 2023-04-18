//! Authentication flow interface
use std::path::PathBuf;

use crate::{
    launcher::download,
    prelude::Profile,
    state::JavaGlobals,
    util::jre::{self, extract_java_majorminor_version, JavaVersion},
    State,
};

pub const JAVA_8_KEY: &str = "JAVA_8";
pub const JAVA_17_KEY: &str = "JAVA_17";
pub const JAVA_18PLUS_KEY: &str = "JAVA_18PLUS";

// Autodetect JavaSettings default
// Make a guess for what the default Java global settings should be
pub async fn autodetect_java_globals() -> crate::Result<JavaGlobals> {
    let mut java_8 = find_java8_jres().await?;
    let mut java_17 = find_java17_jres().await?;
    let mut java_18plus = find_java18plus_jres().await?;

    // Simply select last one found for initial guess
    let mut java_globals = JavaGlobals::new();
    if let Some(jre) = java_8.pop() {
        java_globals.insert(JAVA_8_KEY.to_string(), jre);
    }
    if let Some(jre) = java_17.pop() {
        java_globals.insert(JAVA_17_KEY.to_string(), jre);
    }
    if let Some(jre) = java_18plus.pop() {
        java_globals.insert(JAVA_18PLUS_KEY.to_string(), jre);
    }

    Ok(java_globals)
}

// Gets the optimal JRE key for the given profile, using Daedalus
// Generally this would be used for profile_create, to get the optimal JRE key
// this can be overwritten by the user a profile-by-profile basis
pub async fn get_optimal_jre_key(profile: &Profile) -> crate::Result<String> {
    let state = State::get().await?;

    // Fetch version info from stored profile game_version
    let version = state
        .metadata
        .minecraft
        .versions
        .iter()
        .find(|it| it.id == profile.metadata.game_version)
        .ok_or_else(|| {
            crate::ErrorKind::LauncherError(format!(
                "Invalid or unknown Minecraft version: {}",
                profile.metadata.game_version
            ))
        })?;

    // Get detailed manifest info from Daedalus
    let version_info = download::download_version_info(
        &state,
        version,
        profile.metadata.loader_version.as_ref(),
    )
    .await?;
    let optimal_key = match version_info
        .java_version
        .as_ref()
        .map(|it| it.major_version)
        .unwrap_or(0)
    {
        0..=16 => JAVA_8_KEY.to_string(),
        17 => JAVA_17_KEY.to_string(),
        _ => JAVA_18PLUS_KEY.to_string(),
    };
    Ok(optimal_key)
}

// Searches for jres on the system that are 1.18 or higher
pub async fn find_java18plus_jres() -> crate::Result<Vec<JavaVersion>> {
    let version = extract_java_majorminor_version("1.18")?;
    let jres = jre::get_all_jre().await?;
    // Filter out JREs that are not 1.17 or higher
    Ok(jres
        .into_iter()
        .filter(|jre| {
            let jre_version = extract_java_majorminor_version(&jre.version);
            if let Ok(jre_version) = jre_version {
                jre_version >= version
            } else {
                false
            }
        })
        .collect())
}

// Searches for jres on the system that are 1.8 exactly
pub async fn find_java8_jres() -> crate::Result<Vec<JavaVersion>> {
    let version = extract_java_majorminor_version("1.8")?;
    let jres = jre::get_all_jre().await?;

    // Filter out JREs that are not 1.8
    Ok(jres
        .into_iter()
        .filter(|jre| {
            let jre_version = extract_java_majorminor_version(&jre.version);
            if let Ok(jre_version) = jre_version {
                jre_version == version
            } else {
                false
            }
        })
        .collect())
}

// Searches for jres on the system that are 1.17 exactly
pub async fn find_java17_jres() -> crate::Result<Vec<JavaVersion>> {
    let version = extract_java_majorminor_version("1.17")?;
    let jres = jre::get_all_jre().await?;

    // Filter out JREs that are not 1.8
    Ok(jres
        .into_iter()
        .filter(|jre| {
            let jre_version = extract_java_majorminor_version(&jre.version);
            if let Ok(jre_version) = jre_version {
                jre_version == version
            } else {
                false
            }
        })
        .collect())
}

// Get all JREs that exist on the system
pub async fn get_all_jre() -> crate::Result<Vec<JavaVersion>> {
    Ok(jre::get_all_jre().await?)
}

pub async fn validate_globals() -> crate::Result<bool> {
    let state = State::get().await?;
    let settings = state.settings.read().await;
    Ok(settings.java_globals.is_all_valid().await)
}

// Validates JRE at a given at a given path
pub async fn check_jre(path: PathBuf) -> crate::Result<Option<JavaVersion>> {
    Ok(jre::check_java_at_filepath(&path).await)
}
