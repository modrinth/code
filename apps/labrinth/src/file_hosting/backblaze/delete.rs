use super::authorization::AuthorizationData;
use crate::file_hosting::FileHostingError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeleteFileData {
    pub file_id: String,
    pub file_name: String,
}

pub async fn delete_file_version(
    authorization_data: &AuthorizationData,
    file_id: &str,
    file_name: &str,
) -> Result<DeleteFileData, FileHostingError> {
    let response = reqwest::Client::new()
        .post(format!(
            "{}/b2api/v2/b2_delete_file_version",
            authorization_data.api_url
        ))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(
            reqwest::header::AUTHORIZATION,
            &authorization_data.authorization_token,
        )
        .body(
            serde_json::json!({
                "fileName": file_name,
                "fileId": file_id
            })
            .to_string(),
        )
        .send()
        .await?;

    super::process_response(response).await
}
