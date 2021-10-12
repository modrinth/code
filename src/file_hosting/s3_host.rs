use crate::file_hosting::{DeleteFileData, FileHost, FileHostingError, UploadFileData};
use async_trait::async_trait;
use bytes::{Buf, Bytes};
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
        let mut bucket = Bucket::new(
            bucket_name,
            Region::Custom {
                region: bucket_region.to_string(),
                endpoint: url.to_string(),
            },
            Credentials::new(Some(access_token), Some(secret), None, None, None)?,
        )?;

        bucket.add_header("x-amz-acl", "public-read");

        Ok(S3Host { bucket })
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
        let content_sha1 = sha1::Sha1::from(&file_bytes).hexdigest();
        let content_sha512 = format!("{:x}", sha2::Sha512::digest(file_bytes.bytes()));

        self.bucket
            .put_object_with_content_type(
                format!("/{}", file_name),
                file_bytes.bytes(),
                content_type,
            )
            .await?;

        Ok(UploadFileData {
            file_id: file_name.to_string(),
            file_name: file_name.to_string(),
            content_length: file_bytes.len() as u32,
            content_sha512,
            content_sha1,
            content_md5: None,
            content_type: content_type.to_string(),
            upload_timestamp: chrono::Utc::now().timestamp_millis() as u64,
        })
    }

    async fn delete_file_version(
        &self,
        file_id: &str,
        file_name: &str,
    ) -> Result<DeleteFileData, FileHostingError> {
        self.bucket.delete_object(format!("/{}", file_name)).await?;

        Ok(DeleteFileData {
            file_id: file_id.to_string(),
            file_name: file_name.to_string(),
        })
    }
}
