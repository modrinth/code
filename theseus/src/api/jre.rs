//! Authentication flow interface
use crate::{
    prelude::Profile,
    util::jre::{self, JREError, JavaVersion},
    State,
};
use chrono::{DateTime, Utc};

/// Detect the optimal JRE for the given profile
#[tracing::instrument]
pub async fn detect_optimal_jre(
    profile: &Profile,
) -> crate::Result<JavaVersion> {
    // Get all JREs that exist on the system and are allowed for the given game version
    // If Ok(...), guaranteed to have at least one result
    let usable_jres = get_all_allowable_jre(profile).await?;

    // TODO: with equally viable JREs, have a better system of choosing
    let optimal_jre = usable_jres
        .into_iter()
        .max_by_key(|jre| jre.version.clone())
        .unwrap();
    Ok(optimal_jre)
}

// Get all JREs that exist on the system
pub fn get_all_jre() -> crate::Result<Vec<JavaVersion>> {
    Ok(jre::get_all_jre()?)
}

// Get all allowed JREs for a given game version that exist on the system
// If Ok(...), guaranteed to have at least one result
pub async fn get_all_allowable_jre(
    profile: &Profile,
) -> crate::Result<Vec<JavaVersion>> {
    let needed_versions = match compare_minecraft_game_versions(
        &profile.metadata.game_version,
        "1.17",
    )
    .await?
    {
        std::cmp::Ordering::Greater | std::cmp::Ordering::Equal => {
            // Java versions allowable if game version is 1.17 or higher
            vec!["1.17"] // 20 converts to 1.20
        }
        std::cmp::Ordering::Less => {
            // Java versions allowable if game version is 1.16 or lower
            vec!["1.8"]
        }
    };

    let needed_version_tuples: Result<Vec<(u8, u8)>, JREError> =
        needed_versions
            .iter()
            .map(|v| jre::extract_java_majorminor_version(v))
            .collect();
    let needed_version_tuples = needed_version_tuples?;

    // Get all JREs that exist on the system
    let jres = jre::get_all_jre()?;

    let usable_jres = jres.into_iter().filter(|jre| {
        // Rather than breaking on a bad JRE, just skip it
        needed_version_tuples.contains(
            &jre::extract_java_majorminor_version(&jre.version)
                .unwrap_or((0, 0)),
        )
    });
    let usable_jres = usable_jres.collect::<Vec<JavaVersion>>();

    if usable_jres.is_empty() {
        return Err(JREError::NoJREFound(needed_versions.join(",")).into());
    }
    Ok(usable_jres)
}

async fn compare_minecraft_game_versions(
    version1: &str,
    version2: &str,
) -> crate::Result<std::cmp::Ordering> {
    let state = State::get().await?;
    let game_versions = state.tags.read().await.get_game_versions()?;

    let game_version_1 = game_versions
        .iter()
        .find(|v| v.version == version1)
        .ok_or_else(|| {
        JREError::NoMinecraftVersionFound(version1.to_string())
    })?;
    let game_version_2 = game_versions
        .iter()
        .find(|v| v.version == version2)
        .ok_or_else(|| {
        JREError::NoMinecraftVersionFound(version2.to_string())
    })?;

    // Convert the inner dates to DateTime<Utc> for comparison
    let game_version_1 =
        DateTime::parse_from_rfc3339(&game_version_1.date)?.with_timezone(&Utc);
    let game_version_2 =
        DateTime::parse_from_rfc3339(&game_version_2.date)?.with_timezone(&Utc);

    Ok(game_version_1.cmp(&game_version_2))
}

pub fn find_jre_8() -> crate::Result<Option<JavaVersion>> {
    let jres = jre::get_all_jre()?;
    Ok(jres.into_iter().find(|jre| jre.version == "1.7"))
}

pub fn find_jre_17plus() -> crate::Result<Option<JavaVersion>> {
    let jres = jre::get_all_jre()?;
    let jres_17plus = jres.into_iter().find(|jre| {
        let version = jre::extract_java_majorminor_version(&jre.version)
            .unwrap_or((0, 0));
        version.0 >= 1 && version.1 >= 17
    });

    // Pick highest minor version if multiple 1.17+ JREs are found
    let highest_jre = jres_17plus.into_iter().max_by_key(|jre| {
        jre::extract_java_majorminor_version(&jre.version)
            .unwrap_or((0, 0))
            .1
    });
    Ok(highest_jre)
}
