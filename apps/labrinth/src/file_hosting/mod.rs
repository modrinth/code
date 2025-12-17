use std::convert::Infallible;

use actix_http::header::HeaderName;
use actix_utils::future::{Ready, ready};
use actix_web::FromRequest;
use async_trait::async_trait;
use thiserror::Error;

mod mock;
mod s3_host;

use bytes::Bytes;
pub use mock::MockHost;
pub use s3_host::{S3BucketConfig, S3Host};

#[derive(Error, Debug)]
pub enum FileHostingError {
    #[error("S3 error when {0}: {1}")]
    S3Error(&'static str, s3::error::S3Error),
    #[error("File system error in file hosting: {0}")]
    FileSystemError(#[from] std::io::Error),
    #[error("Invalid Filename")]
    InvalidFilename,
}

#[derive(Debug, Clone)]
pub struct UploadFileData {
    pub file_name: String,
    pub file_publicity: FileHostPublicity,
    pub content_length: u32,
    pub content_sha512: String,
    pub content_sha1: String,
    pub content_md5: Option<String>,
    pub content_type: String,
    pub upload_timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct DeleteFileData {
    pub file_name: String,
}

#[derive(Debug, Copy, Clone)]
pub enum FileHostPublicity {
    Public,
    Private,
}

#[async_trait]
pub trait FileHost {
    async fn upload_file(
        &self,
        content_type: &str,
        file_name: &str,
        file_publicity: FileHostPublicity,
        file_bytes: Bytes,
    ) -> Result<UploadFileData, FileHostingError>;

    async fn get_url_for_private_file(
        &self,
        file_name: &str,
        expiry_secs: u32,
    ) -> Result<String, FileHostingError>;

    async fn delete_file(
        &self,
        file_name: &str,
        file_publicity: FileHostPublicity,
    ) -> Result<DeleteFileData, FileHostingError>;
}

#[derive(Debug, Clone, Copy)]
pub struct UseAltCdn(pub bool);

const HEADER_NAME: HeaderName = HeaderName::from_static("labrinth-alt-cdn");

impl FromRequest for UseAltCdn {
    type Error = Infallible;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_http::Payload,
    ) -> Self::Future {
        let Some(use_alt_cdn) = req.headers().get(HEADER_NAME) else {
            return ready(Ok(Self(false)));
        };
        let use_alt_cdn = use_alt_cdn.as_bytes() == b"true";
        ready(Ok(Self(use_alt_cdn)))
    }
}
