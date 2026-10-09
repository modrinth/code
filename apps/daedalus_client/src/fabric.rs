//! Fetches Fabric-compatible loader metadata.
//!
//! Fabric and Quilt both expose loader profiles for a concrete Minecraft
//! version, but Daedalus publishes templated profiles using
//! `${modrinth.gameVersion}`. A group is a set of Minecraft versions whose
//! upstream loader profiles have the same structure after the concrete
//! Minecraft version is replaced with `${modrinth.gameVersion}`. Fabric uses
//! one universal group, so its public profile paths stay as
//! `versions/{loader}.json`. Quilt has more than one group: versions before
//! 26.x include hashed/intermediary libraries, while 26.x versions do not. For
//! Quilt, Daedalus writes one templated profile per group at
//! `version-group/{group}/loader-version/{loader}`.

use crate::metadata_groups::{
    UNIVERSAL_METADATA_GROUP, metadata_group_for_game_version, metadata_groups,
};
use crate::util::{
    download_file, fetch_json, fetch_optional_json, format_url,
    retain_manifest_versions,
};
use crate::{
    Error, FetchResult, MirrorArtifact, UploadFile, insert_mirrored_artifact,
};
use daedalus::minecraft::{Argument, JavaVersion, Library};
use daedalus::modded::{DUMMY_REPLACE_STRING, Manifest, PartialVersionInfo};
use dashmap::DashMap;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Semaphore;

#[tracing::instrument(skip(semaphore))]
pub async fn fetch_fabric(
    semaphore: Arc<Semaphore>,
) -> Result<FetchResult, Error> {
    fetch(
        daedalus::modded::CURRENT_FABRIC_FORMAT_VERSION,
        "fabric",
        "https://meta.fabricmc.net/v2",
        "https://maven.fabricmc.net/",
        &[],
        semaphore,
    )
    .await
}

#[tracing::instrument(skip(semaphore))]
pub async fn fetch_quilt(
    semaphore: Arc<Semaphore>,
) -> Result<FetchResult, Error> {
    fetch(
        daedalus::modded::CURRENT_QUILT_FORMAT_VERSION,
        "quilt",
        "https://meta.quiltmc.org/v3",
        "https://maven.quiltmc.org/repository/release/",
        &[
            // This version is broken as it contains invalid library coordinates
            "0.17.5-beta.4",
        ],
        semaphore,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip(semaphore))]
async fn fetch(
    format_version: usize,
    mod_loader: &str,
    meta_url: &str,
    maven_url: &str,
    skip_versions: &[&str],
    semaphore: Arc<Semaphore>,
) -> Result<FetchResult, Error> {
    let upload_files = DashMap::new();
    let mirror_artifacts = DashMap::<String, MirrorArtifact>::new();
    let modrinth_manifest = fetch_optional_json::<Manifest>(
        &format_url(&format!("{mod_loader}/v{format_version}/manifest.json",)),
        &semaphore,
    )
    .await?;
    let fabric_manifest = fetch_json::<FabricVersions>(
        &format!("{meta_url}/versions"),
        &semaphore,
    )
    .await?;
    let all_loader_versions = fabric_manifest.loader.clone();
    let all_game_versions = fabric_manifest.game.clone();
    let metadata_groups = metadata_groups(
        mod_loader,
        all_game_versions.iter().map(|x| x.version.as_str()),
    );

    if metadata_groups
        .iter()
        .any(|group| group.id != UNIVERSAL_METADATA_GROUP)
    {
        let loaders = all_loader_versions
            .iter()
            .filter(|x| !skip_versions.contains(&&*x.version))
            .collect::<Vec<_>>();

        let profile_requests = metadata_groups
            .iter()
            .flat_map(|group| {
                loaders.iter().map(move |loader| ProfileRequest {
                    group: group.id.to_string(),
                    loader_profile_template_game_version: Some(
                        group.loader_profile_template_game_version.clone(),
                    ),
                    game_versions: group.game_versions.clone(),
                    loader_version: loader.version.clone(),
                    extra_libraries: Vec::new(),
                    java_version: None,
                    url: format!(
                        "{}/versions/loader/{}/{}/profile/json",
                        meta_url,
                        group.loader_profile_template_game_version,
                        loader.version
                    ),
                })
            })
            .collect::<Vec<_>>();

        fetch_metadata_profiles(
            mod_loader,
            format_version,
            maven_url,
            profile_requests,
            &upload_files,
            &mirror_artifacts,
            &semaphore,
        )
        .await?;

        let version_groups = metadata_groups
            .iter()
            .map(|group| daedalus::modded::VersionGroup {
                id: group.id.to_string(),
                loaders: loaders
                    .iter()
                    .map(|loader| {
                        let version_path = metadata_version_path(
                            mod_loader,
                            format_version,
                            &loader.version,
                            group.id,
                        );

                        daedalus::modded::LoaderVersion {
                            id: loader.version.clone(),
                            url: format_url(&version_path),
                            stable: loader.stable,
                        }
                    })
                    .collect(),
            })
            .collect();

        let mut manifest = daedalus::modded::Manifest {
            game_versions: all_game_versions
                .into_iter()
                .map(|game_version| {
                    let group = metadata_group_for_game_version(
                        &metadata_groups,
                        mod_loader,
                        &game_version.version,
                    )
                    .expect("game version should have a metadata group");

                    daedalus::modded::Version {
                        id: game_version.version.clone(),
                        stable: game_version.stable,
                        version_group: Some(group.id.to_string()),
                        loaders: Vec::new(),
                    }
                })
                .collect(),
            version_groups,
        };

        retain_manifest_versions(&mut manifest, modrinth_manifest.as_ref());

        upload_files.insert(
            format!("{mod_loader}/v{format_version}/manifest.json"),
            UploadFile {
                file: bytes::Bytes::from(serde_json::to_vec(&manifest)?),
                content_type: Some("application/json".to_string()),
            },
        );

        return Ok(FetchResult {
            upload_files,
            mirror_artifacts,
        });
    }
    // We check Modrinth's manifest to find newly added loader versions,
    // intermediary/mapping artifacts, and game versions.
    let (
        fetch_fabric_versions,
        fetch_intermediary_versions,
        has_new_game_versions,
    ) = if let Some(modrinth_manifest) = modrinth_manifest.as_ref() {
        let (mut fetch_versions, mut fetch_intermediary_versions) =
            (Vec::new(), Vec::new());

        for version in &fabric_manifest.loader {
            if !modrinth_manifest
                .game_versions
                .iter()
                .any(|x| x.loaders.iter().any(|x| x.id == version.version))
                && !skip_versions.contains(&&*version.version)
            {
                fetch_versions.push(version);
            }
        }

        for version in &fabric_manifest.intermediary {
            if !modrinth_manifest
                .game_versions
                .iter()
                .any(|x| x.id == version.version)
                && fabric_manifest
                    .game
                    .iter()
                    .any(|x| x.version == version.version)
            {
                fetch_intermediary_versions.push(version);
            }
        }

        let has_new_game_versions =
            fabric_manifest.game.iter().any(|version| {
                !modrinth_manifest
                    .game_versions
                    .iter()
                    .any(|x| x.id == version.version)
            });

        (
            fetch_versions,
            fetch_intermediary_versions,
            has_new_game_versions,
        )
    } else {
        (
            fabric_manifest
                .loader
                .iter()
                .filter(|x| !skip_versions.contains(&&*x.version))
                .collect(),
            fabric_manifest.intermediary.iter().collect(),
            true,
        )
    };

    if !fetch_intermediary_versions.is_empty() {
        for x in &fetch_intermediary_versions {
            insert_mirrored_artifact(
                &x.maven,
                None,
                vec![maven_url.to_string()],
                false,
                &mirror_artifacts,
            )?;
        }
    }

    if !fetch_fabric_versions.is_empty() {
        let universal_group = metadata_groups
            .iter()
            .find(|group| group.id == UNIVERSAL_METADATA_GROUP)
            .expect("fabric metadata should have a universal group");
        let profile_requests = fetch_fabric_versions
            .iter()
            .map(|loader| ProfileRequest {
                group: universal_group.id.to_string(),
                loader_profile_template_game_version: Some(
                    universal_group
                        .loader_profile_template_game_version
                        .clone(),
                ),
                game_versions: universal_group.game_versions.clone(),
                loader_version: loader.version.clone(),
                extra_libraries: Vec::new(),
                java_version: None,
                url: format!(
                    "{}/versions/loader/{}/{}/profile/json",
                    meta_url,
                    universal_group.loader_profile_template_game_version,
                    loader.version
                ),
            })
            .collect::<Vec<_>>();

        fetch_metadata_profiles(
            mod_loader,
            format_version,
            maven_url,
            profile_requests,
            &upload_files,
            &mirror_artifacts,
            &semaphore,
        )
        .await?;
    }

    if !fetch_fabric_versions.is_empty()
        || !fetch_intermediary_versions.is_empty()
        || has_new_game_versions
    {
        let fabric_manifest_path =
            format!("{mod_loader}/v{format_version}/manifest.json",);

        let loader_versions = daedalus::modded::Version {
            id: DUMMY_REPLACE_STRING.to_string(),
            stable: true,
            version_group: None,
            loaders: all_loader_versions
                .iter()
                .filter(|x| !skip_versions.contains(&&*x.version))
                .map(|x| {
                    let version_path = metadata_version_path(
                        mod_loader,
                        format_version,
                        &x.version,
                        UNIVERSAL_METADATA_GROUP,
                    );

                    daedalus::modded::LoaderVersion {
                        id: x.version.clone(),
                        url: format_url(&version_path),
                        stable: x.stable,
                    }
                })
                .collect(),
        };

        let mut manifest = daedalus::modded::Manifest {
            game_versions: std::iter::once(loader_versions)
                .chain(all_game_versions.into_iter().map(|x| {
                    daedalus::modded::Version {
                        id: x.version,
                        stable: x.stable,
                        version_group: None,
                        loaders: vec![],
                    }
                }))
                .collect(),
            version_groups: Vec::new(),
        };

        retain_manifest_versions(&mut manifest, modrinth_manifest.as_ref());

        upload_files.insert(
            fabric_manifest_path,
            UploadFile {
                file: bytes::Bytes::from(serde_json::to_vec(&manifest)?),
                content_type: Some("application/json".to_string()),
            },
        );
    }

    Ok(FetchResult {
        upload_files,
        mirror_artifacts,
    })
}

pub(crate) struct ProfileRequest {
    pub group: String,
    pub loader_profile_template_game_version: Option<String>,
    pub game_versions: Vec<String>,
    pub loader_version: String,
    pub url: String,
    pub extra_libraries: Vec<Library>,
    /// The Java version this group's profiles prefer over the game's.
    pub java_version: Option<JavaVersion>,
}

pub(crate) fn metadata_version_path(
    mod_loader: &str,
    format_version: usize,
    loader_version: &str,
    group: &str,
) -> String {
    if group == UNIVERSAL_METADATA_GROUP {
        format!("{mod_loader}/v{format_version}/versions/{loader_version}.json")
    } else {
        format!(
            "{mod_loader}/v{format_version}/version-group/{group}/loader-version/{loader_version}"
        )
    }
}

pub(crate) async fn fetch_metadata_profiles(
    mod_loader: &str,
    format_version: usize,
    maven_url: &str,
    profile_requests: Vec<ProfileRequest>,
    upload_files: &DashMap<String, UploadFile>,
    mirror_artifacts: &DashMap<String, MirrorArtifact>,
    semaphore: &Arc<Semaphore>,
) -> Result<(), Error> {
    let version_manifests = futures::future::try_join_all(
        profile_requests
            .iter()
            .map(|x| download_file(&x.url, None, semaphore)),
    )
    .await?
    .into_iter()
    .map(|x| serde_json::from_slice(&x))
    .collect::<Result<Vec<PartialVersionInfo>, serde_json::Error>>()?;

    let patched_version_manifests = version_manifests
        .into_iter()
        .zip(profile_requests.iter())
        .map(|(mut version_info, request)| {
            patch_version_info(
                &mut version_info,
                request.loader_profile_template_game_version.as_deref(),
                &request.game_versions,
                maven_url,
                &request.extra_libraries,
                mirror_artifacts,
            )?;
            version_info.java_version.clone_from(&request.java_version);

            Ok(version_info)
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let serialized_version_manifests = patched_version_manifests
        .iter()
        .map(|x| serde_json::to_vec(x).map(bytes::Bytes::from))
        .collect::<Result<Vec<_>, serde_json::Error>>()?;

    serialized_version_manifests
        .into_iter()
        .zip(profile_requests)
        .for_each(|(bytes, request)| {
            let version_path = metadata_version_path(
                mod_loader,
                format_version,
                &request.loader_version,
                &request.group,
            );

            upload_files.insert(
                version_path,
                UploadFile {
                    file: bytes,
                    content_type: Some("application/json".to_string()),
                },
            );
        });

    Ok(())
}

fn template_game_version(value: &str, game_version: Option<&str>) -> String {
    let Some(game_version) = game_version else {
        return value.to_string();
    };
    let is_boundary =
        |x: Option<char>| x.is_none_or(|x| !x.is_alphanumeric() && x != '.');
    let mut templated = String::with_capacity(value.len());
    let mut copied = 0;

    for (start, _) in value.match_indices(game_version) {
        let end = start + game_version.len();

        if is_boundary(value[..start].chars().next_back())
            && is_boundary(value[end..].chars().next())
        {
            templated.push_str(&value[copied..start]);
            templated.push_str(DUMMY_REPLACE_STRING);
            copied = end;
        }
    }

    templated.push_str(&value[copied..]);
    templated
}

/// Swaps the version of a library for the placeholder where it is exactly
/// the game version, so `1.14` is left alone in `1.14-pre1`.
fn template_library_name(name: &str, game_version: Option<&str>) -> String {
    name.split(':')
        .map(|x| {
            if Some(x) == game_version {
                DUMMY_REPLACE_STRING
            } else {
                x
            }
        })
        .collect::<Vec<_>>()
        .join(":")
}

fn patch_version_info(
    version_info: &mut PartialVersionInfo,
    game_version: Option<&str>,
    game_versions: &[String],
    maven_url: &str,
    extra_libraries: &[Library],
    mirror_artifacts: &DashMap<String, MirrorArtifact>,
) -> Result<(), Error> {
    for lib in &mut version_info.libraries {
        let new_name = template_library_name(&lib.name, game_version);

        // Hard-code: This library is not present on fabric's maven, so we fetch it from MC libraries
        if &*lib.name == "net.minecraft:launchwrapper:1.12" {
            lib.url = Some("https://libraries.minecraft.net/".to_string());
        }
        let source_url =
            lib.url.clone().unwrap_or_else(|| maven_url.to_string());

        if lib.name == new_name {
            insert_mirrored_artifact(
                &new_name,
                None,
                vec![source_url],
                false,
                mirror_artifacts,
            )?;
        } else {
            for concrete_game_version in game_versions {
                let concrete_name = new_name
                    .replace(DUMMY_REPLACE_STRING, concrete_game_version);

                insert_mirrored_artifact(
                    &concrete_name,
                    None,
                    vec![source_url.clone()],
                    false,
                    mirror_artifacts,
                )?;
            }

            lib.name = new_name;
        }

        lib.url = Some(format_url("maven/"));
    }

    for lib in extra_libraries {
        let mut lib = lib.clone();

        if lib.downloads.is_none() {
            insert_mirrored_artifact(
                &lib.name,
                None,
                vec![lib.url.unwrap_or_else(|| maven_url.to_string())],
                false,
                mirror_artifacts,
            )?;
            lib.url = Some(format_url("maven/"));
        }

        version_info.libraries.push(lib);
    }

    for argument in version_info
        .arguments
        .iter_mut()
        .flat_map(|x| x.values_mut())
        .flatten()
    {
        if let Argument::Normal(value) = argument {
            *value = template_game_version(value, game_version);
        }
    }

    version_info.id = template_game_version(&version_info.id, game_version);
    version_info.inherits_from =
        template_game_version(&version_info.inherits_from, game_version);

    Ok(())
}

#[derive(Deserialize, Debug, Clone)]
struct FabricVersions {
    pub loader: Vec<FabricLoaderVersion>,
    pub game: Vec<FabricGameVersion>,
    #[serde(alias = "hashed")]
    pub intermediary: Vec<FabricIntermediaryVersion>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct FabricLoaderVersion {
    // pub separator: String,
    // pub build: u32,
    // pub maven: String,
    pub version: String,
    #[serde(default)]
    pub stable: bool,
}

#[derive(Deserialize, Debug, Clone)]
struct FabricIntermediaryVersion {
    pub maven: String,
    pub version: String,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct FabricGameVersion {
    pub version: String,
    pub stable: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_versions_inside_longer_versions_are_not_templated() {
        let template = |value| template_game_version(value, Some("1.8"));

        assert_eq!(
            template("fabric-loader-0.11.8-1.8-ornithe-gen2"),
            format!("fabric-loader-0.11.8-{DUMMY_REPLACE_STRING}-ornithe-gen2")
        );
        assert_eq!(template("a:b:1.8.9"), "a:b:1.8.9");
        assert_eq!(template_game_version("a:b:1.8", None), "a:b:1.8");
    }

    #[test]
    fn only_whole_library_versions_are_templated() {
        let template = |name| template_library_name(name, Some("1.8"));

        assert_eq!(
            template("net.ornithemc:calamus-intermediary-gen2:1.8"),
            format!(
                "net.ornithemc:calamus-intermediary-gen2:{DUMMY_REPLACE_STRING}"
            )
        );
        assert_eq!(template("a:b:1.8-pre1"), "a:b:1.8-pre1");
        assert_eq!(template("a:b:0.11.8"), "a:b:0.11.8");
        assert_eq!(template_library_name("a:b:1.8", None), "a:b:1.8");
    }
}
