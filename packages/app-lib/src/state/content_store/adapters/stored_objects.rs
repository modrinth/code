use super::filesystem::{
    symlink_metadata_if_exists, validate_parent_directories,
};
use crate::state::content_store::{
    StoredFileMetadata, StoredFileStatus, input, validate_digest,
};
use crate::util::content_hash::{FileHashes, hash_file};
use std::collections::HashSet;
use std::path::Path;
use tokio::fs::{self, ReadDir};

/// Visits objects in filesystem order so recovery can register each verified file
/// before inspecting the next one. A later I/O error does not discard that progress.
pub(in crate::state::content_store) struct UnregisteredFiles {
    prefixes: ReadDir,
    entries: Option<ReadDir>,
    prefix: String,
    known: HashSet<String>,
}

impl UnregisteredFiles {
    pub(in crate::state::content_store) async fn open(
        root: &Path,
        known: HashSet<String>,
    ) -> crate::Result<Option<Self>> {
        let objects = root.join("objects/sha512");
        validate_parent_directories(root, &objects.join("payload.jar")).await?;
        let prefixes = match fs::read_dir(&objects).await {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(None);
            }
            Err(error) => return Err(error.into()),
        };
        Ok(Some(Self {
            prefixes,
            entries: None,
            prefix: String::new(),
            known,
        }))
    }

    pub(in crate::state::content_store) async fn next_file(
        &mut self,
    ) -> crate::Result<Option<StoredFileMetadata>> {
        loop {
            if let Some(entries) = &mut self.entries {
                while let Some(entry) = entries.next_entry().await? {
                    let hash = entry.file_name().to_string_lossy().into_owned();
                    if validate_digest(&hash, 128).is_err()
                        || !hash.starts_with(&self.prefix)
                        || self.known.contains(&hash)
                        || !entry.file_type().await?.is_dir()
                    {
                        continue;
                    }
                    for filename in ["payload.jar", "payload.bin"] {
                        let path = entry.path().join(filename);
                        let Some(metadata) =
                            symlink_metadata_if_exists(&path).await?
                        else {
                            continue;
                        };
                        if !metadata.is_file() {
                            continue;
                        }
                        let FileHashes { sha512, sha1, size } =
                            hash_file(&path).await?;
                        if sha512 != hash {
                            tracing::warn!(path = %path.display(), "Preserving an unregistered store object with an unexpected hash");
                            continue;
                        }
                        let mut permissions = metadata.permissions();
                        permissions.set_readonly(true);
                        fs::set_permissions(&path, permissions).await?;
                        return Ok(Some(StoredFileMetadata {
                            sha512,
                            sha1,
                            size: size.try_into().map_err(|_| {
                                input("Content file is too large")
                            })?,
                            relative_path: format!(
                                "objects/sha512/{}/{hash}/{filename}",
                                self.prefix
                            ),
                            status: StoredFileStatus::Ready,
                            modified_at_ns: crate::state::file_modified_at_ns(
                                &metadata,
                            )?
                                as i64,
                            last_used_at: chrono::Utc::now().timestamp(),
                            sources: "[]".to_string(),
                        }));
                    }
                }
                self.entries = None;
            }
            let Some(prefix) = self.prefixes.next_entry().await? else {
                return Ok(None);
            };
            let name = prefix.file_name().to_string_lossy().into_owned();
            if validate_digest(&name, 2).is_err()
                || !prefix.file_type().await?.is_dir()
            {
                continue;
            }
            self.entries = Some(fs::read_dir(prefix.path()).await?);
            self.prefix = name;
        }
    }
}
