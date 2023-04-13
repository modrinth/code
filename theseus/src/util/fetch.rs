//! Functions for fetching infromation from the Internet
use bytes::Bytes;
use lazy_static::lazy_static;
use reqwest::Method;
use serde::de::DeserializeOwned;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time;
use tokio::sync::{RwLock, Semaphore};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

lazy_static! {
    static ref REQWEST_CLIENT: reqwest::Client = {
        let mut headers = reqwest::header::HeaderMap::new();
        let header = reqwest::header::HeaderValue::from_str(&format!(
            "modrinth/theseus/{} (support@modrinth.com)",
            env!("CARGO_PKG_VERSION")
        ))
        .unwrap();
        headers.insert(reqwest::header::USER_AGENT, header);
        reqwest::Client::builder()
            .tcp_keepalive(Some(time::Duration::from_secs(10)))
            .default_headers(headers)
            .build()
            .expect("Reqwest Client Building Failed")
    };
}
const FETCH_ATTEMPTS: usize = 3;

#[tracing::instrument(skip(semaphore))]
pub async fn fetch(
    url: &str,
    sha1: Option<&str>,
    semaphore: &RwLock<Semaphore>,
) -> crate::Result<Bytes> {
    fetch_advanced(Method::GET, url, sha1, None, None, semaphore).await
}

#[tracing::instrument(skip(json_body, semaphore))]
pub async fn fetch_json<T>(
    method: Method,
    url: &str,
    sha1: Option<&str>,
    json_body: Option<serde_json::Value>,
    semaphore: &RwLock<Semaphore>,
) -> crate::Result<T>
where
    T: DeserializeOwned,
{
    let result =
        fetch_advanced(method, url, sha1, json_body, None, semaphore).await?;
    let value = serde_json::from_slice(&result)?;
    Ok(value)
}

/// Downloads a file with retry and checksum functionality
#[tracing::instrument(skip(json_body, semaphore))]
pub async fn fetch_advanced(
    method: Method,
    url: &str,
    sha1: Option<&str>,
    json_body: Option<serde_json::Value>,
    header: Option<(&str, &str)>,
    semaphore: &RwLock<Semaphore>,
) -> crate::Result<Bytes> {
    let io_semaphore = semaphore.read().await;
    let _permit = io_semaphore.acquire().await?;

    for attempt in 1..=(FETCH_ATTEMPTS + 1) {
        let mut req = REQWEST_CLIENT.request(method.clone(), url);

        if let Some(body) = json_body.clone() {
            req = req.json(&body);
        }

        if let Some(header) = header {
            req = req.header(header.0, header.1);
        }

        let result = req.send().await;
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
    semaphore: &RwLock<Semaphore>,
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
    semaphore: &RwLock<Semaphore>,
) -> crate::Result<()> {
    let io_semaphore = semaphore.read().await;
    let _permit = io_semaphore.acquire().await?;

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
    semaphore: &RwLock<Semaphore>,
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
