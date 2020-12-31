use crate::file_hosting::{DeleteFileData, FileHost, FileHostingError, UploadFileData};
use async_trait::async_trait;
use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;

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
        file_bytes: Vec<u8>,
    ) -> Result<UploadFileData, FileHostingError> {
        let content_sha1 = sha1::Sha1::from(&file_bytes).hexdigest();

        self.bucket
            .put_object_with_content_type(
                format!("/{}", file_name),
                file_bytes.as_slice(),
                content_type,
            )
            .await?;

        let provider = &*dotenv::var("S3_PROVIDER").unwrap();

        if provider == "do" {
            reqwest::Client::new()
                .delete(&*format!(
                    "https://api.digitalocean.com/v2/cdn/endpoints/{}/cache",
                    self.bucket.name
                ))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .header(
                    reqwest::header::AUTHORIZATION,
                    self.bucket
                        .credentials
                        .secret_key
                        .clone()
                        .unwrap_or_else(|| "".to_string()),
                )
                .body(
                    serde_json::json!({
                        "files": vec![file_name],
                    })
                    .to_string(),
                )
                .send()
                .await?;
        }

        Ok(UploadFileData {
            file_id: file_name.to_string(),
            file_name: file_name.to_string(),
            content_length: file_bytes.len() as u32,
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

#[cfg(test)]
mod tests {
    use crate::file_hosting::s3_host::S3Host;
    use crate::file_hosting::FileHost;

    #[actix_rt::test]
    async fn test_file_management() {
        let s3_host = S3Host::new(
            &*dotenv::var("S3_BUCKET_NAME").unwrap(),
            &*dotenv::var("S3_REGION").unwrap(),
            &*dotenv::var("S3_URL").unwrap(),
            &*dotenv::var("S3_ACCESS_TOKEN").unwrap(),
            &*dotenv::var("S3_SECRET").unwrap(),
        )
        .unwrap();

        s3_host
            .upload_file(
                "text/plain",
                "test.txt",
                "test file".to_string().into_bytes(),
            )
            .await
            .unwrap();

        s3_host.delete_file_version("", "test.txt").await.unwrap();
    }
}
