use crate::file_hosting::authorization::UploadUrlData;
use crate::file_hosting::FileHostingError;
use serde::{Deserialize, Serialize};

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
    url_data: UploadUrlData,
    content_type: String,
    file_name: String,
    file_bytes: Vec<u8>,
) -> Result<UploadFileData, FileHostingError> {
    Ok(reqwest::Client::new()
        .post(&url_data.upload_url)
        .header(reqwest::header::AUTHORIZATION, url_data.authorization_token)
        .header("X-Bz-File-Name", file_name)
        .header(reqwest::header::CONTENT_TYPE, content_type)
        .header(reqwest::header::CONTENT_LENGTH, file_bytes.len())
        .header(
            "X-Bz-Content-Sha1",
            sha1::Sha1::from(&file_bytes).hexdigest(),
        )
        .body(file_bytes)
        .send()
        .await?
        .json()
        .await?)
}
