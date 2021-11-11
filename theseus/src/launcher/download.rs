use crate::launcher::LauncherError;
use daedalus::get_path_from_artifact;
use daedalus::minecraft::{
    fetch_assets_index, fetch_version_info, Asset, AssetsIndex, DownloadType, Library, Os, Version,
    VersionInfo,
};
use daedalus::modded::{fetch_partial_version, merge_partial_version, LoaderVersion};
use futures::future;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub async fn download_version_info(
    client_path: &Path,
    version: &Version,
    loader_version: Option<&LoaderVersion>,
) -> Result<VersionInfo, LauncherError> {
    let id = loader_version.map(|x| &x.id).unwrap_or(&version.id);

    let path = &*client_path.join(id).join(format!("{}.json", id));

    if path.exists() {
        Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
    } else {
        let mut info = fetch_version_info(version).await?;

        if let Some(loader_version) = loader_version {
            let partial = fetch_partial_version(&*loader_version.url).await?;

            info = merge_partial_version(partial, info);

            info.id = loader_version.id.clone();
        }

        save_file(path, &bytes::Bytes::from(serde_json::to_string(&info)?))?;

        Ok(info)
    }
}

pub async fn download_client(
    client_path: &Path,
    version_info: &VersionInfo,
) -> Result<(), LauncherError> {
    let client_download = version_info
        .downloads
        .get(&DownloadType::Client)
        .ok_or_else(|| {
            LauncherError::InvalidInput(format!(
                "Version {} does not have any client downloads",
                &version_info.id
            ))
        })?;

    let path = &*client_path
        .join(&version_info.id)
        .join(format!("{}.jar", &version_info.id));

    save_and_download_file(path, &client_download.url, Some(&client_download.sha1)).await?;

    Ok(())
}

pub async fn download_assets_index(
    assets_path: &Path,
    version: &VersionInfo,
) -> Result<AssetsIndex, LauncherError> {
    let path = &*assets_path
        .join("indexes")
        .join(format!("{}.json", &version.asset_index.id));

    if path.exists() {
        Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
    } else {
        let index = fetch_assets_index(version).await?;

        save_file(path, &bytes::Bytes::from(serde_json::to_string(&index)?))?;

        Ok(index)
    }
}

pub async fn download_assets(
    assets_path: &Path,
    legacy_path: Option<&Path>,
    index: &AssetsIndex,
) -> Result<(), LauncherError> {
    future::join_all(
        index
            .objects
            .iter()
            .map(|x| download_asset(assets_path, legacy_path, x.0, x.1)),
    )
    .await
    .into_iter()
    .collect::<Result<Vec<()>, LauncherError>>()?;

    Ok(())
}

async fn download_asset(
    assets_path: &Path,
    legacy_path: Option<&Path>,
    name: &str,
    asset: &Asset,
) -> Result<(), LauncherError> {
    let sub_hash = &&asset.hash[..2];

    let resource_path = assets_path.join("objects").join(sub_hash).join(&asset.hash);

    let resource = save_and_download_file(
        &*resource_path,
        &format!(
            "https://resources.download.minecraft.net/{}/{}",
            sub_hash, asset.hash
        ),
        Some(&*asset.hash),
    )
    .await?;

    if let Some(legacy_path) = legacy_path {
        let resource_path =
            legacy_path.join(name.replace('/', &*std::path::MAIN_SEPARATOR.to_string()));
        save_file(resource_path.as_path(), &resource)?;
    }

    Ok(())
}

pub async fn download_libraries(
    libraries_path: &Path,
    natives_path: &Path,
    libraries: &[Library],
) -> Result<(), LauncherError> {
    future::join_all(
        libraries
            .iter()
            .map(|x| download_library(libraries_path, natives_path, x)),
    )
    .await
    .into_iter()
    .collect::<Result<Vec<()>, LauncherError>>()?;

    Ok(())
}

async fn download_library(
    libraries_path: &Path,
    natives_path: &Path,
    library: &Library,
) -> Result<(), LauncherError> {
    if let Some(rules) = &library.rules {
        if !super::rules::parse_rules(rules.as_slice()) {
            return Ok(());
        }
    }

    let (a, b) = future::join(
        download_library_jar(libraries_path, library),
        download_native(natives_path, library),
    )
    .await;

    a?;
    b?;

    Ok(())
}

async fn download_library_jar(
    libraries_path: &Path,
    library: &Library,
) -> Result<(), LauncherError> {
    let mut path = libraries_path.to_path_buf();
    path.push(get_path_from_artifact(&*library.name)?);

    if let Some(downloads) = &library.downloads {
        if let Some(library) = &downloads.artifact {
            save_and_download_file(&*path, &library.url, Some(&library.sha1)).await?;
        }
    } else {
        save_and_download_file(
            &*path,
            &format!(
                "{}{}",
                library
                    .url
                    .as_deref()
                    .unwrap_or("https://libraries.minecraft.net/"),
                get_path_from_artifact(&*library.name)?
            ),
            None,
        )
        .await?;
    }

    Ok(())
}

async fn download_native(natives_path: &Path, library: &Library) -> Result<(), LauncherError> {
    if let Some(natives) = &library.natives {
        if let Some(os_key) = natives.get(&get_os()) {
            if let Some(downloads) = &library.downloads {
                if let Some(classifiers) = &downloads.classifiers {
                    #[cfg(target_pointer_width = "64")]
                    let parsed_key = os_key.replace("${arch}", "64");
                    #[cfg(target_pointer_width = "32")]
                    let parsed_key = os_key.replace("${arch}", "32");

                    if let Some(native) = classifiers.get(&*parsed_key) {
                        let file = download_file(&native.url, Some(&native.sha1)).await?;

                        let reader = std::io::Cursor::new(&*file);

                        let mut archive = zip::ZipArchive::new(reader).unwrap();
                        archive.extract(natives_path).unwrap();
                    }
                }
            }
        }
    }

    Ok(())
}

async fn save_and_download_file(
    path: &Path,
    url: &str,
    sha1: Option<&str>,
) -> Result<bytes::Bytes, LauncherError> {
    let read = std::fs::read(path).ok().map(bytes::Bytes::from);

    if let Some(bytes) = read {
        Ok(bytes)
    } else {
        let file = download_file(url, sha1).await?;

        save_file(path, &file)?;

        Ok(file)
    }
}

fn save_file(path: &Path, bytes: &bytes::Bytes) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = File::create(path)?;
    file.write_all(bytes)?;

    Ok(())
}

pub fn get_os() -> Os {
    match std::env::consts::OS {
        "windows" => Os::Windows,
        "macos" => Os::Osx,
        "linux" => Os::Linux,
        _ => Os::Unknown,
    }
}

pub async fn download_file(url: &str, sha1: Option<&str>) -> Result<bytes::Bytes, LauncherError> {
    let client = reqwest::Client::builder()
        .tcp_keepalive(Some(std::time::Duration::from_secs(10)))
        .build()
        .map_err(|err| LauncherError::FetchError {
            inner: err,
            item: url.to_string(),
        })?;

    for attempt in 1..=4 {
        let result = client.get(url).send().await;

        match result {
            Ok(x) => {
                let bytes = x.bytes().await;

                if let Ok(bytes) = bytes {
                    if let Some(sha1) = sha1 {
                        if &*get_hash(bytes.clone()).await? != sha1 {
                            if attempt <= 3 {
                                continue;
                            } else {
                                return Err(LauncherError::ChecksumFailure {
                                    hash: sha1.to_string(),
                                    url: url.to_string(),
                                    tries: attempt,
                                });
                            }
                        }
                    }

                    return Ok(bytes);
                } else if attempt <= 3 {
                    continue;
                } else if let Err(err) = bytes {
                    return Err(LauncherError::FetchError {
                        inner: err,
                        item: url.to_string(),
                    });
                }
            }
            Err(_) if attempt <= 3 => continue,
            Err(err) => {
                return Err(LauncherError::FetchError {
                    inner: err,
                    item: url.to_string(),
                })
            }
        }
    }

    unreachable!()
}

/// Computes a checksum of the input bytes
pub async fn get_hash(bytes: bytes::Bytes) -> Result<String, LauncherError> {
    let hash = tokio::task::spawn_blocking(|| sha1::Sha1::from(bytes).hexdigest()).await?;

    Ok(hash)
}
