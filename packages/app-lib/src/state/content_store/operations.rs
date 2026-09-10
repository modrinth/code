use super::{
    BlobLease, ContentStore, catalog, hash_file, input, normalize,
    relative_link, sync_directory, writable_copy,
};
use crate::state::instances::adapters::sqlite::content_rows;
use crate::state::{Instance, InstanceFile};
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, Transaction};
use std::path::Path;
use tokio::fs;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Projection {
    relative_path: String,
    sha512: String,
    present: bool,
    mode: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ProjectionJournal {
    id: String,
    instance_id: String,
    instance_path: String,
    before: Option<Projection>,
    after: Option<Projection>,
}

pub(crate) struct PreparedProjection {
    journal: ProjectionJournal,
    pub(crate) blob: Option<BlobLease>,
    _before: Option<BlobLease>,
}

impl ContentStore {
    pub(crate) async fn validate_instance(
        &self,
        instance: &Instance,
    ) -> crate::Result<()> {
        if catalog::setting(&self.pool, "store_layout_version")
            .await?
            .as_deref()
            != Some("1")
        {
            return Err(input(
                "Finish shared-store migration before launching Minecraft",
            ));
        }
        for file in
            content_rows::get_instance_files(&instance.id, &self.pool).await?
        {
            let Some(binding) = catalog::binding(&self.pool, &file.id).await?
            else {
                continue;
            };
            let path = self
                .instance_path(&instance.path, &file.relative_path)
                .await?;
            if file.enabled {
                if self.file_blob(&file).await?.is_none()
                    || !self.matches(&path, &binding.blob_sha512).await?
                {
                    return Err(input(format!(
                        "{} needs repair or re-import before launching this instance",
                        file.relative_path
                    )));
                }
            } else if fs::symlink_metadata(&path).await.is_ok() {
                return Err(input(format!(
                    "Disabled content {} still has a file at its reserved path; resolve it before launching",
                    file.relative_path
                )));
            }
        }
        Ok(())
    }

    pub(crate) async fn restore_bindings(
        &self,
        instance: &Instance,
        files: &[InstanceFile],
        bindings: &[super::Binding],
    ) -> crate::Result<()> {
        let _lease = self.lease().await;
        let mut restored = Vec::new();
        for binding in bindings {
            let file = files
                .iter()
                .find(|file| file.id == binding.file_id)
                .ok_or_else(|| {
                    input("Rollback content reference has no file record")
                })?;
            let blob = self
                .lookup(Some(&binding.blob_sha512), None, Some(file.size))
                .await?
                .ok_or_else(|| {
                    input("Rollback content needs repair or re-import")
                })?;
            let mode = if file.enabled {
                let path = self
                    .instance_path(&instance.path, &file.relative_path)
                    .await?;
                if fs::symlink_metadata(&path).await.is_ok() {
                    if !self.matches(&path, &blob.blob.sha512).await? {
                        return Err(input(
                            "Rollback destination contains different content",
                        ));
                    }
                    remove_projection(&path).await?;
                }
                self.materialize(&blob, &path, false).await?
            } else {
                binding.materialization_kind.clone()
            };
            restored.push((&binding.file_id, &binding.blob_sha512, mode));
        }
        let mut tx = self.pool.begin().await?;
        for (id, blob, mode) in restored {
            catalog::bind(&mut tx, id, blob, &mode).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// The caller holds the instance content lock until the SQL transaction commits.
    pub(crate) async fn prepare(
        &self,
        instance: &Instance,
        relative_path: &str,
        blob: Option<&BlobLease>,
        enabled: bool,
        legacy_path: Option<&str>,
    ) -> crate::Result<PreparedProjection> {
        self.recover(Some(&instance.id)).await?;
        if !super::eligible(relative_path) {
            return Err(input("Unsupported managed content path"));
        }
        let source_relative = legacy_path.unwrap_or(relative_path);
        let source =
            self.instance_path(&instance.path, source_relative).await?;
        let target = self.instance_path(&instance.path, relative_path).await?;
        if source != target && fs::symlink_metadata(&target).await.is_ok() {
            return Err(input(format!(
                "Both {source_relative} and {relative_path} exist; resolve the duplicate before continuing"
            )));
        }
        let existing = content_rows::get_instance_file_by_relative_path(
            &instance.id,
            source_relative,
            &self.pool,
        )
        .await?;
        let binding = match &existing {
            Some(file) => catalog::binding(&self.pool, &file.id).await?,
            None => None,
        };
        let metadata = match fs::symlink_metadata(&source).await {
            Ok(metadata) => Some(metadata),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        };
        let previous_blob = if let Some(metadata) = &metadata {
            if metadata.file_type().is_symlink() {
                let binding = binding.as_ref().ok_or_else(|| input("The content symlink is unowned; import it explicitly before changing it"))?;
                if !self.matches(&source, &binding.blob_sha512).await? {
                    return Err(input(
                        "The content link was changed outside the app; resolve the conflict first",
                    ));
                }
                self.catalog_blob(&binding.blob_sha512).await?
            } else if metadata.is_file() {
                let previous = self.ingest_file(&source).await?;
                if let Some(file) = &existing
                    && binding.is_some()
                    && previous.blob.sha1 != file.sha1
                {
                    return Err(input(
                        "The content file was changed outside the app; preserve or re-import it before continuing",
                    ));
                }
                Some(previous)
            } else {
                return Err(input(
                    "A managed content file was replaced by a directory",
                ));
            }
        } else if let Some(binding) = &binding {
            self.catalog_blob(&binding.blob_sha512).await?
        } else {
            None
        };
        let before = previous_blob.as_ref().map(|blob| Projection {
            relative_path: source_relative.to_string(),
            sha512: blob.blob.sha512.clone(),
            present: metadata.is_some(),
            mode: if metadata
                .as_ref()
                .is_some_and(|metadata| metadata.file_type().is_symlink())
            {
                "symlink"
            } else {
                "copy"
            }
            .to_string(),
        });
        let after = blob.map(|blob| Projection {
            relative_path: relative_path.to_string(),
            sha512: blob.blob.sha512.clone(),
            present: enabled,
            mode: "symlink".to_string(),
        });
        let journal = ProjectionJournal {
            id: uuid::Uuid::new_v4().to_string(),
            instance_id: instance.id.clone(),
            instance_path: instance.path.clone(),
            before,
            after,
        };
        let mut tx = self.pool.begin().await?;
        for projection in
            [&journal.before, &journal.after].into_iter().flatten()
        {
            catalog::retain(
                &mut tx,
                "operation",
                &journal.id,
                &projection.sha512,
            )
            .await?;
        }
        catalog::operation(
            &mut tx,
            &journal.id,
            &instance.id,
            &serde_json::to_string(&journal)?,
        )
        .await?;
        tx.commit().await?;
        Ok(PreparedProjection {
            journal,
            blob: blob.cloned(),
            _before: previous_blob,
        })
    }

    pub(crate) async fn recover(
        &self,
        instance_id: Option<&str>,
    ) -> crate::Result<()> {
        let _lease = self.lease().await;
        for payload in catalog::operations(&self.pool, instance_id).await? {
            let journal: ProjectionJournal = serde_json::from_str(&payload)?;
            self.rollback_journal(&journal).await?;
        }
        Ok(())
    }

    async fn rollback_journal(
        &self,
        journal: &ProjectionJournal,
    ) -> crate::Result<()> {
        if let Some(after) = &journal.after {
            let path = self
                .instance_path(&journal.instance_path, &after.relative_path)
                .await?;
            if fs::symlink_metadata(&path).await.is_ok() {
                if self.matches(&path, &after.sha512).await? {
                    remove_projection(&path).await?;
                } else if !matches!(&journal.before, Some(before) if before.relative_path == after.relative_path && self.matches(&path, &before.sha512).await?)
                {
                    return Err(input(format!(
                        "Cannot recover {}: its contents were changed outside the app",
                        after.relative_path
                    )));
                }
            }
        }
        if let Some(before) = &journal.before {
            let path = self
                .instance_path(&journal.instance_path, &before.relative_path)
                .await?;
            if before.present {
                if fs::symlink_metadata(&path).await.is_ok() {
                    if !self.matches(&path, &before.sha512).await? {
                        return Err(input(
                            "Recovery would overwrite externally changed content",
                        ));
                    }
                } else {
                    let blob = if before.mode == "symlink" { self.catalog_blob(&before.sha512).await? } else { self.lookup(Some(&before.sha512), None, None).await? }.ok_or_else(|| input("Recovery content is missing; re-import or repair it before continuing"))?;
                    self.materialize(&blob, &path, before.mode == "copy")
                        .await?;
                }
            }
        }
        let mut tx = self.pool.begin().await?;
        catalog::finish(&mut tx, &journal.id).await?;
        tx.commit().await?;
        Ok(())
    }

    pub(crate) async fn matches(
        &self,
        path: &Path,
        sha512: &str,
    ) -> crate::Result<bool> {
        let metadata = match fs::symlink_metadata(path).await {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(false);
            }
            Err(error) => return Err(error.into()),
        };
        if metadata.file_type().is_symlink() {
            let rows = catalog::find(&self.pool, Some(sha512), None).await?;
            let Some(blob) = rows.first() else {
                return Ok(false);
            };
            let link = fs::read_link(path).await?;
            let resolved = normalize(
                &path
                    .parent()
                    .ok_or_else(|| input("Invalid link path"))?
                    .join(link),
            );
            return Ok(resolved == self.path(blob)?);
        }
        Ok(metadata.is_file() && hash_file(path).await?.0 == sha512)
    }

    pub(crate) async fn materialize(
        &self,
        blob: &BlobLease,
        target: &Path,
        force_copy: bool,
    ) -> crate::Result<String> {
        let parent = target
            .parent()
            .ok_or_else(|| input("Content destination has no parent"))?;
        fs::create_dir_all(parent).await?;
        if fs::symlink_metadata(target).await.is_ok() {
            return Err(input("Content destination already exists"));
        }
        let temporary =
            parent.join(format!(".modrinth-{}.tmp", uuid::Uuid::new_v4()));
        let mode = if force_copy {
            writable_copy(&blob.path, &temporary).await?;
            "copy"
        } else {
            let source = relative_link(&blob.path, parent);
            #[cfg(unix)]
            let result = fs::symlink(&source, &temporary).await;
            #[cfg(windows)]
            let result = fs::symlink_file(&source, &temporary).await;
            match result {
                Ok(()) => "symlink",
                Err(error) if link_unavailable(&error) => {
                    writable_copy(&blob.path, &temporary).await?;
                    "copy"
                }
                Err(error) => return Err(error.into()),
            }
        };
        if mode == "copy" {
            fs::File::options()
                .write(true)
                .open(&temporary)
                .await?
                .sync_all()
                .await?;
        }
        if let Err(error) = fs::rename(&temporary, target).await {
            let _ = fs::remove_file(&temporary).await;
            return Err(error.into());
        }
        sync_directory(parent).await?;
        Ok(mode.to_string())
    }

    pub(crate) async fn unlink_owned(
        &self,
        file: &InstanceFile,
        instance_path: &str,
    ) -> crate::Result<()> {
        let binding = catalog::binding(&self.pool, &file.id)
            .await?
            .ok_or_else(|| {
                input("Content has not been adopted into the store")
            })?;
        let path = self
            .instance_path(instance_path, &file.relative_path)
            .await?;
        if fs::symlink_metadata(&path).await.is_ok() {
            if !self.matches(&path, &binding.blob_sha512).await? {
                return Err(input(
                    "The content path was changed outside the app",
                ));
            }
            remove_projection(&path).await?;
        }
        Ok(())
    }
}

impl PreparedProjection {
    pub(crate) async fn apply(
        &mut self,
        store: &ContentStore,
    ) -> crate::Result<()> {
        let result = self.apply_inner(store).await;
        if result.is_err() {
            store.rollback_journal(&self.journal).await?;
        }
        result
    }

    async fn apply_inner(&mut self, store: &ContentStore) -> crate::Result<()> {
        if let Some(before) = &self.journal.before {
            let path = store
                .instance_path(
                    &self.journal.instance_path,
                    &before.relative_path,
                )
                .await?;
            if fs::symlink_metadata(&path).await.is_ok() {
                if !store.matches(&path, &before.sha512).await? {
                    return Err(input(
                        "Content changed before the operation could be applied",
                    ));
                }
                remove_projection(&path).await?;
            }
        }
        if let Some(after) = &mut self.journal.after
            && after.present
        {
            let blob = self
                .blob
                .as_ref()
                .ok_or_else(|| input("Missing content lease"))?;
            let path = store
                .instance_path(
                    &self.journal.instance_path,
                    &after.relative_path,
                )
                .await?;
            after.mode = store.materialize(blob, &path, false).await?;
        }
        Ok(())
    }

    pub(crate) async fn commit(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        file_id: Option<&str>,
    ) -> crate::Result<()> {
        if let (Some(file_id), Some(after)) = (file_id, &self.journal.after) {
            catalog::bind(tx, file_id, &after.sha512, &after.mode).await?;
        }
        catalog::finish(tx, &self.journal.id).await
    }

    pub(crate) async fn rollback(
        &self,
        store: &ContentStore,
    ) -> crate::Result<()> {
        store.rollback_journal(&self.journal).await
    }
}

pub(super) fn link_unavailable(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::Unsupported
    ) || cfg!(windows) && matches!(error.raw_os_error(), Some(1 | 50 | 1314))
}

pub(crate) async fn remove_projection(path: &Path) -> crate::Result<()> {
    let metadata = fs::symlink_metadata(path).await?;
    #[cfg(windows)]
    if metadata.is_file() && metadata.permissions().readonly() {
        let mut permissions = metadata.permissions();
        permissions.set_readonly(false);
        fs::set_permissions(path, permissions).await?;
    }
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        return Err(input("Refusing to remove a directory as content"));
    }
    fs::remove_file(path).await?;
    if let Some(parent) = path.parent() {
        sync_directory(parent).await?;
    }
    Ok(())
}
