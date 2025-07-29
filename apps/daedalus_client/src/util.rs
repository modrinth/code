use crate::{Error, ErrorKind};
use bytes::Bytes;
use s3::creds::Credentials;
use s3::{Bucket, Region};
use serde::de::DeserializeOwned;
use std::sync::{Arc, LazyLock};
use tokio::sync::Semaphore;

static BUCKET: LazyLock<Bucket> = LazyLock::new(|| {
    let region = dotenvy::var("S3_REGION").unwrap();
    let b = Bucket::new(
        &dotenvy::var("S3_BUCKET_NAME").unwrap(),
        if &*region == "r2" {
            Region::R2 {
                account_id: dotenvy::var("S3_URL").unwrap(),
            }
        } else {
            Region::Custom {
                region: region.clone(),
                endpoint: dotenvy::var("S3_URL").unwrap(),
            }
        },
        Credentials::new(
            Some(&*dotenvy::var("S3_ACCESS_TOKEN").unwrap()),
            Some(&*dotenvy::var("S3_SECRET").unwrap()),
            None,
            None,
            None,
        )
        .unwrap(),
    )
    .unwrap();

    if region == "path-style" {
        *b.with_path_style()
    } else {
        *b
    }
});

pub static REQWEST_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    let mut headers = reqwest::header::HeaderMap::new();
    if let Ok(header) = reqwest::header::HeaderValue::from_str(&format!(
        "modrinth/daedalus/{} (support@modrinth.com)",
        env!("CARGO_PKG_VERSION")
    )) {
        headers.insert(reqwest::header::USER_AGENT, header);
    }

    reqwest::Client::builder()
        .tcp_keepalive(Some(std::time::Duration::from_secs(10)))
        .timeout(std::time::Duration::from_secs(15))
        .default_headers(headers)
        .build()
        .unwrap()
});

#[tracing::instrument(skip(bytes, semaphore))]
pub async fn upload_file_to_bucket(
    path: String,
    bytes: Bytes,
    content_type: Option<String>,
    semaphore: &Arc<Semaphore>,
) -> Result<(), Error> {
    let _permit = semaphore.acquire().await?;
    let key = path.clone();

    const RETRIES: i32 = 3;
    for attempt in 1..=(RETRIES + 1) {
        tracing::trace!("Attempting file upload, attempt {attempt}");
        let result = if let Some(ref content_type) = content_type {
            BUCKET
                .put_object_with_content_type(key.clone(), &bytes, content_type)
                .await
        } else {
            BUCKET.put_object(key.clone(), &bytes).await
        }
        .map_err(|err| ErrorKind::S3 {
            inner: Box::new(err),
            file: path.clone(),
        });

        match result {
            Ok(_) => return Ok(()),
            Err(_) if attempt <= RETRIES => continue,
            Err(_) => {
                result?;
            }
        }
    }
    unreachable!()
}

pub async fn upload_url_to_bucket_mirrors(
    upload_path: String,
    mirrors: Vec<String>,
    sha1: Option<String>,
    semaphore: &Arc<Semaphore>,
) -> Result<(), Error> {
    if mirrors.is_empty() {
        return Err(ErrorKind::InvalidInput(
            "No mirrors provided!".to_string(),
        )
        .into());
    }

    for (index, mirror) in mirrors.iter().enumerate() {
        let result = upload_url_to_bucket(
            upload_path.clone(),
            mirror.clone(),
            sha1.clone(),
            semaphore,
        )
        .await;

        if result.is_ok() || (result.is_err() && index == (mirrors.len() - 1)) {
            return result;
        }
    }

    unreachable!()
}

#[tracing::instrument(skip(semaphore))]
pub async fn upload_url_to_bucket(
    path: String,
    url: String,
    sha1: Option<String>,
    semaphore: &Arc<Semaphore>,
) -> Result<(), Error> {
    let data = download_file(&url, sha1.as_deref(), semaphore).await?;

    upload_file_to_bucket(path, data, None, semaphore).await?;

    Ok(())
}

#[tracing::instrument(skip(bytes))]
pub async fn sha1_async(bytes: Bytes) -> Result<String, Error> {
    let hash = tokio::task::spawn_blocking(move || {
        sha1_smol::Sha1::from(bytes).hexdigest()
    })
    .await?;

    Ok(hash)
}

#[tracing::instrument(skip(semaphore))]
pub async fn download_file(
    url: &str,
    sha1: Option<&str>,
    semaphore: &Arc<Semaphore>,
) -> Result<bytes::Bytes, crate::Error> {
    let _permit = semaphore.acquire().await?;
    tracing::trace!("Starting file download");

    const RETRIES: u32 = 10;
    for attempt in 1..=(RETRIES + 1) {
        let result = REQWEST_CLIENT
            .get(url.replace("http://", "https://"))
            .send()
            .await
            .and_then(|x| x.error_for_status());

        match result {
            Ok(x) => {
                let bytes = x.bytes().await;

                if let Ok(bytes) = bytes {
                    if let Some(sha1) = sha1 {
                        if &*sha1_async(bytes.clone()).await? != sha1 {
                            if attempt <= 3 {
                                continue;
                            } else {
                                return Err(
                                    crate::ErrorKind::ChecksumFailure {
                                        hash: sha1.to_string(),
                                        url: url.to_string(),
                                        tries: attempt,
                                    }
                                    .into(),
                                );
                            }
                        }
                    }

                    return Ok(bytes);
                } else if attempt <= RETRIES {
                    continue;
                } else if let Err(err) = bytes {
                    return Err(crate::ErrorKind::Fetch {
                        inner: err,
                        item: url.to_string(),
                    }
                    .into());
                }
            }
            Err(_) if attempt <= RETRIES => continue,
            Err(err) => {
                return Err(crate::ErrorKind::Fetch {
                    inner: err,
                    item: url.to_string(),
                }
                .into());
            }
        }
    }

    unreachable!()
}

pub async fn fetch_json<T: DeserializeOwned>(
    url: &str,
    semaphore: &Arc<Semaphore>,
) -> Result<T, Error> {
    Ok(serde_json::from_slice(
        &download_file(url, None, semaphore).await?,
    )?)
}

pub async fn fetch_xml<T: DeserializeOwned>(
    url: &str,
    semaphore: &Arc<Semaphore>,
) -> Result<T, Error> {
    Ok(serde_xml_rs::from_reader(
        &*download_file(url, None, semaphore).await?,
    )?)
}

pub fn format_url(path: &str) -> String {
    format!("{}/{}", &*dotenvy::var("BASE_URL").unwrap(), path)
}
