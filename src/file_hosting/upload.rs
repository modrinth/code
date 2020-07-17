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

#[cfg(feature = "backblaze")]
//Content Types found here: https://www.backblaze.com/b2/docs/content-types.html
pub async fn upload_file(
    url_data: &UploadUrlData,
    content_type: &str,
    file_name: &str,
    file_bytes: Vec<u8>,
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
            sha1::Sha1::from(&file_bytes).hexdigest(),
        )
        .body(file_bytes)
        .send()
        .await?;

    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        Err(FileHostingError::BackblazeError(response.json().await?))
    }
}

#[cfg(not(feature = "backblaze"))]
pub async fn upload_file(
    _url_data: &UploadUrlData,
    content_type: &str,
    file_name: &str,
    file_bytes: Vec<u8>,
) -> Result<UploadFileData, FileHostingError> {
    let path = std::path::Path::new(&dotenv::var("MOCK_FILE_PATH").unwrap())
        .join(file_name.replace("../", ""));
    std::fs::create_dir_all(path.parent().ok_or(FileHostingError::InvalidFilename)?)?;
    let content_sha1 = sha1::Sha1::from(&file_bytes).hexdigest();

    std::fs::write(path, &file_bytes)?;
    Ok(UploadFileData {
        file_id: String::from("MOCK_FILE_ID"),
        file_name: file_name.to_string(),
        account_id: String::from("MOCK_ACCOUNT_ID"),
        bucket_id: String::from("MOCK_BUCKET_ID"),
        content_length: file_bytes.len() as u32,
        content_sha1,
        content_md5: None,
        content_type: content_type.to_string(),
        upload_timestamp: chrono::Utc::now().timestamp_millis() as u64,
    })
}
