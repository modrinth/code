use crate::{format_url, upload_file_to_bucket, Error};
use daedalus::download_file;
use daedalus::minecraft::Library;
use daedalus::modded::{LoaderType, LoaderVersion, Manifest, PartialVersionInfo, Version};
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

pub async fn retrieve_data(uploaded_files: &mut Vec<String>) -> Result<(), Error> {
    let mut list = fetch_fabric_versions(None).await?;
    let old_manifest = daedalus::modded::fetch_manifest(&*format!(
        "fabric/v{}/manifest.json",
        daedalus::modded::CURRENT_FABRIC_FORMAT_VERSION,
    ))
    .await
    .ok();

    let versions = Arc::new(Mutex::new(if let Some(old_manifest) = old_manifest {
        old_manifest.game_versions
    } else {
        Vec::new()
    }));

    let uploaded_files_mutex = Arc::new(Mutex::new(Vec::new()));

    if let Some(latest) = list.loader.get(0) {
        let loaders_mutex = Arc::new(Mutex::new(HashMap::new()));
        let visited_artifacts_mutex = Arc::new(Mutex::new(Vec::new()));

        {
            let mut loaders = loaders_mutex.lock().await;

            loaders.insert(LoaderType::Latest, latest.version.clone());

            if !latest.stable {
                if let Some(stable) = list.loader.iter().find(|x| x.stable) {
                    loaders.insert(LoaderType::Stable, stable.version.clone());
                }
            }

            list.loader = list
                .loader
                .into_iter()
                .filter(|x| loaders.values().any(|val| val == &x.version))
                .collect();
        }

        let mut version_futures = Vec::new();

        for game_version in list.game.iter_mut() {
            let visited_artifacts_mutex = Arc::clone(&visited_artifacts_mutex);
            let loaders_mutex = Arc::clone(&loaders_mutex);
            let uploaded_files_mutex = Arc::clone(&uploaded_files_mutex);

            let versions_mutex = Arc::clone(&versions);
            version_futures.push(async move {
                let loader_version_mutex = Mutex::new(HashMap::new());

                let versions =
                    futures::future::try_join_all(
                        loaders_mutex.lock().await.clone().into_iter().map(
                            |(type_, loader)| async {
                                {
                                    if versions_mutex.lock().await.iter().any(|x| {
                                        x.id == game_version.version
                                            && x.loaders
                                                .get(&type_)
                                                .map(|x| x.id == loader)
                                                .unwrap_or(false)
                                    }) {
                                        return Ok(None);
                                    }
                                }

                                let version =
                                    fetch_fabric_version(&*game_version.version, &*loader).await?;

                                Ok::<Option<(LoaderType, String, PartialVersionInfo)>, Error>(Some(
                                    (type_, loader, version),
                                ))
                            },
                        ),
                    )
                    .await?
                    .into_iter()
                    .flatten();

                futures::future::try_join_all(versions.map(|(type_, loader, version)| async {
                    let libs = futures::future::try_join_all(version.libraries.into_iter().map(
                        |mut lib| async {
                            {
                                let mut visited_assets = visited_artifacts_mutex.lock().await;

                                if visited_assets.contains(&lib.name) {
                                    lib.url = Some(format_url("maven/"));

                                    return Ok(lib);
                                } else {
                                    visited_assets.push(lib.name.clone())
                                }
                            }

                            let artifact_path = daedalus::get_path_from_artifact(&*lib.name)?;

                            let artifact = daedalus::download_file(
                                &*format!(
                                    "{}{}",
                                    lib.url.unwrap_or_else(|| {
                                        "https://maven.fabricmc.net/".to_string()
                                    }),
                                    artifact_path
                                ),
                                None,
                            )
                            .await?;

                            lib.url = Some(format_url("maven/"));

                            upload_file_to_bucket(
                                format!("{}/{}", "maven", artifact_path),
                                artifact.to_vec(),
                                Some("application/java-archive".to_string()),
                                uploaded_files_mutex.as_ref(),
                            )
                            .await?;

                            Ok::<Library, Error>(lib)
                        },
                    ))
                    .await?;

                    let version_path = format!(
                        "fabric/v{}/versions/{}-{}.json",
                        daedalus::modded::CURRENT_FABRIC_FORMAT_VERSION,
                        version.inherits_from,
                        &loader
                    );

                    let inherits_from = version.inherits_from.clone();

                    upload_file_to_bucket(
                        version_path.clone(),
                        serde_json::to_vec(&PartialVersionInfo {
                            arguments: version.arguments,
                            id: version.id,
                            main_class: version.main_class,
                            release_time: version.release_time,
                            time: version.time,
                            type_: version.type_,
                            inherits_from: version.inherits_from,
                            libraries: libs,
                            minecraft_arguments: version.minecraft_arguments,
                            processors: None,
                            data: None,
                        })?,
                        Some("application/json".to_string()),
                        uploaded_files_mutex.as_ref(),
                    )
                    .await?;

                    {
                        let mut loader_version_map = loader_version_mutex.lock().await;
                        async move {
                            loader_version_map.insert(
                                type_,
                                LoaderVersion {
                                    id: format!("{}-{}", inherits_from, loader),
                                    url: format_url(&*version_path),
                                },
                            );
                        }
                        .await;
                    }

                    Ok::<(), Error>(())
                }))
                .await?;

                let mut versions = versions_mutex.lock().await;
                versions.push(Version {
                    id: game_version.version.clone(),
                    loaders: loader_version_mutex.into_inner(),
                });

                Ok::<(), Error>(())
            });
        }

        let mut versions = version_futures.into_iter().peekable();
        let mut chunk_index = 0;
        while versions.peek().is_some() {
            let now = Instant::now();

            let chunk: Vec<_> = versions.by_ref().take(10).collect();
            futures::future::try_join_all(chunk).await?;

            tokio::time::sleep(Duration::from_secs(1)).await;

            chunk_index += 1;

            let elapsed = now.elapsed();
            info!("Chunk {} Elapsed: {:.2?}", chunk_index, elapsed);
        }
    }

    if let Ok(versions) = Arc::try_unwrap(versions) {
        upload_file_to_bucket(
            format!(
                "fabric/v{}/manifest.json",
                daedalus::modded::CURRENT_FABRIC_FORMAT_VERSION,
            ),
            serde_json::to_vec(&Manifest {
                game_versions: versions.into_inner(),
            })?,
            Some("application/json".to_string()),
            uploaded_files_mutex.as_ref(),
        )
        .await?;
    }

    if let Ok(uploaded_files_mutex) = Arc::try_unwrap(uploaded_files_mutex) {
        uploaded_files.extend(uploaded_files_mutex.into_inner());
    }

    Ok(())
}

const FABRIC_META_URL: &str = "https://meta.fabricmc.net/v2";

async fn fetch_fabric_version(
    version_number: &str,
    loader_version: &str,
) -> Result<PartialVersionInfo, Error> {
    Ok(serde_json::from_slice(
        &download_file(
            &*format!(
                "{}/versions/loader/{}/{}/profile/json",
                FABRIC_META_URL, version_number, loader_version
            ),
            None,
        )
        .await?,
    )?)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Versions of fabric components
struct FabricVersions {
    /// Versions of Minecraft that fabric supports
    pub game: Vec<FabricGameVersion>,
    /// Available versions of the fabric loader
    pub loader: Vec<FabricLoaderVersion>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A version of Minecraft that fabric supports
struct FabricGameVersion {
    /// The version number of the game
    pub version: String,
    /// Whether the Minecraft version is stable or not
    pub stable: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// A version of the fabric loader
struct FabricLoaderVersion {
    /// The separator to get the build number
    pub separator: String,
    /// The build number
    pub build: u32,
    /// The maven artifact
    pub maven: String,
    /// The version number of the fabric loader
    pub version: String,
    /// Whether the loader is stable or not
    pub stable: bool,
}
/// Fetches the list of fabric versions
async fn fetch_fabric_versions(url: Option<&str>) -> Result<FabricVersions, Error> {
    Ok(serde_json::from_slice(
        &download_file(
            url.unwrap_or(&*format!("{}/versions", FABRIC_META_URL)),
            None,
        )
        .await?,
    )?)
}
