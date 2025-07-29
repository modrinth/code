use crate::file_hosting::{
    DeleteFileData, FileHost, FileHostPublicity, FileHostingError,
    UploadFileData,
};
use async_trait::async_trait;
use bytes::Bytes;
use chrono::Utc;
use hex::ToHex;
use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
use sha2::Digest;

pub struct S3BucketConfig {
    pub name: String,
    pub uses_path_style: bool,
    pub region: String,
    pub url: String,
    pub access_token: String,
    pub secret: String,
}

pub struct S3Host {
    public_bucket: Bucket,
    private_bucket: Bucket,
}

impl S3Host {
    pub fn new(
        public_bucket: S3BucketConfig,
        private_bucket: S3BucketConfig,
    ) -> Result<S3Host, FileHostingError> {
        let create_bucket =
            |config: S3BucketConfig| -> Result<_, FileHostingError> {
                let mut bucket = Bucket::new(
                    "",
                    if config.region == "r2" {
                        Region::R2 {
                            account_id: config.url,
                        }
                    } else {
                        Region::Custom {
                            region: config.region,
                            endpoint: config.url,
                        }
                    },
                    Credentials {
                        access_key: Some(config.access_token),
                        secret_key: Some(config.secret),
                        ..Credentials::anonymous().unwrap()
                    },
                )
                .map_err(|e| {
                    FileHostingError::S3Error("creating Bucket instance", e)
                })?;

                bucket.name = config.name;
                if config.uses_path_style {
                    bucket.set_path_style();
                } else {
                    bucket.set_subdomain_style();
                }

                Ok(bucket)
            };

        Ok(S3Host {
            public_bucket: *create_bucket(public_bucket)?,
            private_bucket: *create_bucket(private_bucket)?,
        })
    }

    fn get_bucket(&self, publicity: FileHostPublicity) -> &Bucket {
        match publicity {
            FileHostPublicity::Public => &self.public_bucket,
            FileHostPublicity::Private => &self.private_bucket,
        }
    }
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

        self.get_bucket(file_publicity)
            .put_object_with_content_type(
                format!("/{file_name}"),
                &file_bytes,
                content_type,
            )
            .await
            .map_err(|e| FileHostingError::S3Error("uploading file", e))?;

        Ok(UploadFileData {
            file_name: file_name.to_string(),
            file_publicity,
            content_length: file_bytes.len() as u32,
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
        let url = self
            .private_bucket
            .presign_get(format!("/{file_name}"), expiry_secs, None)
            .await
            .map_err(|e| {
                FileHostingError::S3Error("generating presigned URL", e)
            })?;
        Ok(url)
    }

    async fn delete_file(
        &self,
        file_name: &str,
        file_publicity: FileHostPublicity,
    ) -> Result<DeleteFileData, FileHostingError> {
        self.get_bucket(file_publicity)
            .delete_object(format!("/{file_name}"))
            .await
            .map_err(|e| FileHostingError::S3Error("deleting file", e))?;

        Ok(DeleteFileData {
            file_name: file_name.to_string(),
        })
    }
}
