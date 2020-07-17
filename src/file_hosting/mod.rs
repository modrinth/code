use thiserror::Error;

mod authorization;
mod delete;
mod upload;

pub use authorization::authorize_account;
pub use authorization::get_upload_url;
pub use authorization::AuthorizationData;
pub use authorization::AuthorizationPermissions;
pub use authorization::UploadUrlData;

pub use upload::upload_file;
pub use upload::UploadFileData;

pub use delete::delete_file_version;
pub use delete::DeleteFileData;

#[derive(Error, Debug)]
pub enum FileHostingError {
    #[cfg(feature = "backblaze")]
    #[error("Error while accessing the data from backblaze")]
    HttpError(#[from] reqwest::Error),

    #[cfg(feature = "backblaze")]
    #[error("Backblaze error: {0}")]
    BackblazeError(serde_json::Value),

    #[cfg(not(feature = "backblaze"))]
    #[error("File system error in file hosting: {0}")]
    FileSystemError(#[from] std::io::Error),
    #[cfg(not(feature = "backblaze"))]
    #[error("Invalid Filename")]
    InvalidFilename,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn test_authorization() {
        println!("{}", dotenv::var("BACKBLAZE_BUCKET_ID").unwrap());
        let authorization_data = authorize_account(
            dotenv::var("BACKBLAZE_KEY_ID").unwrap(),
            dotenv::var("BACKBLAZE_KEY").unwrap(),
        )
        .await
        .unwrap();

        get_upload_url(
            authorization_data,
            dotenv::var("BACKBLAZE_BUCKET_ID").unwrap(),
        )
        .await
        .unwrap();
    }

    #[actix_rt::test]
    async fn test_file_management() {
        let authorization_data = authorize_account(
            dotenv::var("BACKBLAZE_KEY_ID").unwrap(),
            dotenv::var("BACKBLAZE_KEY").unwrap(),
        )
        .await
        .unwrap();
        let upload_url_data = get_upload_url(
            authorization_data.clone(),
            dotenv::var("BACKBLAZE_BUCKET_ID").unwrap(),
        )
        .await
        .unwrap();
        let upload_data = upload_file(
            &upload_url_data,
            "text/plain",
            "test.txt",
            "test file".to_string().into_bytes(),
        )
        .await
        .unwrap();

        delete_file_version(
            &authorization_data,
            &upload_data.file_id,
            &upload_data.file_name,
        )
        .await
        .unwrap();
    }
}
