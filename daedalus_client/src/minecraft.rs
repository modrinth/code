use crate::download_file;
use crate::{format_url, upload_file_to_bucket, Error};
use daedalus::minecraft::VersionManifest;
use log::info;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, Semaphore};

pub async fn retrieve_data(
    uploaded_files: &mut Vec<String>,
    semaphore: Arc<Semaphore>,
) -> Result<VersionManifest, Error> {
    let old_manifest = daedalus::minecraft::fetch_version_manifest(Some(
        &*format_url(&format!(
            "minecraft/v{}/manifest.json",
            daedalus::minecraft::CURRENT_FORMAT_VERSION
        )),
    ))
    .await
    .ok();

    let mut manifest =
        daedalus::minecraft::fetch_version_manifest(None).await?;
    let cloned_manifest = Arc::new(Mutex::new(manifest.clone()));

    let visited_assets_mutex = Arc::new(Mutex::new(Vec::new()));
    let uploaded_files_mutex = Arc::new(Mutex::new(Vec::new()));

    let now = Instant::now();

    let mut version_futures = Vec::new();

    for version in manifest.versions.iter_mut() {
        version_futures.push(async {
            let old_version = if let Some(old_manifest) = &old_manifest {
                old_manifest.versions.iter().find(|x| x.id == version.id)
            } else {
                None
            };

            if let Some(old_version) = old_version {
                if old_version.sha1 == version.sha1 {
                    return Ok(());
                }
            }

            let visited_assets_mutex = Arc::clone(&visited_assets_mutex);
            let cloned_manifest_mutex = Arc::clone(&cloned_manifest);
            let uploaded_files_mutex = Arc::clone(&uploaded_files_mutex);
            let semaphore = Arc::clone(&semaphore);

            let assets_hash =
                old_version.and_then(|x| x.assets_index_sha1.clone());

            async move {
                let mut upload_futures = Vec::new();

                let version_info =
                    daedalus::minecraft::fetch_version_info(version).await?;

                let version_path = format!(
                    "minecraft/v{}/versions/{}.json",
                    daedalus::minecraft::CURRENT_FORMAT_VERSION,
                    version.id
                );
                let assets_path = format!(
                    "minecraft/v{}/assets/{}.json",
                    daedalus::minecraft::CURRENT_FORMAT_VERSION,
                    version_info.asset_index.id
                );
                let assets_index_url = version_info.asset_index.url.clone();

                {
                    let mut cloned_manifest =
                        cloned_manifest_mutex.lock().await;

                    let position = cloned_manifest
                        .versions
                        .iter()
                        .position(|x| version.id == x.id)
                        .unwrap();
                    cloned_manifest.versions[position].url =
                        format_url(&version_path);
                    cloned_manifest.versions[position].assets_index_sha1 =
                        Some(version_info.asset_index.sha1.clone());
                    cloned_manifest.versions[position].assets_index_url =
                        Some(format_url(&assets_path));
                }

                let mut download_assets = false;

                {
                    let mut visited_assets = visited_assets_mutex.lock().await;

                    if !visited_assets.contains(&version_info.asset_index.id) {
                        if let Some(assets_hash) = assets_hash {
                            if version_info.asset_index.sha1 != assets_hash {
                                download_assets = true;
                            }
                        } else {
                            download_assets = true;
                        }
                    }

                    if download_assets {
                        visited_assets
                            .push(version_info.asset_index.id.clone());
                    }
                }

                if download_assets {
                    let assets_index = download_file(
                        &assets_index_url,
                        Some(&version_info.asset_index.sha1),
                        semaphore.clone(),
                    )
                    .await?;

                    {
                        upload_futures.push(upload_file_to_bucket(
                            assets_path,
                            assets_index.to_vec(),
                            Some("application/json".to_string()),
                            uploaded_files_mutex.as_ref(),
                            semaphore.clone(),
                        ));
                    }
                }

                {
                    upload_futures.push(upload_file_to_bucket(
                        version_path,
                        serde_json::to_vec(&version_info)?,
                        Some("application/json".to_string()),
                        uploaded_files_mutex.as_ref(),
                        semaphore.clone(),
                    ));
                }

                futures::future::try_join_all(upload_futures).await?;

                Ok::<(), Error>(())
            }
            .await?;

            Ok::<(), Error>(())
        })
    }

    futures::future::try_join_all(version_futures).await?;

    upload_file_to_bucket(
        format!(
            "minecraft/v{}/manifest.json",
            daedalus::minecraft::CURRENT_FORMAT_VERSION
        ),
        serde_json::to_vec(&*cloned_manifest.lock().await)?,
        Some("application/json".to_string()),
        uploaded_files_mutex.as_ref(),
        semaphore,
    )
    .await?;

    if let Ok(uploaded_files_mutex) = Arc::try_unwrap(uploaded_files_mutex) {
        uploaded_files.extend(uploaded_files_mutex.into_inner());
    }

    let elapsed = now.elapsed();
    info!("Elapsed: {:.2?}", elapsed);

    Ok(Arc::try_unwrap(cloned_manifest)
        .map_err(|_| Error::ArcError)?
        .into_inner())
}
