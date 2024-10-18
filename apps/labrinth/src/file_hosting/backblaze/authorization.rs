use crate::file_hosting::FileHostingError;
use base64::Engine;
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

pub async fn authorize_account(
    key_id: &str,
    application_key: &str,
) -> Result<AuthorizationData, FileHostingError> {
    let combined_key = format!("{key_id}:{application_key}");
    let formatted_key = format!(
        "Basic {}",
        base64::engine::general_purpose::STANDARD.encode(combined_key)
    );

    let response = reqwest::Client::new()
        .get("https://api.backblazeb2.com/b2api/v2/b2_authorize_account")
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::AUTHORIZATION, formatted_key)
        .send()
        .await?;

    super::process_response(response).await
}

pub async fn get_upload_url(
    authorization_data: &AuthorizationData,
    bucket_id: &str,
) -> Result<UploadUrlData, FileHostingError> {
    let response = reqwest::Client::new()
        .post(
            format!(
                "{}/b2api/v2/b2_get_upload_url",
                authorization_data.api_url
            )
            .to_string(),
        )
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(
            reqwest::header::AUTHORIZATION,
            &authorization_data.authorization_token,
        )
        .body(
            serde_json::json!({
                "bucketId": bucket_id,
            })
            .to_string(),
        )
        .send()
        .await?;

    super::process_response(response).await
}
