use crate::file_hosting::{AuthorizationData, FileHostingError};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeleteFileData {
    pub file_id: String,
    pub file_name: String,
}

#[cfg(feature = "backblaze")]
pub async fn delete_file_version(
    authorization_data: &AuthorizationData,
    file_id: &str,
    file_name: &str,
) -> Result<DeleteFileData, FileHostingError> {
    let response = reqwest::Client::new()
        .post(&format!(
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

    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        Err(FileHostingError::BackblazeError(response.json().await?))
    }
}

#[cfg(not(feature = "backblaze"))]
pub async fn delete_file_version(
    _authorization_data: &AuthorizationData,
    file_id: &str,
    file_name: &str,
) -> Result<DeleteFileData, FileHostingError> {
    let path = std::path::Path::new(&dotenv::var("MOCK_FILE_PATH").unwrap())
        .join(file_name.replace("../", ""));
    std::fs::remove_file(path)?;

    Ok(DeleteFileData {
        file_id: file_id.to_string(),
        file_name: file_name.to_string(),
    })
}
