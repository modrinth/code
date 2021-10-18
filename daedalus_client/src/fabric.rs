use crate::{format_url, upload_file_to_bucket, Error};
use daedalus::fabric::PartialVersionInfo;
use daedalus::minecraft::Library;
use std::collections::HashMap;
use std::sync::{Arc};
use std::time::{Duration, Instant};
use futures::lock::Mutex;

pub async fn retrieve_data() -> Result<(), Error> {
    let mut list = daedalus::fabric::fetch_fabric_versions(None).await?;

    if let Some(latest) = list.loader.get(0) {
        let loaders_mutex = Arc::new(Mutex::new(Vec::new()));
        let visited_artifacts_mutex = Arc::new(Mutex::new(Vec::new()));

        {
            let mut loaders = loaders_mutex.lock().await;

            loaders.push(latest.version.clone());

            if !latest.stable {
                if let Some(stable) = list.loader.iter().find(|x| x.stable) {
                    loaders.push(stable.version.clone());
                }
            }

            list.loader = list
                .loader
                .into_iter()
                .filter(|x| loaders.contains(&x.version))
                .collect();
        }

        let mut version_futures = Vec::new();

        for game_version in list.game.iter_mut() {
            let visited_artifacts_mutex = Arc::clone(&visited_artifacts_mutex);
            let game_version_mutex = Mutex::new(HashMap::new());
            let loaders_mutex = Arc::clone(&loaders_mutex);
            version_futures.push(async move {
                let versions = futures::future::try_join_all(loaders_mutex.lock().await.clone().into_iter().map(
                    |loader| async {
                        let version = daedalus::fabric::fetch_fabric_version(
                            &*game_version.version,
                            &*loader,
                        )
                        .await
                        .expect(&*format!("{}, {}", game_version.version, loader));

                        Ok::<(String, PartialVersionInfo), Error>((loader, version))
                    },
                ))
                .await?;

                futures::future::try_join_all(versions.into_iter().map(
                    |(loader, version)| async {
                        let libs = futures::future::try_join_all(
                            version.libraries.into_iter().map(|mut lib| async {
                                {
                                    let mut visited_assets = visited_artifacts_mutex.lock().await;

                                    if visited_assets.contains(&lib.name) {
                                        lib.url = Some(format_url("maven/"));

                                        return Ok(lib);
                                    } else {
                                        visited_assets.push(lib.name.clone())
                                    }
                                }

                                let artifact_path =
                                    daedalus::get_path_from_artifact(&*lib.name)?;

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
                                )
                                .await?;

                                Ok::<Library, Error>(lib)
                            }),
                        )
                        .await?;

                        let version_path = format!(
                            "fabric/v{}/versions/{}-{}.json",
                            daedalus::fabric::CURRENT_FORMAT_VERSION,
                            version.inherits_from,
                            loader
                        );

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
                            })?,
                            Some("application/json".to_string()),
                        )
                        .await?;

                        {
                            let mut game_version_map = game_version_mutex.lock().await;
                            game_version_map.insert(loader, format_url(&*version_path));
                        }

                        Ok::<(), Error>(())
                    },
                ))
                .await?;

                game_version.urls = Some(
                    game_version_mutex.lock().await
                    .clone(),
                );

                Ok::<(), Error>(())
            });
        }

        let mut versions = version_futures.into_iter().peekable();
        let mut chunk_index = 0;
        while versions.peek().is_some() {
            let now = Instant::now();

            let chunk: Vec<_> = versions.by_ref().take(10).collect();
            futures::future::try_join_all(chunk).await?;

            std::thread::sleep(Duration::from_secs(1));

            chunk_index += 1;

            let elapsed = now.elapsed();
            println!("Chunk {} Elapsed: {:.2?}", chunk_index, elapsed);
        }
    }

    upload_file_to_bucket(
        format!(
            "fabric/v{}/manifest.json",
            daedalus::fabric::CURRENT_FORMAT_VERSION,
        ),
        serde_json::to_vec(&list)?,
        Some("application/json".to_string()),
    )
    .await?;

    Ok(())
}
