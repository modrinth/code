//! Authentication flow interface
use crate::{
    prelude::Profile,
    util::jre::{self, JavaVersion, extract_java_majorminor_version},
    State, state::JavaGlobals,
};
use daedalus as d;

pub const JAVA_8_KEY : &str = "JAVA_8";
pub const JAVA_17PLUS_KEY : &str = "JAVA_17PLUS";


// Autodetect JavaSettings default
// Make a guess for what the default Java global settings should be
pub fn autodetect_java_globals() -> crate::Result<JavaGlobals> {
    let mut java_8 = find_java8_jres()?;
    let mut java_17plus = find_java17plus_jres()?;

    println!("java 8 and java 17");
    dbg!(&java_8);
    dbg!(&java_17plus);

    // Simply select last one found for initial guess
    let mut java_globals = JavaGlobals::new();
    if let Some(jre) = java_8.pop() {
        java_globals.insert(JAVA_8_KEY.to_string(), jre);
    } 
    if let Some(jre) = java_17plus.pop() {
        java_globals.insert(JAVA_17PLUS_KEY.to_string(), jre);
    } 
    dbg!(&java_globals);

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
        .find(|it| it.id == profile.metadata.game_version.as_ref())
        .ok_or_else(|| {
            crate::ErrorKind::LauncherError(format!(
                "Invalid or unknown Minecraft version: {}",
                profile.metadata.game_version
            ))
        })?;
    
    // Get detailed manifest info from Daedalus 
    let version_info = d::minecraft::fetch_version_info(&version).await?;
    let optimal_key = if version_info
            .java_version
            .as_ref()
            .filter(|it| it.major_version >= 17)
            .is_some()
    {
        JAVA_17PLUS_KEY.to_string()
    } else {
        JAVA_8_KEY.to_string()
    };
    Ok(optimal_key)
}


// Searches for jres on the system that are 1.17 or higher
pub fn find_java17plus_jres() -> crate::Result<Vec<JavaVersion>> {
    let version = extract_java_majorminor_version("1.17")?;
    let jres = jre::get_all_jre()?;

    // Filter out JREs that are not 1.17 or higher
    Ok(jres
        .into_iter()
        .filter(|jre| {
            let jre_version = extract_java_majorminor_version(&jre.version);
            dbg!("Comparing JRE version {} to {}", &jre_version, &version);
            if let Ok(jre_version) = jre_version {
                jre_version >= version
            } else {
                false
            }
        })
        .collect())
}

// Searches for jres on the system that are 1.8 exactly
pub fn find_java8_jres() -> crate::Result<Vec<JavaVersion>> {
    let version = extract_java_majorminor_version("1.8")?;
    let jres = jre::get_all_jre()?;

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
pub fn get_all_jre() -> crate::Result<Vec<JavaVersion>> {
    Ok(jre::get_all_jre()?)
}
