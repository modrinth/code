use crate::{format_url, upload_file_to_bucket, Error};
use daedalus::fabric::PartialVersionInfo;
use daedalus::minecraft::Library;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

pub async fn retrieve_data() -> Result<(), Error> {
    let mut list = daedalus::fabric::fetch_fabric_versions(None).await?;

    let loaders = RwLock::new(Vec::new());
    let visited_artifacts_mutex = Arc::new(Mutex::new(Vec::new()));

    if let Some(latest) = list.loader.get(0) {
        {
            let mut loaders = match loaders.write() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };

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

        let mut versions = list
            .game
            .iter_mut()
            .map(|game_version| {
                let loaders = match loaders.read() {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
                };

                let visited_artifacts_mutex = Arc::clone(&visited_artifacts_mutex);
                let game_version_mutex = Mutex::new(HashMap::new());

                async move {
                    let versions = futures::future::try_join_all(loaders.clone().into_iter().map(
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
                                        let mut visited_assets =
                                            match visited_artifacts_mutex.lock() {
                                                Ok(guard) => guard,
                                                Err(poisoned) => poisoned.into_inner(),
                                            };

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
                                let mut game_version_map = match game_version_mutex.lock() {
                                    Ok(guard) => guard,
                                    Err(poisoned) => poisoned.into_inner(),
                                };
                                game_version_map.insert(loader, format_url(&*version_path));
                            }

                            Ok::<(), Error>(())
                        },
                    ))
                    .await?;

                    game_version.urls = Some(
                        match game_version_mutex.lock() {
                            Ok(guard) => guard,
                            Err(poisoned) => poisoned.into_inner(),
                        }
                        .clone(),
                    );

                    Ok::<(), Error>(())
                }
            })
            .peekable();

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
