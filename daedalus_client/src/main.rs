use log::{error, info, warn};
use s3::creds::Credentials;
use s3::error::S3Error;
use s3::{Bucket, Region};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

mod fabric;
mod forge;
mod minecraft;
mod neo;
mod quilt;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    DaedalusError(#[from] daedalus::Error),
    #[error("Error while deserializing JSON")]
    SerdeError(#[from] serde_json::Error),
    #[error("Error while deserializing XML")]
    XMLError(#[from] serde_xml_rs::Error),
    #[error("Unable to fetch {item}")]
    FetchError { inner: reqwest::Error, item: String },
    #[error("Error while managing asynchronous tasks")]
    TaskError(#[from] tokio::task::JoinError),
    #[error("Error while uploading file to S3")]
    S3Error { inner: S3Error, file: String },
    #[error("Error while parsing version as semver: {0}")]
    SemVerError(#[from] semver::Error),
    #[error("Error while reading zip file: {0}")]
    ZipError(#[from] zip::result::ZipError),
    #[error("Error while reading zip file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Error while obtaining strong reference to Arc")]
    ArcError,
    #[error("Error acquiring semaphore: {0}")]
    AcquireError(#[from] tokio::sync::AcquireError),
}

#[tokio::main]
async fn main() {
    env_logger::init();

    if check_env_vars() {
        error!("Some environment variables are missing!");

        return;
    }

    let mut timer = tokio::time::interval(Duration::from_secs(60 * 60));
    let semaphore = Arc::new(Semaphore::new(10));

    loop {
        timer.tick().await;

        let mut uploaded_files = Vec::new();

        let versions = match minecraft::retrieve_data(
            &mut uploaded_files,
            semaphore.clone(),
        )
        .await
        {
            Ok(res) => Some(res),
            Err(err) => {
                error!("{:?}", err);

                None
            }
        };

        if let Some(manifest) = versions {
            match fabric::retrieve_data(
                &manifest,
                &mut uploaded_files,
                semaphore.clone(),
            )
            .await
            {
                Ok(..) => {}
                Err(err) => error!("{:?}", err),
            };
            match forge::retrieve_data(
                &manifest,
                &mut uploaded_files,
                semaphore.clone(),
            )
            .await
            {
                Ok(..) => {}
                Err(err) => error!("{:?}", err),
            };
            match quilt::retrieve_data(
                &manifest,
                &mut uploaded_files,
                semaphore.clone(),
            )
            .await
            {
                Ok(..) => {}
                Err(err) => error!("{:?}", err),
            };
            match neo::retrieve_data(
                &manifest,
                &mut uploaded_files,
                semaphore.clone(),
            )
            .await
            {
                Ok(..) => {}
                Err(err) => error!("{:?}", err),
            };
        }
    }
}

fn check_env_vars() -> bool {
    let mut failed = false;

    fn check_var<T: std::str::FromStr>(var: &str) -> bool {
        if dotenvy::var(var)
            .ok()
            .and_then(|s| s.parse::<T>().ok())
            .is_none()
        {
            warn!(
                "Variable `{}` missing in dotenvy or not of type `{}`",
                var,
                std::any::type_name::<T>()
            );
            true
        } else {
            false
        }
    }

    failed |= check_var::<String>("BASE_URL");

    failed |= check_var::<String>("S3_ACCESS_TOKEN");
    failed |= check_var::<String>("S3_SECRET");
    failed |= check_var::<String>("S3_URL");
    failed |= check_var::<String>("S3_REGION");
    failed |= check_var::<String>("S3_BUCKET_NAME");

    failed
}

lazy_static::lazy_static! {
    static ref CLIENT : Bucket = {
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
            ).unwrap(),
        ).unwrap();

        if region == "path-style" {
            b.with_path_style()
        } else {
            b
        }
    };
}

pub async fn upload_file_to_bucket(
    path: String,
    bytes: Vec<u8>,
    content_type: Option<String>,
    uploaded_files: &tokio::sync::Mutex<Vec<String>>,
    semaphore: Arc<Semaphore>,
) -> Result<(), Error> {
    let _permit = semaphore.acquire().await?;
    info!("{} started uploading", path);
    let key = path.clone();

    for attempt in 1..=4 {
        let result = if let Some(ref content_type) = content_type {
            CLIENT
                .put_object_with_content_type(key.clone(), &bytes, content_type)
                .await
        } else {
            CLIENT.put_object(key.clone(), &bytes).await
        }
        .map_err(|err| Error::S3Error {
            inner: err,
            file: path.clone(),
        });

        match result {
            Ok(_) => {
                {
                    info!("{} done uploading", path);
                    let mut uploaded_files = uploaded_files.lock().await;
                    uploaded_files.push(key);
                }

                return Ok(());
            }
            Err(_) if attempt <= 3 => continue,
            Err(_) => {
                result?;
            }
        }
    }
    unreachable!()
}

pub fn format_url(path: &str) -> String {
    format!("{}/{}", &*dotenvy::var("BASE_URL").unwrap(), path)
}

pub async fn download_file(
    url: &str,
    sha1: Option<&str>,
    semaphore: Arc<Semaphore>,
) -> Result<bytes::Bytes, Error> {
    let _permit = semaphore.acquire().await?;
    info!("{} started downloading", url);
    let val = daedalus::download_file(url, sha1).await?;
    info!("{} finished downloading", url);
    Ok(val)
}

pub async fn download_file_mirrors(
    base: &str,
    mirrors: &[&str],
    sha1: Option<&str>,
    semaphore: Arc<Semaphore>,
) -> Result<bytes::Bytes, Error> {
    let _permit = semaphore.acquire().await?;
    info!("{} started downloading", base);
    let val = daedalus::download_file_mirrors(base, mirrors, sha1).await?;
    info!("{} finished downloading", base);

    Ok(val)
}
