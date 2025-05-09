use crate::file_hosting::{
    DeleteFileData, FileHost, FileHostingError, UploadFileData,
};
use async_trait::async_trait;
use bytes::Bytes;
use chrono::Utc;
use hex::ToHex;
use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
use sha2::Digest;

pub struct S3Host {
    bucket: Bucket,
}

impl S3Host {
    pub fn new(
        bucket_name: &str,
        bucket_region: &str,
        url: &str,
        access_token: &str,
        secret: &str,
    ) -> Result<S3Host, FileHostingError> {
        let bucket = Bucket::new(
            bucket_name,
            if bucket_region == "r2" {
                Region::R2 {
                    account_id: url.to_string(),
                }
            } else {
                Region::Custom {
                    region: bucket_region.to_string(),
                    endpoint: url.to_string(),
                }
            },
            Credentials::new(
                Some(access_token),
                Some(secret),
                None,
                None,
                None,
            )
            .map_err(|_| {
                FileHostingError::S3Error(
                    "Error while creating credentials".to_string(),
                )
            })?,
        )
        .map_err(|_| {
            FileHostingError::S3Error(
                "Error while creating Bucket instance".to_string(),
            )
        })?;

        Ok(S3Host { bucket: *bucket })
    }
}

#[async_trait]
impl FileHost for S3Host {
    async fn upload_file(
        &self,
        content_type: &str,
        file_name: &str,
        file_bytes: Bytes,
    ) -> Result<UploadFileData, FileHostingError> {
        let content_sha1 = sha1::Sha1::digest(&file_bytes).encode_hex();
        let content_sha512 = format!("{:x}", sha2::Sha512::digest(&file_bytes));

        self.bucket
            .put_object_with_content_type(
                format!("/{file_name}"),
                &file_bytes,
                content_type,
            )
            .await
            .map_err(|err| {
                FileHostingError::S3Error(format!(
                    "Error while uploading file {file_name} to S3: {err}"
                ))
            })?;

        Ok(UploadFileData {
            file_id: file_name.to_string(),
            file_name: file_name.to_string(),
            content_length: file_bytes.len() as u32,
            content_sha512,
            content_sha1,
            content_md5: None,
            content_type: content_type.to_string(),
            upload_timestamp: Utc::now().timestamp() as u64,
        })
    }

    async fn delete_file_version(
        &self,
        file_id: &str,
        file_name: &str,
    ) -> Result<DeleteFileData, FileHostingError> {
        self.bucket
            .delete_object(format!("/{file_name}"))
            .await
            .map_err(|err| {
                FileHostingError::S3Error(format!(
                    "Error while deleting file {file_name} to S3: {err}"
                ))
            })?;

        Ok(DeleteFileData {
            file_id: file_id.to_string(),
            file_name: file_name.to_string(),
        })
    }
}
