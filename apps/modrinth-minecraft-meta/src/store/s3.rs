use std::pin::Pin;

use anyhow::{Context, Result, ensure};
use derive_more::Debug;
use s3::{Bucket, Region, creds::Credentials};
use tracing::info_span;
use tracing_anyhow::FutureContext;
use url::Url;

use crate::{
    config::S3,
    store::{ContentType, StoreOps},
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

    fn put<'a>(
        &self,
        path: &'a str,
        data: &'a [u8],
        content_type: ContentType,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>> {
        let bucket = self.bucket.clone();
        Box::pin(async move {
            let response = bucket
                .put_object_with_content_type(path, data, content_type.as_str())
                .context(info_span!("writing S3 object", path))
                .await?;
            ensure!(
                (200..300).contains(&response.status_code()),
                "S3 PUT {path} returned HTTP {}",
                response.status_code()
            );
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_urls_encode_keys_and_preserve_prefix() {
        let store = S3Store {
            bucket: Bucket::new(
                "test",
                Region::Custom {
                    region: "test".into(),
                    endpoint: "https://s3.example.com".into(),
                },
                Credentials::anonymous().unwrap(),
            )
            .unwrap(),
            base_url: "https://cdn.example.com/prefix/".parse().unwrap(),
        };
        assert_eq!(
            store
                .url_for("minecraft/v0/versions/3D Shareware v1.34.json")
                .as_str(),
            "https://cdn.example.com/prefix/minecraft/v0/versions/3D%20Shareware%20v1.34.json"
        );
        assert_eq!(
            store.url_for("maven/a+b/file.jar").as_str(),
            "https://cdn.example.com/prefix/maven/a+b/file.jar"
        );
    }
}
