use crate::state::content_store::adapters::downloads;
use crate::state::content_store::adapters::filesystem;
use crate::state::content_store::adapters::sqlite as catalog;
use crate::state::content_store::{
    ContentStore, GetFileResult, StoredFileHandle, StoredFileMetadata,
    StoredFileStatus, input, validate_digest,
};
use crate::util::content_hash::{hash_file, temporary_file};
use crate::util::fetch::{self, DownloadMeta, FetchProgressFn, FetchSemaphore};
use sha2::{Digest, Sha512};
use std::path::Path;
use tokio::fs::File;

impl ContentStore {
    pub(crate) async fn get_or_download_file(
        &self,
        mirrors: &[&str],
        sha512: Option<&str>,
        sha1: Option<&str>,
        size: Option<u64>,
        download_meta: Option<&DownloadMeta>,
        semaphore: &FetchSemaphore,
        progress: Option<&mut FetchProgressFn<'_>>,
    ) -> crate::Result<GetFileResult> {
        let key = sha512
            .map(|hash| format!("sha512:{hash}"))
            .or_else(|| sha1.map(|hash| format!("sha1:{hash}")));
        let _acquisition = if let Some(key) = &key {
            Some(self.download_lock(key).lock().await)
        } else {
            None
        };
        if let Some(stored_file) = self.lookup(sha512, sha1, size).await? {
            return Ok(GetFileResult {
                stored_file,
                reused: true,
            });
        }
        let download = downloads::download(
            &self.staging,
            &self.pool,
            mirrors,
            sha1,
            download_meta,
            semaphore,
            progress,
        )
        .await?;
        if let Some(expected_sha512) = sha512
            && expected_sha512 != download.sha512
        {
            return Err(input(format!(
                "Downloaded content SHA-512 mismatch: expected {expected_sha512}, got {}",
                download.sha512,
            )));
        }
        if let Some(expected_size) = size
            && expected_size != download.size
        {
            if sha512.is_none() {
                return Err(input(format!(
                    "Downloaded content size mismatch: expected {expected_size} bytes, got {} bytes",
                    download.size,
                )));
            }
            tracing::warn!(
                expected_size,
                actual_size = download.size,
                sha512 = %download.sha512,
                "Downloaded content matches SHA-512 but differs from the declared size; using its actual size"
            );
        }
        let sources = mirrors
            .iter()
            .filter_map(|source| {
                let url = url::Url::parse(source).ok()?;
                (url.scheme() == "https"
                    && url.username().is_empty()
                    && url.password().is_none()
                    && url.query().is_none())
                .then(|| source.to_string())
            })
            .collect::<Vec<_>>();
        let stored_file = self
            .save_staged_file(download.into_staged()?, &sources)
            .await?;
        Ok(GetFileResult {
            stored_file,
            reused: false,
        })
    }

    pub(crate) async fn store_file(
        &self,
        source: &Path,
    ) -> crate::Result<StoredFileHandle> {
        if let Some(stored_file) = self.find_file_by_path(source).await? {
            return Ok(stored_file);
        }
		let hashes = hash_file(source).await?;
		if let Some(stored_file) = self
			.lookup(
				Some(&hashes.sha512),
				Some(&hashes.sha1),
				Some(hashes.size),
			)
			.await?
		{
			return Ok(stored_file);
		}
        let staged = self.stage_file(source).await?;
        self.save_staged_file(staged, &[]).await
    }

    pub(in crate::state::content_store) async fn save_staged_file(
        &self,
        staged: fetch::StagedDownload,
        sources: &[String],
    ) -> crate::Result<StoredFileHandle> {
        if staged.path.parent() != Some(self.staging.as_path()) {
            return Err(input("Content was not staged in the shared store"));
        }
        let fetch::StagedDownload {
            path: temporary,
            size,
            sha1,
            sha512: hash,
            archive,
        } = staged;
        let guard = self.lease().await;
        let _publication = self.publish_lock(&hash).lock().await;
        if let Some(existing) = self
            .lookup_with_guard(Some(&hash), None, Some(size), guard.clone())
            .await?
        {
            return Ok(existing);
        }
        let relative_path = format!(
            "objects/sha512/{}/{}/payload.{}",
            &hash[..2],
            hash,
            if archive { "jar" } else { "bin" }
        );
        let destination = self.root.join(&relative_path);
        filesystem::publish_staged_file(
            &self.root,
            &destination,
            temporary,
            &hash,
        )
        .await?;
        let stored_file = StoredFileMetadata {
            sha512: hash,
            sha1,
            size: size
                .try_into()
                .map_err(|_| input("Content file is too large"))?,
            relative_path,
            status: StoredFileStatus::Ready,
            modified_at_ns: filesystem::modified_at_ns(&destination).await?,
            last_used_at: chrono::Utc::now().timestamp(),
            sources: catalog::encode_sources(sources)?,
        };
        catalog::save_file(&self.pool, &stored_file).await?;
        Ok(StoredFileHandle {
            metadata: stored_file,
            path: destination,
            _guard: guard,
        })
    }

    pub(crate) async fn store_bytes(
        &self,
        bytes: &[u8],
        expected_sha1: Option<&str>,
    ) -> crate::Result<StoredFileHandle> {
        let sha1 = sha1_smol::Sha1::from(bytes).hexdigest();
        if let Some(expected) = expected_sha1 {
            validate_digest(expected, 40)?;
            if sha1 != expected {
                return Err(input(
                    "Content bytes do not match the expected hash",
                ));
            }
        }
        let (mut file, temporary) = self.temporary().await?;
        filesystem::write_staged_bytes(&mut file, bytes).await?;
        drop(file);
        self.save_staged_file(
            fetch::StagedDownload {
                path: temporary,
                size: bytes.len() as u64,
                sha1,
                sha512: format!("{:x}", Sha512::digest(bytes)),
                archive: bytes.starts_with(b"PK"),
            },
            &[],
        )
        .await
    }

    pub(crate) async fn temporary(
        &self,
    ) -> crate::Result<(File, tempfile::TempPath)> {
        temporary_file(Some(&self.staging)).await
    }
    pub(crate) fn require_staging_space(&self, size: u64) -> crate::Result<()> {
        filesystem::require_staging_space(&self.staging, size)
    }

    pub(in crate::state::content_store) async fn stage_file(
        &self,
        source: &Path,
    ) -> crate::Result<fetch::StagedDownload> {
        filesystem::stage_file(&self.staging, source).await
    }
}
