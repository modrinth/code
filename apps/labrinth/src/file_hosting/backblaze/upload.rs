use super::authorization::UploadUrlData;
use crate::file_hosting::FileHostingError;
use bytes::Bytes;
use hex::ToHex;
use serde::{Deserialize, Serialize};
use sha1::Digest;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UploadFileData {
    pub file_id: String,
    pub file_name: String,
    pub account_id: String,
    pub bucket_id: String,
    pub content_length: u32,
    pub content_sha1: String,
    pub content_md5: Option<String>,
    pub content_type: String,
    pub upload_timestamp: u64,
}

//Content Types found here: https://www.backblaze.com/b2/docs/content-types.html
pub async fn upload_file(
    url_data: &UploadUrlData,
    content_type: &str,
    file_name: &str,
    file_bytes: Bytes,
) -> Result<UploadFileData, FileHostingError> {
    let response = reqwest::Client::new()
        .post(&url_data.upload_url)
        .header(
            reqwest::header::AUTHORIZATION,
            &url_data.authorization_token,
        )
        .header("X-Bz-File-Name", file_name)
        .header(reqwest::header::CONTENT_TYPE, content_type)
        .header(reqwest::header::CONTENT_LENGTH, file_bytes.len())
        .header(
            "X-Bz-Content-Sha1",
            sha1::Sha1::digest(&file_bytes).encode_hex::<String>(),
        )
        .body(file_bytes)
        .send()
        .await?;

    super::process_response(response).await
}
