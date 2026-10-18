use std::pin::Pin;

use anyhow::{Context, Result, ensure};
use derive_more::Debug;
use s3::{Bucket, Region, creds::Credentials, error::S3Error};
use tracing::info_span;
use tracing_anyhow::FutureContext;
use url::Url;

use crate::{
    config::S3,
    store::{ContentType, PutMode, PutStatus, StoreOps},
    util::Sha256,
};

#[derive(Debug)]
pub struct S3Store {
    #[debug(skip)]
    bucket: Box<Bucket>,
    base_url: Url,
}

pub fn new(config: &S3) -> Result<S3Store> {
    ensure!(!config.bucket.is_empty(), "S3 bucket must not be empty");

    for (name, url) in [
        ("endpoint", &config.endpoint),
        ("base_url", &config.base_url),
    ] {
        ensure!(
            matches!(url.scheme(), "http" | "https")
                && url.host_str().is_some(),
            "S3 {name} must be an HTTP(S) URL"
        );
    }
    let credentials =
        Credentials::default().context("loading S3 credentials")?;
    let bucket = Bucket::new(
        &config.bucket,
        Region::Custom {
            region: config.region.clone(),
            endpoint: config.endpoint.to_string(),
        },
        credentials,
    )
    .context("creating S3 bucket client")?;
    let bucket = if config.path_style {
        bucket.with_path_style()
    } else {
        bucket
    };
    Ok(S3Store {
        bucket,
        base_url: config.base_url.clone(),
    })
}

async fn matches_hash(
    bucket: &Bucket,
    path: &str,
    sha256: Sha256,
) -> Result<bool> {
    let response = bucket
        .head_object(path)
        .context(info_span!("reading S3 object metadata", path))
        .await;
    match response {
        Ok((_, 404)) => Ok(false),
        Ok((object, status)) => {
            ensure!(
                (200..300).contains(&status),
                "S3 HEAD {path} returned HTTP {status}"
            );
            Ok(object
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.get("sha256"))
                == Some(&sha256.to_string()))
        }
        Err(err) => {
            if matches!(
                err.downcast_ref::<S3Error>(),
                Some(S3Error::HttpFailWithBody(404, _))
            ) {
                return Ok(false);
            }
            Err(err)
        }
    }
}

impl StoreOps for S3Store {
    fn url_for(&self, path: &str) -> Url {
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .expect("S3 base URL supports path segments")
            .pop_if_empty()
            .extend(path.split('/'));
        url
    }

    fn get<'a>(
        &self,
        path: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + 'a>> {
        let bucket = self.bucket.clone();
        Box::pin(async move {
            let response = bucket
                .get_object(path)
                .context(info_span!("reading S3 object", path))
                .await?;
            ensure!(
                (200..300).contains(&response.status_code()),
                "S3 GET {path} returned HTTP {}",
                response.status_code()
            );
            Ok(response.to_vec())
        })
    }

    fn matches_hash<'a>(
        &self,
        path: &'a str,
        sha256: Sha256,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + 'a>> {
        let bucket = self.bucket.clone();
        Box::pin(async move { matches_hash(&bucket, path, sha256).await })
    }

    fn put<'a>(
        &self,
        path: &'a str,
        data: &'a [u8],
        content_type: ContentType,
        mode: PutMode,
    ) -> Pin<Box<dyn Future<Output = Result<PutStatus>> + 'a>> {
        let bucket = self.bucket.clone();
        Box::pin(async move {
            let sha256 = match mode {
                PutMode::Check => {
                    let sha256 = Sha256::from_digest(data);
                    if matches_hash(&bucket, path, sha256).await? {
                        return Ok(PutStatus::Unchanged);
                    }
                    sha256
                }
                PutMode::Prechecked(sha256) => sha256,
            }
            .to_string();

            let response = bucket
                .put_object_builder(path, data)
                .with_content_type(content_type.as_str())
                .with_metadata("sha256", &sha256)?
                .execute()
                .context(info_span!("writing S3 object", path))
                .await?;
            ensure!(
                (200..300).contains(&response.status_code()),
                "S3 PUT {path} returned HTTP {}",
                response.status_code()
            );
            Ok(PutStatus::Changed)
        })
    }
}
