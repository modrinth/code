use async_trait::async_trait;
use thiserror::Error;

mod backblaze;
mod mock;
mod s3_host;

pub use backblaze::BackblazeHost;
use bytes::Bytes;
pub use mock::MockHost;
pub use s3_host::S3Host;

#[derive(Error, Debug)]
pub enum FileHostingError {
    #[error("Error while accessing the data from backblaze")]
    HttpError(#[from] reqwest::Error),
    #[error("Backblaze error: {0}")]
    BackblazeError(serde_json::Value),
    #[error("S3 error: {0}")]
    S3Error(String),
    #[error("File system error in file hosting: {0}")]
    FileSystemError(#[from] std::io::Error),
    #[error("Invalid Filename")]
    InvalidFilename,
}

#[derive(Debug, Clone)]
pub struct UploadFileData {
    pub file_id: String,
    pub file_name: String,
    pub content_length: u32,
    pub content_sha512: String,
    pub content_sha1: String,
    pub content_md5: Option<String>,
    pub content_type: String,
    pub upload_timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct DeleteFileData {
    pub file_id: String,
    pub file_name: String,
}

#[async_trait]
pub trait FileHost {
    async fn upload_file(
        &self,
        content_type: &str,
        file_name: &str,
        file_bytes: Bytes,
    ) -> Result<UploadFileData, FileHostingError>;

    async fn delete_file_version(
        &self,
        file_id: &str,
        file_name: &str,
    ) -> Result<DeleteFileData, FileHostingError>;
}
