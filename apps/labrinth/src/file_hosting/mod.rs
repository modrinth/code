use async_trait::async_trait;
use thiserror::Error;

mod mock;
mod s3_host;

use ariadne::i18n_enum;
use bytes::Bytes;
use derive_more::Display;
pub use mock::MockHost;
pub use s3_host::{S3BucketConfig, S3Host};

#[derive(Error, Debug)]
pub enum FileHostingError {
    #[error("{0}: {1}")]
    S3Error(S3ErrorAction, s3::error::S3Error),
    #[error("File system error in file hosting: {0}")]
    FileSystemError(#[from] std::io::Error),
    #[error("Invalid Filename")]
    InvalidFilename,
}

i18n_enum!(
    FileHostingError,
    root_key: "labrinth.error.file_hosting_error",
    S3Error(action, cause) => "s3",
    FileSystemError(cause) => "file_system",
    InvalidFilename! => "invalid_filename",
);

#[derive(Copy, Clone, Debug, Display)]
pub enum S3ErrorAction {
    #[display("S3 error when creating bucket instance")]
    CreatingBucketInstance,
    #[display("S3 error when uploading file")]
    UploadingFile,
    #[display("S3 error when generating presigned URL")]
    GeneratingPresignedUrl,
    #[display("S3 error when deleting file")]
    DeletingFile,
}

i18n_enum!(
    S3ErrorAction,
    root_key: "labrinth.error.file_hosting_error.s3",
    CreatingBucketInstance! => "creating_bucket_instance",
    UploadingFile! => "uploading_file",
    GeneratingPresignedUrl! => "generating_presigned_url",
    DeletingFile! => "deleting_file",
);

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
