use super::file_io::validate_parent_directories;
use super::{
    ContentStore, FileContent, ReadableContent, StoredFileHandle,
    StoredFileMetadata, StoredFileRecord, StoredFileStatus, catalog,
    content_file_path, hash_file_with_progress, input, validate_digest,
    validate_relative,
};
use crate::state::{InstanceFile, file_modified_at_ns};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::OwnedRwLockReadGuard;
impl ContentStore {
    pub(super) fn path(
        &self,
        stored_file: &StoredFileMetadata,
    ) -> crate::Result<PathBuf> {
        validate_digest(&stored_file.sha512, 128)?;
        let prefix = format!(
            "objects/sha512/{}/{}/",
            &stored_file.sha512[..2],
            stored_file.sha512
        );
        if !stored_file.relative_path.starts_with(&prefix)
            || !matches!(
                stored_file.relative_path.strip_prefix(&prefix),
                Some("payload.jar" | "payload.bin")
            )
        {
            return Err(input("Invalid content store object path"));
        }
        Ok(self.root.join(&stored_file.relative_path))
    }

    pub(crate) async fn lookup(
        &self,
        sha512: Option<&str>,
        sha1: Option<&str>,
        size: Option<u64>,
    ) -> crate::Result<Option<StoredFileHandle>> {
        if let Some(hash) = sha512 {
            validate_digest(hash, 128)?;
        }
        if let Some(hash) = sha1 {
            validate_digest(hash, 40)?;
        }
        let guard = self.lease().await;
        self.lookup_with_guard(sha512, sha1, size, guard).await
    }

    pub(super) async fn lookup_with_guard(
        &self,
        sha512: Option<&str>,
        sha1: Option<&str>,
        size: Option<u64>,
        guard: Arc<OwnedRwLockReadGuard<()>>,
    ) -> crate::Result<Option<StoredFileHandle>> {
        let candidates = catalog::find_files(&self.pool, sha512, sha1)
            .await?
            .into_iter()
            .filter(|stored_file| {
                sha1.is_none_or(|expected| stored_file.sha1 == expected)
            })
            .filter(|stored_file| {
                sha512.is_some()
                    || size.is_none_or(|size| stored_file.size as u64 == size)
            })
            .collect::<Vec<_>>();
        if candidates.len() != 1 {
            return Ok(None);
        }
        let stored_file = candidates
            .into_iter()
            .next()
            .ok_or_else(|| input("Missing content object"))?;
        if !self.is_healthy(&stored_file, false).await? {
            return Ok(None);
        }
        if let Some(expected_size) = size
            && expected_size != stored_file.size as u64
        {
            tracing::warn!(
                expected_size,
                actual_size = stored_file.size,
                sha512 = %stored_file.sha512,
                "Cached content matches SHA-512 but differs from the declared size; using its actual size"
            );
        }
        catalog::mark_file_used(&self.pool, &stored_file.sha512).await?;
        Ok(Some(StoredFileHandle {
            path: self.path(&stored_file)?,
            metadata: stored_file,
            _guard: guard,
        }))
    }

    pub(crate) async fn is_healthy(
        &self,
        stored_file: &StoredFileMetadata,
        verify: bool,
    ) -> crate::Result<bool> {
        self.is_healthy_with_progress(stored_file, verify, &|_| {})
            .await
    }

    pub(crate) async fn is_healthy_with_progress(
        &self,
        stored_file: &StoredFileMetadata,
        verify: bool,
        on_read: &(dyn Fn(u64) + Send + Sync),
    ) -> crate::Result<bool> {
        if stored_file.status != StoredFileStatus::Ready && !verify {
            return Ok(false);
        }
        let path = self.path(stored_file)?;
        self.validate_object_parent(&path).await?;
        let metadata = match fs::symlink_metadata(&path).await {
            Ok(metadata) if metadata.is_file() => metadata,
            Ok(_) => {
                catalog::set_file_status(
                    &self.pool,
                    &stored_file.sha512,
                    StoredFileStatus::Quarantined,
                )
                .await?;
                return Ok(false);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                catalog::set_file_status(
                    &self.pool,
                    &stored_file.sha512,
                    StoredFileStatus::Quarantined,
                )
                .await?;
                return Ok(false);
            }
            Err(error) => return Err(error.into()),
        };
        let modified_at_ns = file_modified_at_ns(&metadata)? as i64;
        if !verify
            && metadata.len() == stored_file.size as u64
            && modified_at_ns == stored_file.modified_at_ns
        {
            return Ok(true);
        }
        let hashes = hash_file_with_progress(&path, on_read).await?;
        if hashes.sha512 != stored_file.sha512
            || hashes.sha1 != stored_file.sha1
            || hashes.size != stored_file.size as u64
        {
            catalog::set_file_status(
                &self.pool,
                &stored_file.sha512,
                StoredFileStatus::Quarantined,
            )
            .await?;
            return Ok(false);
        }
        let mut verified = stored_file.clone();
        verified.modified_at_ns = modified_at_ns;
        catalog::save_file(&self.pool, &verified).await?;
        Ok(true)
    }

    pub(crate) async fn find_file_by_path(
        &self,
        path: &Path,
    ) -> crate::Result<Option<StoredFileHandle>> {
        let canonical = match fs::canonicalize(path).await {
            Ok(path) => path,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(None);
            }
            Err(error) => return Err(error.into()),
        };
        let Ok(relative) = canonical.strip_prefix(&self.root) else {
            return Ok(None);
        };
        let components = relative.components().collect::<Vec<_>>();
        if components.len() != 5 {
            return Ok(None);
        }
        let Some(hash) = components[3].as_os_str().to_str() else {
            return Ok(None);
        };
        if hash.len() != 128
            || !hash.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Ok(None);
        }
        let Some(stored_file) = self.lookup(Some(hash), None, None).await?
        else {
            return Ok(None);
        };
        Ok((stored_file.path == canonical).then_some(stored_file))
    }

    pub(crate) async fn file_content(
        &self,
        file: &InstanceFile,
    ) -> crate::Result<FileContent> {
        let Some(binding) = catalog::file_storage(&self.pool, &file.id).await?
        else {
            return Ok(FileContent::Unmanaged);
        };
        let stored_file = self
            .lookup(Some(&binding.blob_sha512), None, Some(file.size))
            .await?;
        Ok(match stored_file {
            Some(stored_file) => FileContent::Stored {
                storage: binding,
                stored_file,
            },
            None => FileContent::Damaged(binding),
        })
    }

    pub(crate) async fn read_path(
        &self,
        file: &InstanceFile,
        instance_path: &str,
    ) -> crate::Result<ReadableContent> {
        match self.file_content(file).await? {
            FileContent::Stored { stored_file, .. } => {
                return Ok(ReadableContent::Stored(stored_file));
            }
            FileContent::Damaged(_) => {
                return Err(input(format!(
                    "{} needs repair or re-import",
                    file.relative_path
                )));
            }
            FileContent::Unmanaged => {}
        }
        validate_relative(instance_path)?;
        let relative_path = content_file_path(file);
        validate_relative(&relative_path)?;
        let path = self.profiles.join(instance_path).join(relative_path);
        if !fs::metadata(&path).await?.is_file() {
            return Err(input("Only regular content files can be read"));
        }
        Ok(ReadableContent::Local(path))
    }

    pub(crate) async fn instance_path(
        &self,
        instance_path: &str,
        relative_path: &str,
    ) -> crate::Result<PathBuf> {
        validate_relative(instance_path)?;
        validate_relative(relative_path)?;
        let base = self.profiles.join(instance_path);
        let destination = base.join(relative_path);
        validate_parent_directories(&self.profiles, &destination).await?;
        Ok(destination)
    }

    pub(crate) async fn get_file_record(
        &self,
        sha512: &str,
    ) -> crate::Result<Option<StoredFileRecord>> {
        let guard = self.lease().await;
        let Some(stored_file) =
            catalog::find_files(&self.pool, Some(sha512), None)
                .await?
                .into_iter()
                .next()
        else {
            return Ok(None);
        };
        let path = self.path(&stored_file)?;
        self.validate_object_parent(&path).await?;
        Ok(Some(StoredFileRecord {
            metadata: stored_file,
            path,
            _guard: guard,
        }))
    }

    pub(super) async fn validate_object_parent(
        &self,
        path: &Path,
    ) -> crate::Result<()> {
        validate_parent_directories(&self.root, path).await
    }
}
