//! Functions for fetching infromation from the Internet
use crate::config::REQWEST_CLIENT;
use std::path::Path;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
    sync::SemaphorePermit,
};

const FETCH_ATTEMPTS: usize = 3;

/// Downloads a file with retry and checksum functionality
#[tracing::instrument(skip(_permit))]
pub async fn fetch<'a>(
    url: &str,
    sha1: Option<&str>,
    _permit: &SemaphorePermit<'a>,
) -> crate::Result<bytes::Bytes> {
    for attempt in 1..=(FETCH_ATTEMPTS + 1) {
        let result = REQWEST_CLIENT.get(url).send().await;

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
            Err(err) => return Err(err.into()),
        }
    }

    unreachable!()
}

/// Downloads a file from specified mirrors
#[tracing::instrument(skip(permit))]
pub async fn fetch_mirrors<'a>(
    mirrors: &[&str],
    sha1: Option<&str>,
    permit: &SemaphorePermit<'a>,
) -> crate::Result<bytes::Bytes> {
    if mirrors.is_empty() {
        return Err(crate::ErrorKind::InputError(
            "No mirrors provided!".to_string(),
        )
        .into());
    }

    for (index, mirror) in mirrors.iter().enumerate() {
        let result = fetch(mirror, sha1, permit).await;

        if result.is_ok() || (result.is_err() && index == (mirrors.len() - 1)) {
            return result;
        }
    }

    unreachable!()
}

#[tracing::instrument(skip(bytes, _permit))]
pub async fn write<'a>(
    path: &Path,
    bytes: &[u8],
    _permit: &SemaphorePermit<'a>,
) -> crate::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }

    let mut file = File::create(path).await?;
    log::debug!("Done writing file {}", path.display());
    file.write_all(bytes).await?;
    Ok(())
}

async fn sha1_async(bytes: bytes::Bytes) -> crate::Result<String> {
    let hash = tokio::task::spawn_blocking(move || {
        sha1::Sha1::from(bytes).hexdigest()
    })
    .await?;

    Ok(hash)
}
