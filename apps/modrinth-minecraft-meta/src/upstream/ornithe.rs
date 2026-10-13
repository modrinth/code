use derive_more::Display;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::upstream::mojang::{JavaVersion, Library};

pub const GAMES_URL: &str = "https://meta.ornithemc.net/v3/versions/gen2/game";
pub const LOADERS_URL: &str =
    "https://meta.ornithemc.net/v3/versions/gen2/fabric-loader";
pub const PROFILE_BASE_URL: &str =
    "https://meta.ornithemc.net/v3/versions/gen2/fabric-loader/";
pub const LIBRARIES_BASE_URL: &str =
    "https://meta.ornithemc.net/v3/versions/gen2/libraries/";
pub const VERSION_MANIFEST_BASE_URL: &str =
    "https://ornithemc.net/mc-versions/gen2/version/manifest/";
pub const MAVEN_URL: &str = "https://maven.ornithemc.net/releases/";

/// Host of the LWJGL builds which Ornithe's version manifests swap in for
/// Mojang's.
pub const LWJGL_UPGRADE_HOST: &str = "maven.legacyfabric.net";

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct GameVersionName(pub String);

/// Entry of the game version list at [`GAMES_URL`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameVersion {
    pub version: GameVersionName,
    pub stable: bool,
}

#[derive(
    Debug, Display, Clone, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub struct LoaderVersionName(pub String);

/// Entry of the Fabric Loader version list at [`LOADERS_URL`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoaderVersion {
    pub version: LoaderVersionName,
    pub stable: bool,
}

/// Ornithe's copy of a Mojang version manifest, served at
/// [`VERSION_MANIFEST_BASE_URL`]`{game_version}.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionManifest {
    pub libraries: Vec<Library>,
    #[serde(default)]
    pub java_version: Option<JavaVersion>,
}

/// Example URL: <https://meta.ornithemc.net/v3/versions/gen2/fabric-loader/1.8.9/0.19.5/profile/json>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameLoaderProfile {
    pub id: String,
}

/// Whether Ornithe swaps this library in for Mojang's LWJGL.
pub fn is_lwjgl_upgrade(library: &Library) -> bool {
    let downloads = library.downloads.as_ref();
    downloads
        .and_then(|downloads| downloads.artifact.as_ref())
        .into_iter()
        .chain(
            downloads
                .and_then(|downloads| downloads.classifiers.as_ref())
                .into_iter()
                .flat_map(|classifiers| classifiers.values()),
        )
        .filter_map(|download| download.url.as_ref())
        .chain(
            library
                .url
                .as_deref()
                .and_then(|url| Url::parse(url).ok())
                .as_ref(),
        )
        .any(|url| url.host_str() == Some(LWJGL_UPGRADE_HOST))
}
