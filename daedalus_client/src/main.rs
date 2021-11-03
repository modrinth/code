use log::{error, warn};
use rusoto_core::credential::StaticProvider;
use rusoto_core::{HttpClient, Region, RusotoError};
use rusoto_s3::{PutObjectError, S3Client};
use rusoto_s3::{PutObjectRequest, S3};
use std::time::Duration;

mod fabric;
mod forge;
mod minecraft;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    DaedalusError(#[from] daedalus::Error),
    #[error("Error while deserializing JSON")]
    SerdeError(#[from] serde_json::Error),
    #[error("Unable to fetch {item}")]
    FetchError { inner: reqwest::Error, item: String },
    #[error("Error while managing asynchronous tasks")]
    TaskError(#[from] tokio::task::JoinError),
    #[error("Error while uploading file to S3")]
    S3Error {
        inner: RusotoError<PutObjectError>,
        file: String,
    },
    #[error("Error while parsing version as semver: {0}")]
    SemVerError(#[from] semver::Error),
    #[error("Error while reading zip file: {0}")]
    ZipError(#[from] zip::result::ZipError),
    #[error("Error while reading zip file: {0}")]
    IoError(#[from] std::io::Error),
}

#[tokio::main]
async fn main() {
    env_logger::init();

    if check_env_vars() {
        error!("Some environment variables are missing!");

        return;
    }

    let mut timer = tokio::time::interval(Duration::from_secs(10 * 60));

    loop {
        timer.tick().await;
        tokio::spawn(async {
            let mut uploaded_files = Vec::new();

            match fabric::retrieve_data(&mut uploaded_files).await {
                Ok(..) => {}
                Err(err) => error!("{:?}", err),
            };
            match minecraft::retrieve_data(&mut uploaded_files).await {
                Ok(..) => {}
                Err(err) => error!("{:?}", err),
            };
            match forge::retrieve_data(&mut uploaded_files).await {
                Ok(..) => {}
                Err(err) => error!("{:?}", err),
            };

            match  purge_digitalocean_cache(uploaded_files).await {
                Ok(..) => {}
                Err(err) => error!("{:?}", err),
            };
        });
    }
}

fn check_env_vars() -> bool {
    let mut failed = false;

    fn check_var<T: std::str::FromStr>(var: &str) -> bool {
        if dotenv::var(var)
            .ok()
            .and_then(|s| s.parse::<T>().ok())
            .is_none()
        {
            warn!(
                "Variable `{}` missing in dotenv or not of type `{}`",
                var,
                std::any::type_name::<T>()
            );
            true
        } else {
            false
        }
    }

    failed |= check_var::<String>("BASE_URL");
    failed |= check_var::<String>("BASE_FOLDER");

    failed |= check_var::<String>("S3_ACCESS_TOKEN");
    failed |= check_var::<String>("S3_SECRET");
    failed |= check_var::<String>("S3_URL");
    failed |= check_var::<String>("S3_REGION");
    failed |= check_var::<String>("S3_BUCKET_NAME");

    failed |= check_var::<bool>("DO_INTEGRATION");

    let do_integration = dotenv::var("DO_INTEGRATION")
        .ok()
        .map(|x| x.parse::<bool>().ok())
        .flatten()
        .unwrap_or(false);

    if do_integration {
        failed |= check_var::<String>("DO_ACCESS_KEY");
        failed |= check_var::<String>("DO_ENDPOINT_ID");
    }

    failed
}

lazy_static::lazy_static! {
    static ref CLIENT : S3Client = S3Client::new_with(
            HttpClient::new().unwrap(),
            StaticProvider::new(
                dotenv::var("S3_ACCESS_TOKEN").unwrap(),
                dotenv::var("S3_SECRET").unwrap(),
                None,
                None,
            ),
            Region::Custom {
                name: dotenv::var("S3_REGION").unwrap(),
                endpoint: dotenv::var("S3_URL").unwrap(),
            },
        );
}

pub async fn upload_file_to_bucket(
    path: String,
    bytes: Vec<u8>,
    content_type: Option<String>,
    uploaded_files: &tokio::sync::Mutex<Vec<String>>,
) -> Result<(), Error> {
    let key = format!("{}/{}", &*dotenv::var("BASE_FOLDER").unwrap(), path);

    CLIENT
        .put_object(PutObjectRequest {
            bucket: dotenv::var("S3_BUCKET_NAME").unwrap(),
            key: key.clone(),
            body: Some(bytes.into()),
            acl: Some("public-read".to_string()),
            content_type,
            ..Default::default()
        })
        .await
        .map_err(|err| Error::S3Error {
            inner: err,
            file: format!("{}/{}", &*dotenv::var("BASE_FOLDER").unwrap(), path),
        })?;

    {
        let mut uploaded_files = uploaded_files.lock().await;
        uploaded_files.push(key);
    }

    Ok(())
}

pub fn format_url(path: &str) -> String {
    format!(
        "{}/{}/{}",
        &*dotenv::var("BASE_URL").unwrap(),
        &*dotenv::var("BASE_FOLDER").unwrap(),
        path
    )
}

#[derive(serde::Serialize)]
struct PurgeCacheRequest {
    pub files: Vec<String>,
}

pub async fn purge_digitalocean_cache(files: Vec<String>) -> Result<(), Error> {
    if !dotenv::var("DO_INTEGRATION")
        .ok()
        .map(|x| x.parse::<bool>().ok())
        .flatten()
        .unwrap_or(false) {

        return Ok(())
    }

    let client = reqwest::Client::new();

    client
        .delete(&format!(
            "https://api.digitalocean.com/v2/cdn/endpoints/{}/cache",
            &*dotenv::var("DO_ENDPOINT_ID").unwrap()
        ))
        .header("Authorization", &*format!("Bearer {}", &*dotenv::var("DO_ACCESS_KEY").unwrap()))
        .json(&PurgeCacheRequest { files })
        .send().await.map_err(|err| Error::FetchError {
            inner: err,
            item: "purging digital ocean cache".to_string()
        })?;

    Ok(())
}
