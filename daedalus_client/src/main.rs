use log::{error, info, warn};
use rusoto_core::credential::StaticProvider;
use rusoto_core::{HttpClient, Region, RusotoError};
use rusoto_s3::{PutObjectError, S3Client};
use rusoto_s3::{PutObjectRequest, S3};

mod fabric;
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
}

#[tokio::main]
async fn main() {
    if check_env_vars() {
        error!("Some environment variables are missing!");

        return;
    }

    fabric::retrieve_data().await.unwrap();
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
        .flatten();

    if do_integration.unwrap_or(false) {
        failed |= check_var::<bool>("DO_ACCESS_KEY");
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
) -> Result<(), Error> {
    CLIENT
        .put_object(PutObjectRequest {
            bucket: dotenv::var("S3_BUCKET_NAME").unwrap(),
            key: format!("{}/{}", &*dotenv::var("BASE_FOLDER").unwrap(), path),
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
