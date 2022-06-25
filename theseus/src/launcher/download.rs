//! Downloader for Minecraft data

use crate::{
    state::DirectoryInfo as Dirs,
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
use tokio::fs;

pub async fn download_minecraft(
    version: &GameVersionInfo,
    dirs: &Dirs,
) -> crate::Result<()> {
    let assets_index = download_assets_index(dirs, version).await?;

    tokio::try_join! {
        download_client(dirs, version),
        download_assets(dirs, version.assets == "legacy", &assets_index),
        download_libraries(dirs, version.libraries.as_slice())
    }?;

    Ok(())
}

pub async fn download_version_info(
    dirs: &Dirs,
    version: &GameVersion,
    loader: Option<&LoaderVersion>,
) -> crate::Result<GameVersionInfo> {
    let version_id = loader.map_or(&version.id, |it| &it.id);
    let path = dirs
        .version_dir(version_id)
        .join(format!("{version_id}.json"));

    if path.exists() {
        fs::read(path)
            .err_into::<crate::Error>()
            .await
            .and_then(|ref it| Ok(serde_json::from_slice(it)?))
    } else {
        let mut info = d::minecraft::fetch_version_info(version).await?;

        if let Some(loader) = loader {
            let partial = d::modded::fetch_partial_version(&loader.url).await?;
            info = d::modded::merge_partial_version(partial, info);
            info.id = loader.id.clone();
        }

        write(&path, &serde_json::to_vec(&info)?).await?;
        Ok(info)
    }
}

pub async fn download_client(
    dirs: &Dirs,
    version_info: &GameVersionInfo,
) -> crate::Result<()> {
    let ref version = version_info.id;
    let client_download = version_info
        .downloads
        .get(&d::minecraft::DownloadType::Client)
        .ok_or(crate::Error::LauncherError(format!(
            "No client downloads exist for version {version}"
        )))?;
    let path = dirs.version_dir(version).join(format!("{version}.jar"));

    fetch(&client_download.url, Some(&client_download.sha1))
        .and_then(|it| async move {
            write(&path, &it).await?;
            Ok(())
        })
        .await
}

pub async fn download_assets_index(
    dirs: &Dirs,
    version: &GameVersionInfo,
) -> crate::Result<AssetsIndex> {
    let path = dirs
        .assets_index_dir()
        .join(format!("{}.json", &version.asset_index.id));

    if path.exists() {
        fs::read(path)
            .err_into::<crate::Error>()
            .await
            .and_then(|ref it| Ok(serde_json::from_slice(it)?))
    } else {
        let index = d::minecraft::fetch_assets_index(version).await?;
        write(&path, &serde_json::to_vec(&index)?).await?;
        Ok(index)
    }
}

pub async fn download_assets(
    dirs: &Dirs,
    with_legacy: bool,
    index: &AssetsIndex,
) -> crate::Result<()> {
    stream::iter(index.objects.iter())
        .map(Ok::<(&String, &Asset), crate::Error>)
        .try_for_each_concurrent(None, |(name, asset)| async move {
            let ref hash = asset.hash;
            let resource_path = dirs.object_dir(hash);
            let url = format!(
                "https://resources.download.minecraft.net/{sub_hash}/hash",
                sub_hash = &hash[..2]
            );

            let resource = fetch(&url, Some(hash)).await?;
            tokio::try_join! {
                write(&resource_path, &resource),
                async {
                    if with_legacy {
                        let resource_path = dirs.legacy_assets_dir().join(
                            name.replace('/', &String::from(std::path::MAIN_SEPARATOR))
                        );
                        write(&resource_path, &resource).await?;
                    }
                    Ok(())
                }
            }?;

            Ok(())
        }).await?;
    Ok(())
}

pub async fn download_libraries(
    dirs: &Dirs,
    libraries: &[Library],
) -> crate::Result<()> {
    stream::iter(libraries.iter())
        .map(Ok::<&Library, crate::Error>)
        .try_for_each_concurrent(None, |library| async move {
            if let Some(rules) = &library.rules {
                if !rules.iter().all(super::parse_rule) {
                    return Ok(());
                }
            }

            tokio::try_join! {
                async {
                    let artifact_path = d::get_path_from_artifact(&library.name)?;
                    let path = dirs.libraries_dir().join(&artifact_path);

                    match library.downloads {
                        Some(d::minecraft::LibraryDownloads {
                            artifact: Some(ref artifact),
                            ..
                        }) => {
                            fetch(&artifact.url, Some(&artifact.sha1))
                                .and_then(|it| async move {
                                    write(&path, &it).await?;
                                    Ok(())
                                })
                                .await
                        }
                        None => {
                            let url = [
                                library
                                    .url
                                    .as_deref()
                                    .unwrap_or("https://libraries.minecraft.net"),
                                &artifact_path
                            ].concat();

                            fetch(&url, None)
                                .and_then(|it| async move {
                                    write(&path, &it).await?;
                                    Ok(())
                                })
                                .await
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
                            let data = fetch(&native.url, Some(&native.sha1)).await?;
                            let reader = std::io::Cursor::new(&data);
                            let mut archive = zip::ZipArchive::new(reader).unwrap();

                            archive.extract(dirs.natives_dir()).unwrap();
                        }
                    }

                    Ok(())
                }
            }?;

            Ok(())
        }
    ).await?;

    Ok(())
}
