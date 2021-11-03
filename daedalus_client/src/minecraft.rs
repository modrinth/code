use crate::{format_url, upload_file_to_bucket, Error};
use daedalus::download_file;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::time::{Duration, Instant};
use log::info;

pub async fn retrieve_data(uploaded_files: &mut Vec<String>) -> Result<(), Error> {
    let old_manifest =
        daedalus::minecraft::fetch_version_manifest(Some(&*crate::format_url(&*format!(
            "minecraft/v{}/manifest.json",
            daedalus::minecraft::CURRENT_FORMAT_VERSION
        ))))
        .await
        .ok();

    let mut manifest = daedalus::minecraft::fetch_version_manifest(None).await?;
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

            let assets_hash = old_version.map(|x| x.assets_index_sha1.clone()).flatten();

            async move {
                let mut upload_futures = Vec::new();

                let mut version_info = daedalus::minecraft::fetch_version_info(version).await?;

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
                    let mut cloned_manifest = cloned_manifest_mutex.lock().await;

                    let position = cloned_manifest
                        .versions
                        .iter()
                        .position(|x| version.id == x.id)
                        .unwrap();
                    cloned_manifest.versions[position].url = format_url(&version_path);
                    cloned_manifest.versions[position].assets_index_sha1 =
                        Some(version_info.asset_index.sha1.clone());
                    cloned_manifest.versions[position].assets_index_url =
                        Some(format_url(&assets_path));
                    version_info.asset_index.url = format_url(&assets_path);
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
                        visited_assets.push(version_info.asset_index.id.clone());
                    }
                }

                if download_assets {
                    let assets_index =
                        download_file(&assets_index_url, Some(&version_info.asset_index.sha1))
                            .await?;

                    {
                        upload_futures.push(upload_file_to_bucket(
                            assets_path,
                            assets_index.to_vec(),
                            Some("application/json".to_string()),
                            uploaded_files_mutex.as_ref()
                        ));
                    }
                }

                {
                    upload_futures.push(upload_file_to_bucket(
                        version_path,
                        serde_json::to_vec(&version_info)?,
                        Some("application/json".to_string()),
                        uploaded_files_mutex.as_ref()
                    ));
                }

                futures::future::try_join_all(upload_futures).await?;

                Ok::<(), Error>(())
            }
            .await?;

            Ok::<(), Error>(())
        })
    }

    {
        let mut versions = version_futures.into_iter().peekable();
        let mut chunk_index = 0;
        while versions.peek().is_some() {
            let now = Instant::now();

            let chunk: Vec<_> = versions.by_ref().take(100).collect();
            futures::future::try_join_all(chunk).await?;

            tokio::time::sleep(Duration::from_secs(1)).await;

            chunk_index += 1;

            let elapsed = now.elapsed();
            info!("Chunk {} Elapsed: {:.2?}", chunk_index, elapsed);
        }
    }

    upload_file_to_bucket(
        format!(
            "minecraft/v{}/manifest.json",
            daedalus::minecraft::CURRENT_FORMAT_VERSION
        ),
        serde_json::to_vec(&*cloned_manifest.lock().await)?,
        Some("application/json".to_string()),
        uploaded_files_mutex.as_ref()
    )
    .await?;

    if let Ok(uploaded_files_mutex) = Arc::try_unwrap(uploaded_files_mutex) {
        uploaded_files.extend(uploaded_files_mutex.into_inner());
    }

    let elapsed = now.elapsed();
    info!("Elapsed: {:.2?}", elapsed);

    Ok(())
}
