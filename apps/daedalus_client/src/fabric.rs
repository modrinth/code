use crate::util::{download_file, fetch_json, format_url};
use crate::{Error, MirrorArtifact, UploadFile, insert_mirrored_artifact};
use daedalus::modded::{DUMMY_REPLACE_STRING, Manifest, PartialVersionInfo};
use dashmap::DashMap;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Semaphore;

#[tracing::instrument(skip(semaphore, upload_files, mirror_artifacts))]
pub async fn fetch_fabric(
    semaphore: Arc<Semaphore>,
    upload_files: &DashMap<String, UploadFile>,
    mirror_artifacts: &DashMap<String, MirrorArtifact>,
) -> Result<(), Error> {
    fetch(
        daedalus::modded::CURRENT_FABRIC_FORMAT_VERSION,
        "fabric",
        "https://meta.fabricmc.net/v2",
        "https://maven.fabricmc.net/",
        &[],
        semaphore,
        upload_files,
        mirror_artifacts,
    )
    .await
}

#[tracing::instrument(skip(semaphore, upload_files, mirror_artifacts))]
pub async fn fetch_quilt(
    semaphore: Arc<Semaphore>,
    upload_files: &DashMap<String, UploadFile>,
    mirror_artifacts: &DashMap<String, MirrorArtifact>,
) -> Result<(), Error> {
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
        upload_files,
        mirror_artifacts,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip(semaphore, upload_files, mirror_artifacts))]
async fn fetch(
    format_version: usize,
    mod_loader: &str,
    meta_url: &str,
    maven_url: &str,
    skip_versions: &[&str],
    semaphore: Arc<Semaphore>,
    upload_files: &DashMap<String, UploadFile>,
    mirror_artifacts: &DashMap<String, MirrorArtifact>,
) -> Result<(), Error> {
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

    // We check Modrinth's fabric version manifest and compare if the fabric version exists in Modrinth's database
    // We also check intermediary versions that are newly added to query
    let (fetch_fabric_versions, fetch_intermediary_versions) =
        if let Some(modrinth_manifest) = modrinth_manifest {
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

            (fetch_versions, fetch_intermediary_versions)
        } else {
            (
                fabric_manifest
                    .loader
                    .iter()
                    .filter(|x| !skip_versions.contains(&&*x.version))
                    .collect(),
                fabric_manifest.intermediary.iter().collect(),
            )
        };

    const DUMMY_GAME_VERSION: &str = "1.21";

    if !fetch_intermediary_versions.is_empty() {
        for x in &fetch_intermediary_versions {
            insert_mirrored_artifact(
                &x.maven,
                None,
                vec![maven_url.to_string()],
                false,
                mirror_artifacts,
            )?;
        }
    }

    if !fetch_fabric_versions.is_empty() {
        let fabric_version_manifest_urls = fetch_fabric_versions
            .iter()
            .map(|x| {
                format!(
                    "{}/versions/loader/{}/{}/profile/json",
                    meta_url, DUMMY_GAME_VERSION, x.version
                )
            })
            .collect::<Vec<_>>();
        let fabric_version_manifests = futures::future::try_join_all(
            fabric_version_manifest_urls
                .iter()
                .map(|x| download_file(x, None, &semaphore)),
        )
        .await?
        .into_iter()
        .map(|x| serde_json::from_slice(&x))
        .collect::<Result<Vec<PartialVersionInfo>, serde_json::Error>>()?;

        let patched_version_manifests = fabric_version_manifests
            .into_iter()
            .map(|mut version_info| {
                for lib in &mut version_info.libraries {
                    let new_name = lib
                        .name
                        .replace(DUMMY_GAME_VERSION, DUMMY_REPLACE_STRING);

                    // Hard-code: This library is not present on fabric's maven, so we fetch it from MC libraries
                    if &*lib.name == "net.minecraft:launchwrapper:1.12" {
                        lib.url = Some(
                            "https://libraries.minecraft.net/".to_string(),
                        );
                    }

                    // If a library is not intermediary, we add it to mirror artifacts to be mirrored
                    if lib.name == new_name {
                        insert_mirrored_artifact(
                            &new_name,
                            None,
                            vec![
                                lib.url
                                    .clone()
                                    .unwrap_or_else(|| maven_url.to_string()),
                            ],
                            false,
                            mirror_artifacts,
                        )?;
                    } else {
                        lib.name = new_name;
                    }

                    lib.url = Some(format_url("maven/"));
                }

                version_info.id = version_info
                    .id
                    .replace(DUMMY_GAME_VERSION, DUMMY_REPLACE_STRING);
                version_info.inherits_from = version_info
                    .inherits_from
                    .replace(DUMMY_GAME_VERSION, DUMMY_REPLACE_STRING);

                Ok(version_info)
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let serialized_version_manifests = patched_version_manifests
            .iter()
            .map(|x| serde_json::to_vec(x).map(bytes::Bytes::from))
            .collect::<Result<Vec<_>, serde_json::Error>>()?;

        serialized_version_manifests
            .into_iter()
            .enumerate()
            .for_each(|(index, bytes)| {
                let loader = fetch_fabric_versions[index];

                let version_path = format!(
                    "{mod_loader}/v{format_version}/versions/{}.json",
                    loader.version
                );

                upload_files.insert(
                    version_path,
                    UploadFile {
                        file: bytes,
                        content_type: Some("application/json".to_string()),
                    },
                );
            });
    }

    if !fetch_fabric_versions.is_empty()
        || !fetch_intermediary_versions.is_empty()
    {
        let fabric_manifest_path =
            format!("{mod_loader}/v{format_version}/manifest.json",);

        let loader_versions = daedalus::modded::Version {
            id: DUMMY_REPLACE_STRING.to_string(),
            stable: true,
            loaders: fabric_manifest
                .loader
                .into_iter()
                .map(|x| {
                    let version_path = format!(
                        "{mod_loader}/v{format_version}/versions/{}.json",
                        x.version,
                    );

                    daedalus::modded::LoaderVersion {
                        id: x.version,
                        url: format_url(&version_path),
                        stable: x.stable,
                    }
                })
                .collect(),
        };

        let manifest = daedalus::modded::Manifest {
            game_versions: std::iter::once(loader_versions)
                .chain(fabric_manifest.game.into_iter().map(|x| {
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
