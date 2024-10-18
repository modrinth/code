use super::{DeleteFileData, FileHost, FileHostingError, UploadFileData};
use async_trait::async_trait;
use bytes::Bytes;
use reqwest::Response;
use serde::Deserialize;
use sha2::Digest;

mod authorization;
mod delete;
mod upload;

pub struct BackblazeHost {
    upload_url_data: authorization::UploadUrlData,
    authorization_data: authorization::AuthorizationData,
}

impl BackblazeHost {
    pub async fn new(key_id: &str, key: &str, bucket_id: &str) -> Self {
        let authorization_data =
            authorization::authorize_account(key_id, key).await.unwrap();
        let upload_url_data =
            authorization::get_upload_url(&authorization_data, bucket_id)
                .await
                .unwrap();

        BackblazeHost {
            upload_url_data,
            authorization_data,
        }
    }
}

#[async_trait]
impl FileHost for BackblazeHost {
    async fn upload_file(
        &self,
        content_type: &str,
        file_name: &str,
        file_bytes: Bytes,
    ) -> Result<UploadFileData, FileHostingError> {
        let content_sha512 = format!("{:x}", sha2::Sha512::digest(&file_bytes));

        let upload_data = upload::upload_file(
            &self.upload_url_data,
            content_type,
            file_name,
            file_bytes,
        )
        .await?;
        Ok(UploadFileData {
            file_id: upload_data.file_id,
            file_name: upload_data.file_name,
            content_length: upload_data.content_length,
            content_sha512,
            content_sha1: upload_data.content_sha1,
            content_md5: upload_data.content_md5,
            content_type: upload_data.content_type,
            upload_timestamp: upload_data.upload_timestamp,
        })
    }

    /*
        async fn upload_file_streaming(
            &self,
            content_type: &str,
            file_name: &str,
            stream: reqwest::Body
        ) -> Result<UploadFileData, FileHostingError> {
            use futures::stream::StreamExt;

            let mut data = Vec::new();
            while let Some(chunk) = stream.next().await {
                data.extend_from_slice(&chunk.map_err(|e| FileHostingError::Other(e))?);
            }
            self.upload_file(content_type, file_name, data).await
        }
    */

    async fn delete_file_version(
        &self,
        file_id: &str,
        file_name: &str,
    ) -> Result<DeleteFileData, FileHostingError> {
        let delete_data = delete::delete_file_version(
            &self.authorization_data,
            file_id,
            file_name,
        )
        .await?;
        Ok(DeleteFileData {
            file_id: delete_data.file_id,
            file_name: delete_data.file_name,
        })
    }
}

pub async fn process_response<T>(
    response: Response,
) -> Result<T, FileHostingError>
where
    T: for<'de> Deserialize<'de>,
{
    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        Err(FileHostingError::BackblazeError(response.json().await?))
    }
}
