//! Downloader for Minecraft data

use crate::{
    state::State,
    util::{fetch::*, platform::OsExt},
};
use daedalus::{
    self as d,
    minecraft::{
        Asset, AssetsIndex, Library, Os, Version as GameVersion,
        VersionInfo as GameVersionInfo,
    },
    modded::LoaderVersion,
};
use futures::prelude::*;
use tokio::{fs, sync::OnceCell};

#[tracing::instrument(skip_all)]
pub async fn download_minecraft(
    st: &State,
    version: &GameVersionInfo,
) -> crate::Result<()> {
    log::info!("Downloading Minecraft version {}", version.id);
    let assets_index = download_assets_index(st, version).await?;

    tokio::try_join! {
        download_client(st, version),
        download_assets(st, version.assets == "legacy", &assets_index),
        download_libraries(st, version.libraries.as_slice(), &version.id)
    }?;

    log::info!("Done downloading Minecraft!");
    Ok(())
}

#[tracing::instrument(skip_all, fields(version = version.id.as_str(), loader = ?loader))]
pub async fn download_version_info(
    st: &State,
    version: &GameVersion,
    loader: Option<&LoaderVersion>,
) -> crate::Result<GameVersionInfo> {
    let version_id = loader.map_or(&version.id, |it| &it.id);
    log::debug!("Loading version info for Minecraft {version_id}");
    let path = st
        .directories
        .version_dir(version_id)
        .join(format!("{version_id}.json"));

    let res = if path.exists() {
        fs::read(path)
            .err_into::<crate::Error>()
            .await
            .and_then(|ref it| Ok(serde_json::from_slice(it)?))
    } else {
        log::info!("Downloading version info for version {}", &version.id);
        let mut info = d::minecraft::fetch_version_info(version).await?;

        if let Some(loader) = loader {
            let partial = d::modded::fetch_partial_version(&loader.url).await?;
            info = d::modded::merge_partial_version(partial, info);
            info.id = loader.id.clone();
        }

        let permit = st.io_semaphore.acquire().await.unwrap();
        write(&path, &serde_json::to_vec(&info)?, &permit).await?;
        Ok(info)
    }?;

    log::debug!("Loaded version info for Minecraft {version_id}");
    Ok(res)
}

#[tracing::instrument(skip_all)]
pub async fn download_client(
    st: &State,
    version_info: &GameVersionInfo,
) -> crate::Result<()> {
    let ref version = version_info.id;
    log::debug!("Locating client for version {version}");
    let client_download = version_info
        .downloads
        .get(&d::minecraft::DownloadType::Client)
        .ok_or(
            crate::ErrorKind::LauncherError(format!(
                "No client downloads exist for version {version}"
            ))
            .as_error(),
        )?;
    let path = st
        .directories
        .version_dir(version)
        .join(format!("{version}.jar"));

    if !path.exists() {
        let permit = st.io_semaphore.acquire().await.unwrap();
        let bytes =
            fetch(&client_download.url, Some(&client_download.sha1), &permit)
                .await?;
        write(&path, &bytes, &permit).await?;
        log::info!("Fetched client version {version}");
    }

    log::debug!("Client loaded for version {version}!");
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn download_assets_index(
    st: &State,
    version: &GameVersionInfo,
) -> crate::Result<AssetsIndex> {
    log::debug!("Loading assets index");
    let path = st
        .directories
        .assets_index_dir()
        .join(format!("{}.json", &version.asset_index.id));

    let res = if path.exists() {
        fs::read(path)
            .err_into::<crate::Error>()
            .await
            .and_then(|ref it| Ok(serde_json::from_slice(it)?))
    } else {
        let index = d::minecraft::fetch_assets_index(version).await?;
        let permit = st.io_semaphore.acquire().await.unwrap();
        write(&path, &serde_json::to_vec(&index)?, &permit).await?;
        log::info!("Fetched assets index");
        Ok(index)
    }?;

    log::debug!("Assets index successfully loaded!");
    Ok(res)
}

#[tracing::instrument(skip(st, index))]
pub async fn download_assets(
    st: &State,
    with_legacy: bool,
    index: &AssetsIndex,
) -> crate::Result<()> {
    log::debug!("Loading assets");
    stream::iter(index.objects.iter())
        .map(Ok::<(&String, &Asset), crate::Error>)
        .try_for_each_concurrent(Some(st.settings.read().await.max_concurrent_downloads), |(name, asset)| async move {
            let ref hash = asset.hash;
            let resource_path = st.directories.object_dir(hash);
            let url = format!(
                "https://resources.download.minecraft.net/{sub_hash}/{hash}",
                sub_hash = &hash[..2]
            );

            let fetch_cell = OnceCell::<bytes::Bytes>::new();
            tokio::try_join! {
                async {
                    if !resource_path.exists() {
                        let permit = st.io_semaphore.acquire().await.unwrap();
                        let resource = fetch_cell
                            .get_or_try_init(|| fetch(&url, Some(hash), &permit))
                            .await?;
                        write(&resource_path, &resource, &permit).await?;
                        log::info!("Fetched asset with hash {hash}");
                    }
                    Ok::<_, crate::Error>(())
                },
                async {
                    if with_legacy {
                        let permit = st.io_semaphore.acquire().await.unwrap();
                        let resource = fetch_cell
                            .get_or_try_init(|| fetch(&url, Some(hash), &permit))
                            .await?;
                        let resource_path = st.directories.legacy_assets_dir().join(
                            name.replace('/', &String::from(std::path::MAIN_SEPARATOR))
                        );
                        write(&resource_path, &resource, &permit).await?;
                        log::info!("Fetched legacy asset with hash {hash}");
                    }
                    Ok::<_, crate::Error>(())
                },
            }?;

            log::debug!("Loaded asset with hash {hash}");
            Ok(())
        }).await?;

    log::debug!("Done loading assets!");
    Ok(())
}

#[tracing::instrument(skip(st, libraries))]
pub async fn download_libraries(
    st: &State,
    libraries: &[Library],
    version: &str,
) -> crate::Result<()> {
    log::debug!("Loading libraries");

    tokio::try_join! {
        fs::create_dir_all(st.directories.libraries_dir()),
        fs::create_dir_all(st.directories.version_natives_dir(version))
    }?;

    stream::iter(libraries.iter())
        .map(Ok::<&Library, crate::Error>)
        .try_for_each_concurrent(Some(st.settings.read().await.max_concurrent_downloads), |library| async move {
            if let Some(rules) = &library.rules {
                if !rules.iter().all(super::parse_rule) {
                    return Ok(());
                }
            }
            tokio::try_join! {
                async {
                    let artifact_path = d::get_path_from_artifact(&library.name)?;
                    let path = st.directories.libraries_dir().join(&artifact_path);

                    match library.downloads {
                        _ if path.exists() => Ok(()),
                        Some(d::minecraft::LibraryDownloads {
                            artifact: Some(ref artifact),
                            ..
                        }) => {
                            let permit = st.io_semaphore.acquire().await.unwrap();
                            let bytes = fetch(&artifact.url, Some(&artifact.sha1), &permit)
                                .await?;
                            write(&path, &bytes, &permit).await?;
                            log::info!("Fetched library {}", &library.name);
                            Ok::<_, crate::Error>(())
                        }
                        None => {
                            let url = [
                                library
                                    .url
                                    .as_deref()
                                    .unwrap_or("https://libraries.minecraft.net"),
                                &artifact_path
                            ].concat();

                            let permit = st.io_semaphore.acquire().await.unwrap();
                            let bytes = fetch(&url, None, &permit).await?;
                            write(&path, &bytes, &permit).await?;
                            log::info!("Fetched library {}", &library.name);
                            Ok::<_, crate::Error>(())
                        }
                        _ => Ok(())
                    }
                },
                async {
                    // HACK: pseudo try block using or else
                    if let Some((os_key, classifiers)) = None.or_else(|| Some((
                        library
                            .natives
                            .as_ref()?
                            .get(&Os::native())?,
                        library
                            .downloads
                            .as_ref()?
                            .classifiers
                            .as_ref()?
                    ))) {
                        let parsed_key = os_key.replace(
                            "${arch}",
                            crate::util::platform::ARCH_WIDTH,
                        );

                        if let Some(native) = classifiers.get(&parsed_key) {
                            let permit = st.io_semaphore.acquire().await.unwrap();
                            let data = fetch(&native.url, Some(&native.sha1), &permit).await?;
                            let reader = std::io::Cursor::new(&data);
                            let mut archive = zip::ZipArchive::new(reader).unwrap();
                            archive.extract(&st.directories.version_natives_dir(version)).unwrap();
                            log::info!("Fetched native {}", &library.name);
                        }
                    }

                    Ok(())
                }
            }?;

            log::debug!("Loaded library {}", library.name);
            Ok(())
        }
    ).await?;

    log::debug!("Done loading libraries!");
    Ok(())
}
