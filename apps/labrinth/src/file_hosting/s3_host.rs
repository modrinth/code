use crate::file_hosting::{
    DeleteFileData, FileHost, FileHostPublicity, FileHostingError,
    UploadFileData,
};
use async_trait::async_trait;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use bytes::Bytes;
use chrono::Utc;
use hex::ToHex;
use sha2::Digest;
use std::error::Error;
use std::time::Duration;

pub struct S3BucketConfig {
    pub name: String,
    pub uses_path_style: bool,
    pub region: String,
    pub url: String,
    pub access_token: String,
    pub secret: String,
}

pub struct S3Host {
    public_bucket: S3Bucket,
    private_bucket: S3Bucket,
}

struct S3Bucket {
    name: String,
    client: Client,
}

impl S3Host {
    pub fn new(
        public_bucket: S3BucketConfig,
        private_bucket: S3BucketConfig,
    ) -> Result<S3Host, FileHostingError> {
        let create_bucket = |config: S3BucketConfig| -> S3Bucket {
            let (region, endpoint_url, provider_name) = if config.region == "r2"
            {
                (
                    "auto".to_string(),
                    format!("https://{}.r2.cloudflarestorage.com", config.url),
                    "R2",
                )
            } else {
                (config.region, config.url, "Labrinth")
            };

            let s3_config = aws_sdk_s3::config::Builder::new()
                .behavior_version(BehaviorVersion::latest())
                .region(Region::new(region))
                .endpoint_url(endpoint_url)
                .credentials_provider(Credentials::new(
                    config.access_token,
                    config.secret,
                    None,
                    None,
                    provider_name,
                ))
                .force_path_style(config.uses_path_style)
                .build();

            S3Bucket {
                name: config.name,
                client: Client::from_conf(s3_config),
            }
        };

        Ok(S3Host {
            public_bucket: create_bucket(public_bucket),
            private_bucket: create_bucket(private_bucket),
        })
    }

    fn get_bucket(&self, publicity: FileHostPublicity) -> &S3Bucket {
        match publicity {
            FileHostPublicity::Public => &self.public_bucket,
            FileHostPublicity::Private => &self.private_bucket,
        }
    }
}

fn s3_error(
    context: &'static str,
    error: impl Error + Send + Sync + 'static,
) -> FileHostingError {
    FileHostingError::S3Error(context, Box::new(error))
}

#[async_trait]
impl FileHost for S3Host {
    async fn upload_file(
        &self,
        content_type: &str,
        file_name: &str,
        file_publicity: FileHostPublicity,
        file_bytes: Bytes,
    ) -> Result<UploadFileData, FileHostingError> {
        let content_sha1 = sha1::Sha1::digest(&file_bytes).encode_hex();
        let content_sha512 = format!("{:x}", sha2::Sha512::digest(&file_bytes));
        let content_length = file_bytes.len() as u32;
        let bucket = self.get_bucket(file_publicity);

        bucket
            .client
            .put_object()
            .bucket(bucket.name.as_str())
            .key(file_name)
            .content_type(content_type)
            .body(ByteStream::from(file_bytes))
            .send()
            .await
            .map_err(|e| s3_error("uploading file", e))?;

        Ok(UploadFileData {
            file_name: file_name.to_string(),
            file_publicity,
            content_length,
            content_sha512,
            content_sha1,
            content_md5: None,
            content_type: content_type.to_string(),
            upload_timestamp: Utc::now().timestamp() as u64,
        })
    }

    async fn get_url_for_private_file(
        &self,
        file_name: &str,
        expiry_secs: u32,
    ) -> Result<String, FileHostingError> {
        let presigning_config = PresigningConfig::expires_in(
            Duration::from_secs(expiry_secs.into()),
        )
        .map_err(|e| s3_error("creating presigning config", e))?;
        let url = self
            .private_bucket
            .client
            .get_object()
            .bucket(self.private_bucket.name.as_str())
            .key(file_name)
            .presigned(presigning_config)
            .await
            .map_err(|e| s3_error("generating presigned URL", e))?;
        Ok(url.uri().to_string())
    }

    async fn delete_file(
        &self,
        file_name: &str,
        file_publicity: FileHostPublicity,
    ) -> Result<DeleteFileData, FileHostingError> {
        let bucket = self.get_bucket(file_publicity);

        bucket
            .client
            .delete_object()
            .bucket(bucket.name.as_str())
            .key(file_name)
            .send()
            .await
            .map_err(|e| s3_error("deleting file", e))?;

        Ok(DeleteFileData {
            file_name: file_name.to_string(),
        })
    }

    async fn read_file(
        &self,
        file_name: &str,
        file_publicity: FileHostPublicity,
    ) -> Result<Bytes, FileHostingError> {
        let bucket = self.get_bucket(file_publicity);

        let response = bucket
            .client
            .get_object()
            .bucket(bucket.name.as_str())
            .key(file_name)
            .send()
            .await
            .map_err(|e| s3_error("reading file", e))?;

        Ok(response.body.collect().await.map_err(|e| s3_error("reading file body", e))?.into_bytes())
    }
}
