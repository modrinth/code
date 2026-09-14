use crate::state::content_store::adapters::filesystem;
use crate::state::content_store::adapters::filesystem::{
    symlink_metadata_if_exists, validate_parent_directories,
};
use crate::state::content_store::adapters::sqlite as catalog;
use crate::state::content_store::{
    ContentStore, FileContent, InstanceFileStorage, ReadableContent,
    StoredFileHandle, StoredFileMetadata, StoredFileRecord, StoredFileStatus,
    content_file_path, hash_file_with_progress, input, validate_digest,
    validate_relative,
};
use crate::state::{InstanceFile, file_modified_at_ns};
use itertools::Itertools;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::OwnedRwLockReadGuard;
impl ContentStore {
    pub(in crate::state::content_store) fn path(
        &self,
        stored_file: &StoredFileMetadata,
    ) -> crate::Result<PathBuf> {
        validate_digest(&stored_file.sha512, 128)?;
        let prefix = format!(
            "objects/sha512/{}/{}/",
            &stored_file.sha512[..2],
            stored_file.sha512
        );
        if !matches!(
            stored_file.relative_path.strip_prefix(&prefix),
            Some("payload.jar" | "payload.bin")
        ) {
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

    pub(in crate::state::content_store) async fn lookup_with_guard(
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
            .exactly_one();
        let Ok(stored_file) = candidates else {
            return Ok(None);
        };
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
        let Some(metadata) = symlink_metadata_if_exists(&path).await? else {
            return self.quarantine(stored_file).await;
        };
        if !metadata.is_file() {
            return self.quarantine(stored_file).await;
        }
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
            return self.quarantine(stored_file).await;
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
        let Some(canonical) = filesystem::canonical_path(path).await? else {
            return Ok(None);
        };
        let Ok(relative) = canonical.strip_prefix(&self.root) else {
            return Ok(None);
        };
        let Some((objects, algorithm, prefix, hash, payload)) =
            relative.components().collect_tuple()
        else {
            return Ok(None);
        };
        if objects.as_os_str() != "objects"
            || algorithm.as_os_str() != "sha512"
            || !matches!(
                payload.as_os_str().to_str(),
                Some("payload.jar" | "payload.bin")
            )
        {
            return Ok(None);
        }
        let Some(hash) = hash.as_os_str().to_str() else {
            return Ok(None);
        };
        if validate_digest(hash, 128).is_err()
            || !hash
                .starts_with(prefix.as_os_str().to_str().unwrap_or_default())
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
        self.file_content_with_binding(file, binding).await
    }

    pub(crate) async fn file_content_with_binding(
        &self,
        file: &InstanceFile,
        binding: InstanceFileStorage,
    ) -> crate::Result<FileContent> {
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
        if !filesystem::is_regular_file(&path, true).await? {
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
        let Some(stored_file) = catalog::find_file(&self.pool, sha512).await?
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

    pub(in crate::state::content_store) async fn validate_object_parent(
        &self,
        path: &Path,
    ) -> crate::Result<()> {
        validate_parent_directories(&self.root, path).await
    }

    async fn quarantine(
        &self,
        stored_file: &StoredFileMetadata,
    ) -> crate::Result<bool> {
        catalog::set_file_status(
            &self.pool,
            &stored_file.sha512,
            StoredFileStatus::Quarantined,
        )
        .await?;
        Ok(false)
    }
}
