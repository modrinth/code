//! Authentication flow interface
use crate::{launcher::auth as inner, State, prelude::Profile, util::jre::{self, JavaVersion, JREError}, state::GameVersionString};
use futures::prelude::*;
use tokio::sync::oneshot;



/// Detect the optimal JRE for the given profile
#[tracing::instrument]
pub async fn detect_optimal_jre(profile : &Profile) -> crate::Result<JavaVersion> {

    // TODO: implement this
    let needed_version =  if profile.metadata.game_version >= GameVersionString("1.17".to_string()) {
        "1.7".to_string()
    } else {
        "1.7".to_string()
    };

    let jres = jre::get_all_jre()?;

    dbg!("all jres: ",&jres);

    let usable_jres = jres.into_iter().filter(|jre | {
        needed_version == jre.version
    });
    dbg!("all usable jres: ",&usable_jres);

    //TODO: with equally viable JREs, have a better system of choosing
    // TODO: get rid of clone
    let optimal_jre = usable_jres.max_by_key(|jre| jre.version.clone()).ok_or(JREError::NoJREFound(needed_version))?;

    Ok(optimal_jre)

}
