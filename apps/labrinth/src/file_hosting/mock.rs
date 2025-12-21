use super::{
    DeleteFileData, FileHost, FileHostPublicity, FileHostingError,
    UploadFileData,
};
use async_trait::async_trait;
use bytes::Bytes;
use chrono::Utc;
use hex::ToHex;
use sha2::Digest;
use std::path::PathBuf;

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
        file_publicity: FileHostPublicity,
        file_bytes: Bytes,
    ) -> Result<UploadFileData, FileHostingError> {
        let file_name = urlencoding::decode(file_name)
            .map_err(|_| FileHostingError::InvalidFilename)?;
        let path = get_file_path(&file_name, file_publicity);
        std::fs::create_dir_all(
            path.parent().ok_or(FileHostingError::InvalidFilename)?,
        )?;
        let content_sha1 = sha1::Sha1::digest(&file_bytes).encode_hex();
        let content_sha512 = format!("{:x}", sha2::Sha512::digest(&file_bytes));

        std::fs::write(path, &*file_bytes)?;
        Ok(UploadFileData {
            file_name: file_name.to_string(),
            file_publicity,
            content_length: file_bytes.len() as u32,
            content_sha512,
            content_sha1,
            content_md5: None,
            content_type: content_type.to_string(),
            upload_timestamp: Utc::now().timestamp() as u64,
        })
    }

    async fn get_url_for_private_file(
        &self,
        file_name: &str,
        _expiry_secs: u32,
    ) -> Result<String, FileHostingError> {
        let cdn_url = dotenvy::var("CDN_URL").unwrap();
        Ok(format!("{cdn_url}/private/{file_name}"))
    }

    async fn delete_file(
        &self,
        file_name: &str,
        file_publicity: FileHostPublicity,
    ) -> Result<DeleteFileData, FileHostingError> {
        let path = get_file_path(file_name, file_publicity);
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(DeleteFileData {
            file_name: file_name.to_string(),
        })
    }
}

fn get_file_path(
    file_name: &str,
    file_publicity: FileHostPublicity,
) -> PathBuf {
    let mut path = PathBuf::from(dotenvy::var("MOCK_FILE_PATH").unwrap());

    if matches!(file_publicity, FileHostPublicity::Private) {
        path.push("private");
    }
    path.push(file_name.replace("../", ""));

    path
}
