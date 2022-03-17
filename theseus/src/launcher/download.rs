use crate::launcher::LauncherError;
use daedalus::get_path_from_artifact;
use daedalus::minecraft::{
    fetch_assets_index, fetch_version_info, Asset, AssetsIndex, DownloadType,
    Library, Os, Version, VersionInfo,
};
use daedalus::modded::{
    fetch_partial_version, merge_partial_version, LoaderVersion,
};
use futures::future;
use std::path::Path;
use std::time::Duration;
use tokio::{fs::File, io::AsyncWriteExt, sync::Semaphore};

// TODO: unhardcode
const MAX_OPEN: usize = 32;
static DOWNLOADS_SEMAPHORE: Semaphore = Semaphore::const_new(MAX_OPEN);
static WRITE_SEMAPHORE: Semaphore = Semaphore::const_new(MAX_OPEN);

pub async fn download_version_info(
    client_path: &Path,
    version: &Version,
    loader_version: Option<&LoaderVersion>,
) -> Result<VersionInfo, LauncherError> {
    let id = match loader_version {
        Some(x) => &x.id,
        None => &version.id,
    };

    let mut path = client_path.join(id);
    path.push(&format!("{id}.json"));

    if path.exists() {
        let contents = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&contents)?)
    } else {
        let mut info = fetch_version_info(version).await?;

        if let Some(loader_version) = loader_version {
            let partial = fetch_partial_version(&loader_version.url).await?;
            info = merge_partial_version(partial, info);
            info.id = loader_version.id.clone();
        }
        let info_s = serde_json::to_string(&info)?;
        save_file(&path, &bytes::Bytes::from(info_s)).await?;

        Ok(info)
    }
}

pub async fn download_client(
    client_path: &Path,
    version_info: &VersionInfo,
) -> Result<(), LauncherError> {
    let version = &version_info.id;
    let client_download = version_info
        .downloads
        .get(&DownloadType::Client)
        .ok_or_else(|| {
            LauncherError::InvalidInput(format!(
                "Version {version} does not have any client downloads"
            ))
        })?;

    let mut path = client_path.join(version);
    path.push(&format!("{version}.jar"));

    save_and_download_file(
        &path,
        &client_download.url,
        Some(&client_download.sha1),
    )
    .await?;
    Ok(())
}

pub async fn download_assets_index(
    assets_path: &Path,
    version: &VersionInfo,
) -> Result<AssetsIndex, LauncherError> {
    let path =
        assets_path.join(format!("indexes/{}.json", &version.asset_index.id));

    if path.exists() {
        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        let index = fetch_assets_index(version).await?;

        save_file(&path, &bytes::Bytes::from(serde_json::to_string(&index)?))
            .await?;

        Ok(index)
    }
}

pub async fn download_assets(
    assets_path: &Path,
    legacy_path: Option<&Path>,
    index: &AssetsIndex,
) -> Result<(), LauncherError> {
    future::join_all(index.objects.iter().map(|(name, asset)| {
        download_asset(assets_path, legacy_path, name, asset)
    }))
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
    let hash = &asset.hash;
    let sub_hash = &hash[..2];

    let mut resource_path = assets_path.join("objects");
    resource_path.push(sub_hash);
    resource_path.push(hash);

    let url =
        format!("https://resources.download.minecraft.net/{sub_hash}/{hash}");

    let resource =
        save_and_download_file(&resource_path, &url, Some(hash)).await?;

    if let Some(legacy_path) = legacy_path {
        let resource_path = legacy_path
            .join(name.replace('/', &std::path::MAIN_SEPARATOR.to_string()));
        save_file(resource_path.as_path(), &resource).await?;
    }

    Ok(())
}

pub async fn download_libraries(
    libraries_path: &Path,
    natives_path: &Path,
    libraries: &[Library],
) -> Result<(), LauncherError> {
    future::join_all(libraries.iter().map(|library| {
        download_library(libraries_path, natives_path, library)
    }))
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
        if !super::rules::parse_rules(rules) {
            return Ok(());
        }
    }

    future::try_join(
        download_library_jar(libraries_path, library),
        download_native(natives_path, library),
    )
    .await?;

    Ok(())
}

async fn download_library_jar(
    libraries_path: &Path,
    library: &Library,
) -> Result<(), LauncherError> {
    let artifact_path = get_path_from_artifact(&library.name)?;
    let path = libraries_path.join(&artifact_path);

    if let Some(downloads) = &library.downloads {
        if let Some(library) = &downloads.artifact {
            save_and_download_file(&path, &library.url, Some(&library.sha1))
                .await?;
        }
    } else {
        let url = format!(
            "{}{artifact_path}",
            library
                .url
                .as_deref()
                .unwrap_or("https://libraries.minecraft.net/"),
        );
        save_and_download_file(&path, &url, None).await?;
    }

    Ok(())
}

async fn download_native(
    natives_path: &Path,
    library: &Library,
) -> Result<(), LauncherError> {
    use daedalus::minecraft::LibraryDownload;
    use std::collections::HashMap;

    // Try blocks in stable Rust when?
    let optional_cascade =
        || -> Option<(&String, &HashMap<String, LibraryDownload>)> {
            let os_key = library.natives.as_ref()?.get(&get_os())?;
            let classifiers =
                library.downloads.as_ref()?.classifiers.as_ref()?;
            Some((os_key, classifiers))
        };

    if let Some((os_key, classifiers)) = optional_cascade() {
        #[cfg(target_pointer_width = "64")]
        let parsed_key = os_key.replace("${arch}", "64");
        #[cfg(target_pointer_width = "32")]
        let parsed_key = os_key.replace("${arch}", "32");

        if let Some(native) = classifiers.get(&parsed_key) {
            let file = download_file(&native.url, Some(&native.sha1)).await?;

            let reader = std::io::Cursor::new(&file);

            let mut archive = zip::ZipArchive::new(reader).unwrap();
            archive.extract(natives_path).unwrap();
        }
    }
    Ok(())
}

async fn save_and_download_file(
    path: &Path,
    url: &str,
    sha1: Option<&str>,
) -> Result<bytes::Bytes, LauncherError> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(bytes::Bytes::from(bytes)),
        Err(_) => {
            let file = download_file(url, sha1).await?;
            save_file(path, &file).await?;
            Ok(file)
        }
    }
}

async fn save_file(path: &Path, bytes: &bytes::Bytes) -> std::io::Result<()> {
    let _save_permit = WRITE_SEMAPHORE.acquire().await.unwrap();
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let mut file = File::create(path).await?;
    file.write_all(bytes).await?;
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

pub async fn download_file(
    url: &str,
    sha1: Option<&str>,
) -> Result<bytes::Bytes, LauncherError> {
    let _download_permit = DOWNLOADS_SEMAPHORE.acquire().await.unwrap();
    let client = reqwest::Client::builder()
        .tcp_keepalive(Some(Duration::from_secs(10)))
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
                        if &get_hash(bytes.clone()).await? != sha1 {
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
async fn get_hash(bytes: bytes::Bytes) -> Result<String, LauncherError> {
    let hash =
        tokio::task::spawn_blocking(|| sha1::Sha1::from(bytes).hexdigest())
            .await?;

    Ok(hash)
}
