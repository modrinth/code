use crate::util::{download_file, fetch_json, format_url};
use crate::{
    Error, FetchResult, MirrorArtifact, UploadFile, insert_mirrored_artifact,
};
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
    let modrinth_manifest = fetch_json::<Manifest>(
        &format_url(&format!("{mod_loader}/v{format_version}/manifest.json",)),
        &semaphore,
    )
    .await
    .ok();
    let fabric_manifest = fetch_json::<FabricVersions>(
        &format!("{meta_url}/versions"),
        &semaphore,
    )
    .await?;
    let all_loader_versions = fabric_manifest.loader.clone();
    let all_game_versions = fabric_manifest.game.clone();
    let metadata_groups = metadata_groups(mod_loader, &all_game_versions);

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
                    template_game_version: group.template_game_version.clone(),
                    loader_version: loader.version.clone(),
                    url: format!(
                        "{}/versions/loader/{}/{}/profile/json",
                        meta_url, group.template_game_version, loader.version
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

        let manifest = daedalus::modded::Manifest {
            game_versions: all_game_versions
                .into_iter()
                .map(|game_version| {
                    let group = metadata_group_id_for_game_version(
                        mod_loader,
                        &game_version.version,
                    );

                    daedalus::modded::Version {
                        id: game_version.version.clone(),
                        stable: game_version.stable,
                        loaders: loaders
                            .iter()
                            .map(|loader| {
                                let version_path = metadata_version_path(
                                    mod_loader,
                                    format_version,
                                    &loader.version,
                                    group,
                                );

                                daedalus::modded::LoaderVersion {
                                    id: loader.version.clone(),
                                    url: format_url(&version_path),
                                    stable: loader.stable,
                                }
                            })
                            .collect(),
                    }
                })
                .collect(),
        };

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
    ) = if let Some(modrinth_manifest) = modrinth_manifest {
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
                template_game_version: universal_group
                    .template_game_version
                    .clone(),
                loader_version: loader.version.clone(),
                url: format!(
                    "{}/versions/loader/{}/{}/profile/json",
                    meta_url,
                    universal_group.template_game_version,
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

        let manifest = daedalus::modded::Manifest {
            game_versions: std::iter::once(loader_versions)
                .chain(all_game_versions.into_iter().map(|x| {
                    daedalus::modded::Version {
                        id: x.version,
                        stable: x.stable,
                        loaders: vec![],
                    }
                }))
                .collect(),
        };

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

struct MetadataGroup {
    id: &'static str,
    template_game_version: String,
}

const UNIVERSAL_METADATA_GROUP: &str = "universal";

struct ProfileRequest {
    group: String,
    template_game_version: String,
    loader_version: String,
    url: String,
}

fn metadata_groups(
    mod_loader: &str,
    game_versions: &[FabricGameVersion],
) -> Vec<MetadataGroup> {
    if mod_loader != "quilt" {
        return vec![MetadataGroup {
            id: UNIVERSAL_METADATA_GROUP,
            template_game_version: "1.21".to_string(),
        }];
    }

    let pre_26_game_versions = game_versions
        .iter()
        .filter(|x| {
            metadata_group_id_for_game_version(mod_loader, &x.version)
                == "pre-26-x"
        })
        .cloned()
        .collect::<Vec<_>>();
    let game_versions_26 = game_versions
        .iter()
        .filter(|x| {
            metadata_group_id_for_game_version(mod_loader, &x.version) == "26-x"
        })
        .cloned()
        .collect::<Vec<_>>();

    let mut groups = Vec::new();

    if !pre_26_game_versions.is_empty() {
        groups.push(MetadataGroup {
            id: "pre-26-x",
            template_game_version: pre_26_game_versions
                .iter()
                .find(|x| x.version == "1.21")
                .unwrap_or(&pre_26_game_versions[0])
                .version
                .clone(),
        });
    }

    if !game_versions_26.is_empty() {
        groups.push(MetadataGroup {
            id: "26-x",
            template_game_version: game_versions_26[0].version.clone(),
        });
    }

    groups
}

fn metadata_group_id_for_game_version(
    mod_loader: &str,
    game_version: &str,
) -> &'static str {
    if mod_loader == "quilt"
        && (game_version.starts_with("26.") || game_version.starts_with("26w"))
    {
        "26-x"
    } else if mod_loader == "quilt" {
        "pre-26-x"
    } else {
        UNIVERSAL_METADATA_GROUP
    }
}

fn metadata_version_path(
    mod_loader: &str,
    format_version: usize,
    loader_version: &str,
    group: &str,
) -> String {
    if group == UNIVERSAL_METADATA_GROUP {
        format!("{mod_loader}/v{format_version}/versions/{loader_version}.json")
    } else {
        format!(
            "{mod_loader}/v{format_version}/versions/{loader_version}/{group}.json"
        )
    }
}

async fn fetch_metadata_profiles(
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
                &request.template_game_version,
                maven_url,
                mirror_artifacts,
            )?;

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

fn patch_version_info(
    version_info: &mut PartialVersionInfo,
    game_version: &str,
    maven_url: &str,
    mirror_artifacts: &DashMap<String, MirrorArtifact>,
) -> Result<(), Error> {
    for lib in &mut version_info.libraries {
        let new_name = lib.name.replace(game_version, DUMMY_REPLACE_STRING);

        // Hard-code: This library is not present on fabric's maven, so we fetch it from MC libraries
        if &*lib.name == "net.minecraft:launchwrapper:1.12" {
            lib.url = Some("https://libraries.minecraft.net/".to_string());
        }

        // If a library is not intermediary, we add it to mirror artifacts to be mirrored
        if lib.name == new_name {
            insert_mirrored_artifact(
                &new_name,
                None,
                vec![lib.url.clone().unwrap_or_else(|| maven_url.to_string())],
                false,
                mirror_artifacts,
            )?;
        } else {
            lib.name = new_name;
        }

        lib.url = Some(format_url("maven/"));
    }

    version_info.id =
        version_info.id.replace(game_version, DUMMY_REPLACE_STRING);
    version_info.inherits_from = version_info
        .inherits_from
        .replace(game_version, DUMMY_REPLACE_STRING);

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
struct FabricLoaderVersion {
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
struct FabricGameVersion {
    pub version: String,
    pub stable: bool,
}
