//! Authentication flow interface
use crate::{prelude::Profile, util::jre::{self, JavaVersion, JREError}, state::GameVersionString};


/// Detect the optimal JRE for the given profile
#[tracing::instrument]
pub async fn detect_optimal_jre(profile : &Profile) -> crate::Result<JavaVersion> {
    // Get all JREs that exist on the system and are allowed for the given game version
    // If Ok(...), guaranteed to have at least one result
    let usable_jres = get_all_allowable_jre(profile)?;

    // TODO: with equally viable JREs, have a better system of choosing
    let optimal_jre = usable_jres.into_iter().max_by_key(|jre| jre.version.clone()).unwrap();
    Ok(optimal_jre)
}

// Get all JREs that exist on the system
pub fn get_all_jre() -> crate::Result<Vec<JavaVersion>> {
    Ok(jre::get_all_jre()?)  
}

// Get all allowed JREs for a given game version that exist on the system
// If Ok(...), guaranteed to have at least one result
pub fn get_all_allowable_jre(profile : &Profile) -> crate::Result<Vec<JavaVersion>> {
    let needed_versions =  if profile.metadata.game_version >= GameVersionString("1.17".to_string()) {
        // Java versions allowable if game version is 1.17 or higher
        vec!["1.17", "20"] // 20 converts to 1.20
    } else {
        // Java versions allowable if game version is 1.16 or lower
        vec!["1.8"]
    };
    let needed_version_tuples : Result<Vec<(u8,u8)>, JREError> = needed_versions.iter().map(| v| jre::extract_java_majorminor_version(v)).collect();
    let needed_version_tuples = needed_version_tuples?;

    // Get all JREs that exist on the system
    let jres = jre::get_all_jre()?;

    let usable_jres = jres.into_iter().filter(|jre | {
        // Rather than breaking on a bad JRE, just skip it
        needed_version_tuples.contains(&jre::extract_java_majorminor_version(&jre.version).unwrap_or((0,0)))
    });
    let usable_jres = usable_jres.collect::<Vec<JavaVersion>>();

    if usable_jres.is_empty() {
        return Err(JREError::NoJREFound(needed_versions.join(",")).into());
    }
    Ok(usable_jres)
}