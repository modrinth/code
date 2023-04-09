//! Functions for fetching infromation from the Internet
use crate::config::REQWEST_CLIENT;
use bytes::Bytes;
use reqwest::Method;
use serde::de::DeserializeOwned;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use tokio::sync::Semaphore;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

const FETCH_ATTEMPTS: usize = 3;

pub async fn fetch(
    url: &str,
    sha1: Option<&str>,
    semaphore: &Semaphore,
) -> crate::Result<Bytes> {
    fetch_advanced(Method::GET, url, sha1, semaphore).await
}

pub async fn fetch_json<T>(
    method: Method,
    url: &str,
    sha1: Option<&str>,
    semaphore: &Semaphore,
) -> crate::Result<T>
where
    T: DeserializeOwned,
{
    let result = fetch_advanced(method, url, sha1, semaphore).await?;
    let value = serde_json::from_slice(&result)?;
    Ok(value)
}

/// Downloads a file with retry and checksum functionality
#[tracing::instrument(skip(semaphore))]
pub async fn fetch_advanced(
    method: Method,
    url: &str,
    sha1: Option<&str>,
    semaphore: &Semaphore,
) -> crate::Result<Bytes> {
    let _permit = semaphore.acquire().await?;
    for attempt in 1..=(FETCH_ATTEMPTS + 1) {
        let result = REQWEST_CLIENT.request(method.clone(), url).send().await;

        match result {
            Ok(x) => {
                let bytes = x.bytes().await;

                if let Ok(bytes) = bytes {
                    if let Some(sha1) = sha1 {
                        let hash = sha1_async(bytes.clone()).await?;
                        if &*hash != sha1 {
                            if attempt <= 3 {
                                continue;
                            } else {
                                return Err(crate::ErrorKind::HashError(
                                    sha1.to_string(),
                                    hash,
                                )
                                .into());
                            }
                        }
                    }

                    log::debug!("Done downloading URL {url}");
                    return Ok(bytes);
                } else if attempt <= 3 {
                    continue;
                } else if let Err(err) = bytes {
                    return Err(err.into());
                }
            }
            Err(_) if attempt <= 3 => continue,
            Err(err) => {
                return Err(err.into());
            }
        }
    }

    unreachable!()
}

/// Downloads a file from specified mirrors
#[tracing::instrument(skip(semaphore))]
pub async fn fetch_mirrors(
    mirrors: &[&str],
    sha1: Option<&str>,
    semaphore: &Semaphore,
) -> crate::Result<Bytes> {
    if mirrors.is_empty() {
        return Err(crate::ErrorKind::InputError(
            "No mirrors provided!".to_string(),
        )
        .into());
    }

    for (index, mirror) in mirrors.iter().enumerate() {
        let result = fetch(mirror, sha1, semaphore).await;

        if result.is_ok() || (result.is_err() && index == (mirrors.len() - 1)) {
            return result;
        }
    }

    unreachable!()
}

#[tracing::instrument(skip(bytes, semaphore))]
pub async fn write<'a>(
    path: &Path,
    bytes: &[u8],
    semaphore: &Semaphore,
) -> crate::Result<()> {
    let _permit = semaphore.acquire().await?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }

    let mut file = File::create(path).await?;
    file.write_all(bytes).await?;
    log::debug!("Done writing file {}", path.display());
    Ok(())
}

#[tracing::instrument(skip(bytes, semaphore))]
pub async fn write_cached_icon(
    icon_path: &str,
    cache_dir: &Path,
    bytes: Bytes,
    semaphore: &Semaphore,
) -> crate::Result<PathBuf> {
    let extension = Path::new(&icon_path).extension().and_then(OsStr::to_str);
    let hash = sha1_async(bytes.clone()).await?;
    let path = cache_dir.join("icons").join(if let Some(ext) = extension {
        format!("{hash}.{ext}")
    } else {
        hash
    });

    write(&path, &bytes, semaphore).await?;

    let path = dunce::canonicalize(path)?;
    Ok(path)
}

async fn sha1_async(bytes: Bytes) -> crate::Result<String> {
    let hash = tokio::task::spawn_blocking(move || {
        sha1::Sha1::from(bytes).hexdigest()
    })
    .await?;

    Ok(hash)
}
