use crate::file_hosting::FileHostingError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationPermissions {
    bucket_id: Option<String>,
    bucket_name: Option<String>,
    capabilities: Vec<String>,
    name_prefix: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationData {
    pub absolute_minimum_part_size: i32,
    pub account_id: String,
    pub allowed: AuthorizationPermissions,
    pub api_url: String,
    pub authorization_token: String,
    pub download_url: String,
    pub recommended_part_size: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UploadUrlData {
    pub bucket_id: String,
    pub upload_url: String,
    pub authorization_token: String,
}

#[cfg(feature = "backblaze")]
pub async fn authorize_account(
    key_id: String,
    application_key: String,
) -> Result<AuthorizationData, FileHostingError> {
    let combined_key = format!("{}:{}", key_id, application_key);
    let formatted_key = format!("Basic {}", base64::encode(combined_key));

    let response = reqwest::Client::new()
        .get("https://api.backblazeb2.com/b2api/v2/b2_authorize_account")
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::AUTHORIZATION, formatted_key)
        .send()
        .await?;

    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        Err(FileHostingError::BackblazeError(response.json().await?))
    }
}

#[cfg(feature = "backblaze")]
pub async fn get_upload_url(
    authorization_data: AuthorizationData,
    bucket_id: String,
) -> Result<UploadUrlData, FileHostingError> {
    let response = reqwest::Client::new()
        .post(&format!("{}/b2api/v2/b2_get_upload_url", authorization_data.api_url).to_string())
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(
            reqwest::header::AUTHORIZATION,
            authorization_data.authorization_token,
        )
        .body(
            serde_json::json!({
                "bucketId": bucket_id,
            })
            .to_string(),
        )
        .send()
        .await?;

    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        Err(FileHostingError::BackblazeError(response.json().await?))
    }
}

#[cfg(not(feature = "backblaze"))]
pub async fn authorize_account(
    _key_id: String,
    _application_key: String,
) -> Result<AuthorizationData, FileHostingError> {
    Ok(AuthorizationData {
        absolute_minimum_part_size: 5000000,
        account_id: String::from("MOCK_ACCOUNT_ID"),
        allowed: AuthorizationPermissions {
            bucket_id: None,
            bucket_name: None,
            capabilities: vec![
                String::from("listKeys"),
                String::from("writeKeys"),
                String::from("deleteKeys"),
                String::from("listAllBucketNames"),
                String::from("listBuckets"),
                String::from("writeBuckets"),
                String::from("deleteBuckets"),
                String::from("readBuckets"),
                String::from("listFiles"),
                String::from("readFiles"),
                String::from("shareFiles"),
                String::from("writeFiles"),
                String::from("deleteFiles"),
            ],
            name_prefix: None,
        },
        api_url: String::from("https://api.example.com"),
        authorization_token: String::from("MOCK_AUTH_TOKEN"),
        download_url: String::from("https://download.example.com"),
        recommended_part_size: 100000000,
    })
}

#[cfg(not(feature = "backblaze"))]
pub async fn get_upload_url(
    _authorization_data: AuthorizationData,
    _bucket_id: String,
) -> Result<UploadUrlData, FileHostingError> {
    Ok(UploadUrlData {
        bucket_id: String::from("MOCK_BUCKET_ID"),
        upload_url: String::from("https://download.example.com"),
        authorization_token: String::from("MOCK_AUTH_TOKEN"),
    })
}
