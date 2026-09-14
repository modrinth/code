use super::{
    ContentStore, GetFileResult, StoredFileHandle, StoredFileMetadata,
    StoredFileStatus, catalog, hash_file, input, sync_directory,
    validate_digest,
};
use crate::state::file_modified_at_ns;
use crate::util::fetch::{self, DownloadMeta, FetchProgressFn, FetchSemaphore};
use sha2::{Digest, Sha512};
use std::path::Path;
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
        let download = fetch::fetch_file_mirrors_in(
            mirrors,
            sha1,
            download_meta,
            None,
            semaphore,
            &self.pool,
            progress,
            Some(&self.staging),
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
        self.store_file_with_sources(source, &[]).await
    }

    async fn store_file_with_sources(
        &self,
        source: &Path,
        sources: &[String],
    ) -> crate::Result<StoredFileHandle> {
        if let Some(stored_file) = self.find_file_by_path(source).await? {
            return Ok(stored_file);
        }
        let super::FileHashes { sha512, sha1, size } =
            hash_file(source).await?;
        if let Some(stored_file) =
            self.lookup(Some(&sha512), Some(&sha1), Some(size)).await?
        {
            return Ok(stored_file);
        }
        self.require_staging_space(size)?;
        let staged = self.stage_file(source).await?;
        self.save_staged_file(staged, sources).await
    }

    pub(crate) fn require_staging_space(&self, size: u64) -> crate::Result<()> {
        if fs4::available_space(&self.staging)?
            < size.saturating_add(64 * 1024 * 1024)
        {
            return Err(std::io::Error::new(
				std::io::ErrorKind::StorageFull,
				"There is not enough space to import this file into the shared content store",
			).into());
        }
        Ok(())
    }

    pub(super) async fn stage_file(
        &self,
        source: &Path,
    ) -> crate::Result<fetch::StagedDownload> {
        let mut input_file = File::open(source).await?;
        let before = input_file.metadata().await?;
        if !before.is_file() {
            return Err(input("Only regular content files can be stored"));
        }
        let temporary = self.temporary().await?;
        let mut output = File::create(&temporary).await?;
        let mut sha512 = Sha512::new();
        let mut sha1 = sha1_smol::Sha1::new();
        let mut buffer = vec![0; 256 * 1024];
        let mut size = 0_u64;
        let mut prefix = [0_u8; 2];
        let mut prefix_len = 0;
        loop {
            let count = input_file.read(&mut buffer).await?;
            if count == 0 {
                break;
            }
            let prefix_count = (prefix.len() - prefix_len).min(count);
            prefix[prefix_len..prefix_len + prefix_count]
                .copy_from_slice(&buffer[..prefix_count]);
            prefix_len += prefix_count;
            output.write_all(&buffer[..count]).await?;
            sha512.update(&buffer[..count]);
            sha1.update(&buffer[..count]);
            size += count as u64;
        }
        output.sync_all().await?;
        drop(output);
        let after = input_file.metadata().await?;
        if before.len() != after.len()
            || before.modified()? != after.modified()?
            || size != after.len()
        {
            return Err(input(
                "Content changed while it was being imported; try again after closing the instance",
            ));
        }
        Ok(fetch::StagedDownload {
            path: temporary,
            size,
            sha1: sha1.hexdigest(),
            sha512: format!("{:x}", sha512.finalize()),
            archive: prefix_len == prefix.len() && prefix == *b"PK",
        })
    }

    pub(super) async fn save_staged_file(
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
        let parent = destination
            .parent()
            .ok_or_else(|| input("Invalid store path"))?;
        self.validate_object_parent(&destination).await?;
        fs::create_dir_all(parent).await?;
        self.validate_object_parent(&destination).await?;
        if let Ok(metadata) = fs::symlink_metadata(&destination).await {
            if !metadata.is_file()
                || metadata.file_type().is_symlink()
                || hash_file(&destination).await?.sha512 != hash
            {
                return Err(input(format!(
                    "Content object {hash} needs repair before it can be replaced"
                )));
            }
        } else {
            let destination_copy = destination.clone();
            tokio::task::spawn_blocking(move || {
                temporary
                    .persist_noclobber(destination_copy)
                    .map_err(|error| error.error)
            })
            .await??;
        }
        let mut permissions = fs::metadata(&destination).await?.permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&destination, permissions).await?;
        sync_directory(parent).await?;
        let stored_file = StoredFileMetadata {
            sha512: hash,
            sha1,
            size: size
                .try_into()
                .map_err(|_| input("Content file is too large"))?,
            relative_path,
            status: StoredFileStatus::Ready,
            modified_at_ns: file_modified_at_ns(
                &fs::metadata(&destination).await?,
            )? as i64,
            last_used_at: chrono::Utc::now().timestamp(),
            sources: serde_json::to_string(sources)?,
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
        let temporary = self.temporary().await?;
        let mut file = File::create(&temporary).await?;
        file.write_all(bytes).await?;
        file.sync_all().await?;
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

    pub(crate) async fn temporary(&self) -> crate::Result<tempfile::TempPath> {
        let staging = self.staging.clone();
        Ok(tokio::task::spawn_blocking(move || {
            tempfile::NamedTempFile::new_in(staging)
                .map(|file| file.into_temp_path())
        })
        .await??)
    }
}
