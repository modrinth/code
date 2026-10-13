//! Fetches Ornithe loader metadata.

use crate::fabric::{
    FabricGameVersion, FabricLoaderVersion, ProfileRequest,
    fetch_metadata_profiles, metadata_version_path,
};
use crate::util::{
    fetch_json, fetch_optional_json, format_url, retain_manifest_versions,
};
use crate::{Error, FetchResult, MirrorArtifact, UploadFile};
use daedalus::minecraft::{
    JavaVersion, Library, VERSION_MANIFEST_URL, VersionManifest,
};
use daedalus::modded::{
    CURRENT_ORNITHE_FORMAT_VERSION, LoaderVersion, Manifest, Version,
    VersionGroup,
};
use dashmap::DashMap;
use itertools::Itertools;
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Semaphore;

const MOD_LOADER: &str = "ornithe";
const META_URL: &str = "https://meta.ornithemc.net/v3/versions/gen2";
const MAVEN_URL: &str = "https://maven.fabricmc.net/";
const MC_VERSIONS_URL: &str =
    "https://ornithemc.net/mc-versions/gen2/version/manifest";
const LWJGL_MAVEN_HOST: &str = "maven.legacyfabric.net";

/// Mojang game version IDs paired with the Ornithe ID of the same client JAR.
const GAME_VERSION_ALIASES: &[(&str, &str)] = &[
    ("1.0", "1.0.0"),
    ("b1.3b", "b1.3"),
    ("a1.2.2b", "a1.2.2"),
    ("a1.2.2a", "a1.2.2-1624"),
    ("c0.30_01c", "c0.30-c-renew"),
];

fn ornithe_game_version(mojang_game_version: &str) -> String {
    GAME_VERSION_ALIASES
        .iter()
        .find(|(mojang, _)| *mojang == mojang_game_version)
        .map_or_else(
            || mojang_game_version.replace(" Pre-Release ", "-pre"),
            |(_, ornithe)| ornithe.to_string(),
        )
}

struct GameVersion {
    id: String,
    ornithe_id: String,
    stable: bool,
    /// The URL of Mojang's version JSON.
    url: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameVersionInfo {
    libraries: Vec<Library>,
    java_version: Option<JavaVersion>,
}

struct GameVersionUpgrades<'a> {
    game_version: &'a GameVersion,
    libraries: Vec<Library>,
    removed_libraries: Vec<String>,
    java_version: Option<JavaVersion>,
}

fn library_artifact(library: &Library) -> &str {
    library
        .name
        .rsplit_once(':')
        .map_or(library.name.as_str(), |x| x.0)
}

/// The artifacts of Mojang's libraries that Ornithe's version JSON drops.
fn removed_libraries(
    mojang_libraries: &[Library],
    ornithe_libraries: &[Library],
) -> Vec<String> {
    let ornithe_artifacts = ornithe_libraries
        .iter()
        .map(library_artifact)
        .collect::<HashSet<_>>();

    mojang_libraries
        .iter()
        .map(library_artifact)
        .filter(|x| !ornithe_artifacts.contains(x))
        .sorted()
        .dedup()
        .map(str::to_string)
        .collect()
}

fn is_lwjgl_upgrade(library: &Library) -> bool {
    let downloads = library.downloads.as_ref();

    downloads
        .and_then(|x| x.artifact.as_ref())
        .into_iter()
        .chain(
            downloads
                .and_then(|x| x.classifiers.as_ref())
                .into_iter()
                .flat_map(|x| x.values()),
        )
        .map(|x| x.url.as_str())
        .chain(library.url.as_deref())
        .any(|url| url.contains(LWJGL_MAVEN_HOST))
}

struct LibraryGroup {
    id: String,
    libraries: Vec<Library>,
    removed_libraries: Vec<String>,
    /// The Java version Ornithe asks for, in place of the game's.
    java_version: Option<JavaVersion>,
    game_versions: Vec<String>,
    /// Set for the group of a game version Ornithe names differently.
    ornithe_game_version: Option<String>,
}

impl LibraryGroup {
    fn profile_game_version(&self) -> &str {
        self.ornithe_game_version.as_deref().unwrap_or_else(|| {
            self.game_versions
                .iter()
                .max_by_key(|x| x.len())
                .expect("library group should have a game version")
        })
    }
}

fn group_by_libraries<'a>(
    game_versions: impl IntoIterator<Item = GameVersionUpgrades<'a>>,
) -> Vec<LibraryGroup> {
    let mut groups = Vec::<LibraryGroup>::new();

    for GameVersionUpgrades {
        game_version,
        mut libraries,
        removed_libraries,
        java_version,
    } in game_versions
    {
        libraries.sort_by(|a, b| a.name.cmp(&b.name));

        let ornithe_game_version = (game_version.id != game_version.ornithe_id)
            .then(|| game_version.ornithe_id.clone());
        let java = java_version
            .as_ref()
            .map(|x| format!("java:{}:{}", x.component, x.major_version));
        let removed = removed_libraries
            .iter()
            .map(|x| format!("removed:{x}"))
            .collect::<Vec<_>>();
        let id = sha1_smol::Sha1::from(
            libraries
                .iter()
                .map(|x| x.name.as_str())
                .chain(ornithe_game_version.as_deref())
                .chain(java.as_deref())
                .chain(removed.iter().map(String::as_str))
                .join("\n"),
        )
        .hexdigest()[..8]
            .to_string();

        if let Some(group) = groups.iter_mut().find(|x| x.id == id) {
            group.game_versions.push(game_version.id.clone());
        } else {
            groups.push(LibraryGroup {
                id,
                libraries,
                removed_libraries,
                java_version,
                game_versions: vec![game_version.id.clone()],
                ornithe_game_version,
            });
        }
    }

    groups
}

#[tracing::instrument(skip(semaphore))]
pub async fn fetch(semaphore: Arc<Semaphore>) -> Result<FetchResult, Error> {
    let format_version = CURRENT_ORNITHE_FORMAT_VERSION;
    let upload_files = DashMap::new();
    let mirror_artifacts = DashMap::<String, MirrorArtifact>::new();
    let manifest_path = format!("{MOD_LOADER}/v{format_version}/manifest.json");

    let modrinth_manifest = fetch_optional_json::<Manifest>(
        &format_url(&manifest_path),
        &semaphore,
    )
    .await?;
    let minecraft_manifest =
        fetch_json::<VersionManifest>(VERSION_MANIFEST_URL, &semaphore).await?;
    let loaders = fetch_json::<Vec<FabricLoaderVersion>>(
        &format!("{META_URL}/fabric-loader"),
        &semaphore,
    )
    .await?;
    let game_versions = fetch_json::<Vec<FabricGameVersion>>(
        &format!("{META_URL}/game"),
        &semaphore,
    )
    .await?
    .into_iter()
    .filter_map(|x| {
        let mojang_version = minecraft_manifest
            .versions
            .iter()
            .find(|y| ornithe_game_version(&y.id) == x.version)?;

        Some(GameVersion {
            id: mojang_version.id.clone(),
            ornithe_id: x.version,
            stable: x.stable,
            url: mojang_version.url.clone(),
        })
    })
    .collect::<Vec<_>>();

    let library_upgrades =
        futures::future::join_all(game_versions.iter().map(|x| {
            let semaphore = &semaphore;
            async move {
                let upgrades = async {
                    let mut libraries = fetch_json::<Vec<Library>>(
                        &format!("{META_URL}/libraries/{}", x.ornithe_id),
                        semaphore,
                    )
                    .await?;
                    let version_info = fetch_json::<GameVersionInfo>(
                        &format!("{MC_VERSIONS_URL}/{}.json", x.ornithe_id),
                        semaphore,
                    )
                    .await?;
                    let mojang_version_info =
                        fetch_json::<GameVersionInfo>(&x.url, semaphore)
                            .await?;
                    let removed_libraries = removed_libraries(
                        &mojang_version_info.libraries,
                        &version_info.libraries,
                    );

                    libraries.extend(
                        version_info
                            .libraries
                            .into_iter()
                            .filter(is_lwjgl_upgrade),
                    );

                    Ok::<_, Error>(GameVersionUpgrades {
                        game_version: x,
                        libraries,
                        removed_libraries,
                        java_version: version_info.java_version,
                    })
                }
                .await;

                match upgrades {
                    Ok(upgrades) => Some(upgrades),
                    Err(err) => {
                        tracing::warn!(
                            game_version = %x.ornithe_id,
                            error = %err,
                            "Skipping Ornithe game version"
                        );
                        None
                    }
                }
            }
        }))
        .await;
    let groups = group_by_libraries(library_upgrades.into_iter().flatten());

    let published_profiles = modrinth_manifest
        .iter()
        .flat_map(|x| &x.version_groups)
        .flat_map(|group| {
            group
                .loaders
                .iter()
                .map(|loader| (group.id.as_str(), loader.id.as_str()))
        })
        .collect::<HashSet<_>>();
    let published_game_versions = modrinth_manifest
        .iter()
        .flat_map(|x| &x.game_versions)
        .filter_map(|x| Some((x.version_group.as_deref()?, x.id.as_str())))
        .collect::<HashSet<_>>();

    let published_profiles = &published_profiles;
    let published_game_versions = &published_game_versions;
    let profile_requests = groups
        .iter()
        .flat_map(|group| {
            let group_unchanged = group.game_versions.iter().all(|x| {
                published_game_versions
                    .contains(&(group.id.as_str(), x.as_str()))
            });

            loaders
                .iter()
                .filter(move |loader| {
                    !group_unchanged
                        || !published_profiles.contains(&(
                            group.id.as_str(),
                            loader.version.as_str(),
                        ))
                })
                .map(move |loader| ProfileRequest {
                    group: group.id.clone(),
                    loader_profile_template_game_version: group
                        .ornithe_game_version
                        .is_none()
                        .then(|| group.profile_game_version().to_string()),
                    game_versions: group.game_versions.clone(),
                    loader_version: loader.version.clone(),
                    url: format!(
                        "{META_URL}/fabric-loader/{}/{}/profile/json",
                        group.profile_game_version(),
                        loader.version
                    ),
                    extra_libraries: group.libraries.clone(),
                    removed_libraries: group.removed_libraries.clone(),
                    java_version: group.java_version.clone(),
                })
        })
        .collect::<Vec<_>>();

    let failed_profiles = fetch_metadata_profiles(
        MOD_LOADER,
        format_version,
        MAVEN_URL,
        profile_requests,
        true,
        &upload_files,
        &mirror_artifacts,
        &semaphore,
    )
    .await?;

    let mut manifest = Manifest {
        game_versions: game_versions
            .into_iter()
            .filter_map(|game_version| {
                let group = groups
                    .iter()
                    .find(|x| x.game_versions.contains(&game_version.id))?;

                Some(Version {
                    version_group: Some(group.id.clone()),
                    id: game_version.id,
                    stable: game_version.stable,
                    loaders: Vec::new(),
                })
            })
            .collect(),
        version_groups: groups
            .iter()
            .map(|group| VersionGroup {
                id: group.id.clone(),
                loaders: loaders
                    .iter()
                    .filter(|loader| {
                        !failed_profiles.contains(&(
                            group.id.clone(),
                            loader.version.clone(),
                        ))
                    })
                    .map(|loader| LoaderVersion {
                        id: loader.version.clone(),
                        url: format_url(&metadata_version_path(
                            MOD_LOADER,
                            format_version,
                            &loader.version,
                            &group.id,
                        )),
                        stable: loader.stable,
                    })
                    .collect(),
            })
            .collect(),
    };

    retain_manifest_versions(&mut manifest, modrinth_manifest.as_ref());

    let used_groups = manifest
        .game_versions
        .iter()
        .filter_map(|x| x.version_group.clone())
        .collect::<HashSet<_>>();
    manifest
        .version_groups
        .retain(|x| used_groups.contains(&x.id));

    upload_files.insert(
        manifest_path,
        UploadFile {
            file: bytes::Bytes::from(serde_json::to_vec(&manifest)?),
            content_type: Some("application/json".to_string()),
        },
    );

    Ok(FetchResult {
        upload_files,
        mirror_artifacts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn libraries(names: &[&str]) -> Vec<Library> {
        names
            .iter()
            .map(|name| {
                serde_json::from_value(serde_json::json!({ "name": name }))
                    .unwrap()
            })
            .collect()
    }

    fn upgrades<'a>(
        game_version: &'a GameVersion,
        library_names: &[&str],
        java_major_version: Option<u32>,
    ) -> GameVersionUpgrades<'a> {
        GameVersionUpgrades {
            game_version,
            libraries: libraries(library_names),
            removed_libraries: Vec::new(),
            java_version: java_major_version.map(|major_version| JavaVersion {
                component: "jre-legacy".to_string(),
                major_version,
            }),
        }
    }

    fn game_version(id: &str) -> GameVersion {
        GameVersion {
            id: id.to_string(),
            ornithe_id: ornithe_game_version(id),
            stable: true,
            url: String::new(),
        }
    }

    #[test]
    fn game_versions_with_the_same_upgrades_share_a_group() {
        let game_versions =
            ["1.8.9", "13w16a-04192037", "1.14.4"].map(game_version);
        let groups = group_by_libraries([
            upgrades(&game_versions[0], &["a:a:1", "b:b:1"], Some(8)),
            upgrades(&game_versions[1], &["b:b:1", "a:a:1"], Some(8)),
            upgrades(&game_versions[2], &["a:a:1"], Some(8)),
        ]);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].game_versions, ["1.8.9", "13w16a-04192037"]);
        assert_eq!(groups[0].profile_game_version(), "13w16a-04192037");
        assert_eq!(groups[1].game_versions, ["1.14.4"]);
        assert_ne!(groups[0].id, groups[1].id);
    }

    #[test]
    fn renamed_game_versions_get_an_untemplated_group() {
        let game_versions =
            ["1.0.1", "1.0", "1.14.2 Pre-Release 4"].map(game_version);
        let groups = group_by_libraries(
            game_versions
                .iter()
                .map(|x| upgrades(x, &["a:a:1"], Some(8))),
        );

        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].ornithe_game_version, None);
        assert_eq!(groups[1].game_versions, ["1.0"]);
        assert_eq!(groups[1].profile_game_version(), "1.0.0");
        assert_eq!(groups[2].profile_game_version(), "1.14.2-pre4");
    }

    #[test]
    fn game_versions_asking_for_different_java_get_separate_groups() {
        let game_versions = ["1.8.9", "1.12.2"].map(game_version);
        let groups = group_by_libraries([
            upgrades(&game_versions[0], &["a:a:1"], Some(8)),
            upgrades(&game_versions[1], &["a:a:1"], Some(25)),
        ]);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[1].java_version.as_ref().unwrap().major_version, 25);
        assert_ne!(groups[0].id, groups[1].id);
    }

    #[test]
    fn libraries_ornithe_drops_are_removed() {
        let removed = removed_libraries(
            &libraries(&[
                "net.minecraft:launchwrapper:1.5",
                "org.ow2.asm:asm-all:4.1",
                "org.lwjgl.lwjgl:lwjgl:2.9.0",
                "org.lwjgl.lwjgl:lwjgl:2.9.1-nightly-20130708-debug3",
            ]),
            &libraries(&[
                "org.ow2.asm:asm:7.0",
                "net.minecraft:launchwrapper:1.5",
                "org.lwjgl.lwjgl:lwjgl:2.9.4+legacyfabric.15",
            ]),
        );

        assert_eq!(removed, ["org.ow2.asm:asm-all"]);
    }

    #[test]
    fn game_versions_dropping_different_libraries_get_separate_groups() {
        let game_versions = ["1.5.2", "1.6.4"].map(game_version);
        let mut dropping_asm_all =
            upgrades(&game_versions[0], &["a:a:1"], Some(8));
        dropping_asm_all.removed_libraries =
            vec!["org.ow2.asm:asm-all".to_string()];
        let groups = group_by_libraries([
            dropping_asm_all,
            upgrades(&game_versions[1], &["a:a:1"], Some(8)),
        ]);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].removed_libraries, ["org.ow2.asm:asm-all"]);
        assert!(groups[1].removed_libraries.is_empty());
    }

    #[test]
    fn only_legacy_fabric_libraries_are_lwjgl_upgrades() {
        let libraries: Vec<Library> = serde_json::from_value(serde_json::json!([
            {
                "name": "org.lwjgl.lwjgl:lwjgl-platform:2.9.4+legacyfabric.15",
                "downloads": { "classifiers": { "natives-osx": {
                    "sha1": "a",
                    "size": 1,
                    "url": "https://maven.legacyfabric.net/natives-osx.jar"
                } } },
                "natives": { "osx-arm64": "natives-osx" }
            },
            {
                "name": "net.java.jinput:jinput:2.0.5",
                "downloads": { "artifact": {
                    "sha1": "b",
                    "size": 2,
                    "url": "https://libraries.minecraft.net/jinput.jar"
                } }
            }
        ]))
        .unwrap();

        assert!(is_lwjgl_upgrade(&libraries[0]));
        assert!(!is_lwjgl_upgrade(&libraries[1]));
    }
}
