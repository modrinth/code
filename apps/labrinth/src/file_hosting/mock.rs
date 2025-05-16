use super::{DeleteFileData, FileHost, FileHostingError, UploadFileData};
use async_trait::async_trait;
use bytes::Bytes;
use chrono::Utc;
use hex::ToHex;
use sha2::Digest;

#[derive(Default)]
pub struct MockHost(());

impl MockHost {
    pub fn new() -> Self {
        MockHost(())
    }
}

#[async_trait]
impl FileHost for MockHost {
    async fn upload_file(
        &self,
        content_type: &str,
        file_name: &str,
        file_bytes: Bytes,
    ) -> Result<UploadFileData, FileHostingError> {
        let path =
            std::path::Path::new(&dotenvy::var("MOCK_FILE_PATH").unwrap())
                .join(file_name.replace("../", ""));
        std::fs::create_dir_all(
            path.parent().ok_or(FileHostingError::InvalidFilename)?,
        )?;
        let content_sha1 = sha1::Sha1::digest(&file_bytes).encode_hex();
        let content_sha512 = format!("{:x}", sha2::Sha512::digest(&file_bytes));

        std::fs::write(path, &*file_bytes)?;
        Ok(UploadFileData {
            file_id: String::from("MOCK_FILE_ID"),
            file_name: file_name.to_string(),
            content_length: file_bytes.len() as u32,
            content_sha512,
            content_sha1,
            content_md5: None,
            content_type: content_type.to_string(),
            upload_timestamp: Utc::now().timestamp() as u64,
        })
    }

    async fn delete_file_version(
        &self,
        file_id: &str,
        file_name: &str,
    ) -> Result<DeleteFileData, FileHostingError> {
        let path =
            std::path::Path::new(&dotenvy::var("MOCK_FILE_PATH").unwrap())
                .join(file_name.replace("../", ""));
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(DeleteFileData {
            file_id: file_id.to_string(),
            file_name: file_name.to_string(),
        })
    }
}
