use super::{
    Binding, BlobLease, CatalogBlob, ContentStore, FileContent,
    MaterializationKind, catalog, hash_file, input, normalize, relative_link,
    sync_directory, writable_copy,
};
use crate::state::instances::adapters::sqlite::content_rows;
use crate::state::{Instance, InstanceFile};
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, Transaction};
use std::path::Path;
use tokio::fs;

pub(crate) fn content_file_path(file: &InstanceFile) -> String {
    materialized_content_path(&file.relative_path, file.enabled)
}

pub(crate) fn materialized_content_path(
    relative_path: &str,
    enabled: bool,
) -> String {
    let canonical = relative_path.trim_end_matches(".disabled");
    if enabled {
        canonical.to_string()
    } else {
        format!("{canonical}.disabled")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ContentProjectionStatus {
    Healthy,
    Missing,
    Conflict,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Projection {
    relative_path: String,
    sha512: String,
    present: bool,
    mode: MaterializationKind,
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
    pub(crate) async fn inspect_projection(
        &self,
        instance: &Instance,
        file: &InstanceFile,
        binding: &super::Binding,
    ) -> crate::Result<ContentProjectionStatus> {
        let opposite = self
            .instance_path(
                &instance.path,
                &materialized_content_path(&file.relative_path, !file.enabled),
            )
            .await?;
        match fs::symlink_metadata(&opposite).await {
            Ok(_) => return Ok(ContentProjectionStatus::Conflict),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        let path = self
            .instance_path(&instance.path, &content_file_path(file))
            .await?;
        match fs::symlink_metadata(&path).await {
            Ok(_) if self.matches(&path, &binding.blob_sha512).await? => {
                Ok(ContentProjectionStatus::Healthy)
            }
            Ok(_) => Ok(ContentProjectionStatus::Conflict),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(ContentProjectionStatus::Missing)
            }
            Err(error) => Err(error.into()),
        }
    }

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
            if !file.enabled {
                let active_path = self
                    .instance_path(
                        &instance.path,
                        &materialized_content_path(&file.relative_path, true),
                    )
                    .await?;
                match fs::symlink_metadata(&active_path).await {
                    Ok(_) => {
                        return Err(input(format!(
                            "Disabled content {} has an unexpected active file",
                            file.relative_path
                        )));
                    }
                    Err(error)
                        if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error.into()),
                }
                continue;
            }
            let content = self.file_content(&file).await?;
            let projection =
                self.inspect_projection(instance, &file, &binding).await?;
            if !matches!(content, FileContent::Stored(_))
                || projection != ContentProjectionStatus::Healthy
            {
                return Err(input(format!(
                    "{} needs repair or re-import before launching this instance",
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
            let path = self
                .instance_path(&instance.path, &content_file_path(file))
                .await?;
            if fs::symlink_metadata(&path).await.is_ok() {
                if !self.matches(&path, &blob.blob.sha512).await? {
                    return Err(input(
                        "Rollback destination contains different content",
                    ));
                }
                remove_projection(&path).await?;
            }
            let mode = self
                .materialize(
                    &blob,
                    &path,
                    binding.materialization_kind == MaterializationKind::Copy,
                )
                .await?;
            restored.push((&binding.file_id, &binding.blob_sha512, mode));
        }
        let mut tx = self.pool.begin().await?;
        for (id, blob, mode) in restored {
            catalog::bind(&mut tx, id, blob, mode).await?;
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
        known_source_blob: Option<&BlobLease>,
    ) -> crate::Result<PreparedProjection> {
        self.recover(Some(&instance.id)).await?;
        if !super::eligible(relative_path) {
            return Err(input("Unsupported managed content path"));
        }
        let canonical_path = relative_path.trim_end_matches(".disabled");
        let requested_source = legacy_path.unwrap_or(canonical_path);
        let mut existing = content_rows::get_instance_file_by_relative_path(
            &instance.id,
            requested_source,
            &self.pool,
        )
        .await?;
        if existing.is_none() && requested_source != canonical_path {
            existing = content_rows::get_instance_file_by_relative_path(
                &instance.id,
                canonical_path,
                &self.pool,
            )
            .await?;
        }
        let source_relative = existing
            .as_ref()
            .map(content_file_path)
            .unwrap_or_else(|| requested_source.to_string());
        let target_relative =
            materialized_content_path(canonical_path, enabled);
        let source =
            self.instance_path(&instance.path, &source_relative).await?;
        let target =
            self.instance_path(&instance.path, &target_relative).await?;
        if source != target && fs::symlink_metadata(&target).await.is_ok() {
            return Err(input(format!(
                "Both {source_relative} and {target_relative} exist; resolve the duplicate before continuing"
            )));
        }
        let binding = match &existing {
            Some(file) => catalog::binding(&self.pool, &file.id).await?,
            None => None,
        };
        let metadata = match fs::symlink_metadata(&source).await {
            Ok(metadata) => Some(metadata),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        };
        let mut previous_lease = None;
        let before = if let Some(metadata) = &metadata {
            if metadata.file_type().is_symlink() {
                let binding = binding.as_ref().ok_or_else(|| {
					input(
						"The content symlink is unowned; import it explicitly before changing it",
					)
				})?;
                if !self.matches(&source, &binding.blob_sha512).await? {
                    return Err(input(
                        "The content link was changed outside the app; resolve the conflict first",
                    ));
                }
                self.catalog_blob(&binding.blob_sha512).await?.ok_or_else(
                    || input("Managed content is missing from the catalog"),
                )?;
                Some(Projection {
                    relative_path: source_relative.clone(),
                    sha512: binding.blob_sha512.clone(),
                    present: true,
                    mode: MaterializationKind::Symlink,
                })
            } else if metadata.is_file() {
                let previous = if let Some(known) = known_source_blob {
                    known.clone()
                } else {
                    self.ingest_file(&source).await?
                };
                if let Some(file) = &existing
                    && binding.is_some()
                    && previous.blob.sha1 != file.sha1
                {
                    return Err(input(
                        "The content file was changed outside the app; preserve or re-import it before continuing",
                    ));
                }
                let projection = Projection {
                    relative_path: source_relative.clone(),
                    sha512: previous.blob.sha512.clone(),
                    present: true,
                    mode: MaterializationKind::Copy,
                };
                previous_lease = Some(previous);
                Some(projection)
            } else {
                return Err(input(
                    "A managed content file was replaced by a directory",
                ));
            }
        } else if let Some(binding) = &binding {
            self.catalog_blob(&binding.blob_sha512).await?.ok_or_else(
                || input("Managed content is missing from the catalog"),
            )?;
            Some(Projection {
                relative_path: source_relative.clone(),
                sha512: binding.blob_sha512.clone(),
                present: false,
                mode: binding.materialization_kind,
            })
        } else {
            None
        };
        let after = blob.map(|blob| {
            let mode = before
                .as_ref()
                .filter(|previous| {
                    previous.present
                        && previous.sha512 == blob.blob.sha512
                        && previous.relative_path != target_relative
                })
                .map(|previous| previous.mode)
                .unwrap_or(MaterializationKind::Symlink);
            Projection {
                relative_path: target_relative,
                sha512: blob.blob.sha512.clone(),
                present: true,
                mode,
            }
        });
        let journal = ProjectionJournal {
            id: uuid::Uuid::new_v4().to_string(),
            instance_id: instance.id.clone(),
            instance_path: instance.path.clone(),
            before,
            after,
        };
        self.save_journal(&journal).await?;
        Ok(PreparedProjection {
            journal,
            blob: blob.cloned(),
            _before: previous_lease,
        })
    }

    pub(crate) async fn prepare_move(
        &self,
        instance: &Instance,
        file: &InstanceFile,
        binding: &Binding,
        enabled: bool,
    ) -> crate::Result<PreparedProjection> {
        self.recover(Some(&instance.id)).await?;
        if binding.file_id != file.id || !super::eligible(&file.relative_path) {
            return Err(input("Invalid managed content move"));
        }
        if self.inspect_projection(instance, file, binding).await?
            != ContentProjectionStatus::Healthy
        {
            return Err(input(
                "Content cannot be toggled because its instance path is missing or changed",
            ));
        }
        let source_relative = content_file_path(file);
        let target_relative =
            materialized_content_path(&file.relative_path, enabled);
        let journal = ProjectionJournal {
            id: uuid::Uuid::new_v4().to_string(),
            instance_id: instance.id.clone(),
            instance_path: instance.path.clone(),
            before: Some(Projection {
                relative_path: source_relative,
                sha512: binding.blob_sha512.clone(),
                present: true,
                mode: binding.materialization_kind,
            }),
            after: Some(Projection {
                relative_path: target_relative,
                sha512: binding.blob_sha512.clone(),
                present: true,
                mode: binding.materialization_kind,
            }),
        };
        self.save_journal(&journal).await?;
        Ok(PreparedProjection {
            journal,
            blob: None,
            _before: None,
        })
    }

    async fn save_journal(
        &self,
        journal: &ProjectionJournal,
    ) -> crate::Result<()> {
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
            &journal.instance_id,
            &serde_json::to_string(journal)?,
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    pub(crate) async fn recover(
        &self,
        instance_id: Option<&str>,
    ) -> crate::Result<()> {
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
        if journal_noop(journal) {
            let mut tx = self.pool.begin().await?;
            catalog::finish(&mut tx, &journal.id).await?;
            tx.commit().await?;
            return Ok(());
        }
        if let Some((before, after)) = journal_move(journal) {
            let before_path = self
                .instance_path(&journal.instance_path, &before.relative_path)
                .await?;
            let after_path = self
                .instance_path(&journal.instance_path, &after.relative_path)
                .await?;
            match fs::symlink_metadata(&before_path).await {
                Ok(_) => {
                    if !self.matches(&before_path, &before.sha512).await? {
                        return Err(input(format!(
                            "Cannot recover {}: its contents were changed outside the app",
                            before.relative_path
                        )));
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    if !self.matches(&after_path, &after.sha512).await? {
                        return Err(input(format!(
                            "Cannot recover {}: the toggled content is missing or changed",
                            after.relative_path
                        )));
                    }
                    rename_projection(&after_path, &before_path).await?;
                }
                Err(error) => return Err(error.into()),
            }
            let mut tx = self.pool.begin().await?;
            catalog::finish(&mut tx, &journal.id).await?;
            tx.commit().await?;
            return Ok(());
        }
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
                    match before.mode {
                        MaterializationKind::Symlink => {
                            let blob = self
								.catalog_blob(&before.sha512)
								.await?
								.ok_or_else(|| {
									input(
										"Recovery content is missing from the catalog",
									)
								})?;
                            materialize_catalog_symlink(&blob, &path).await?;
                        }
                        MaterializationKind::Copy => {
                            let blob = self
								.lookup(Some(&before.sha512), None, None)
								.await?
								.ok_or_else(|| {
									input(
										"Recovery content needs repair or re-import",
									)
								})?;
                            self.materialize(&blob, &path, true).await?;
                        }
                    }
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
    ) -> crate::Result<MaterializationKind> {
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
            MaterializationKind::Copy
        } else {
            let source = relative_link(&blob.path, parent);
            #[cfg(unix)]
            let result = fs::symlink(&source, &temporary).await;
            #[cfg(windows)]
            let result = fs::symlink_file(&source, &temporary).await;
            match result {
                Ok(()) => MaterializationKind::Symlink,
                Err(error) if link_unavailable(&error) => {
                    writable_copy(&blob.path, &temporary).await?;
                    MaterializationKind::Copy
                }
                Err(error) => return Err(error.into()),
            }
        };
        if mode == MaterializationKind::Copy {
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
        Ok(mode)
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
        if journal_noop(&self.journal) {
            return Ok(());
        }
        if let Some((before, after)) = journal_move(&self.journal) {
            let source = store
                .instance_path(
                    &self.journal.instance_path,
                    &before.relative_path,
                )
                .await?;
            let target = store
                .instance_path(
                    &self.journal.instance_path,
                    &after.relative_path,
                )
                .await?;
            if !store.matches(&source, &before.sha512).await? {
                return Err(input(
                    "Content changed before the toggle could be applied",
                ));
            }
            if fs::symlink_metadata(&target).await.is_ok() {
                return Err(input(
                    "The content toggle destination already exists",
                ));
            }
            rename_projection(&source, &target).await?;
            return Ok(());
        }
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
            catalog::bind(tx, file_id, &after.sha512, after.mode).await?;
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

fn journal_move(
    journal: &ProjectionJournal,
) -> Option<(&Projection, &Projection)> {
    let before = journal.before.as_ref()?;
    let after = journal.after.as_ref()?;
    (before.present
        && after.present
        && before.sha512 == after.sha512
        && before.mode == after.mode
        && before.relative_path != after.relative_path)
        .then_some((before, after))
}

fn journal_noop(journal: &ProjectionJournal) -> bool {
    matches!(
        (&journal.before, &journal.after),
        (Some(before), Some(after))
            if before.present == after.present
                && before.sha512 == after.sha512
                && before.mode == after.mode
                && before.relative_path == after.relative_path
    )
}

async fn rename_projection(source: &Path, target: &Path) -> crate::Result<()> {
    let metadata = fs::symlink_metadata(source).await?;
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        return Err(input("Refusing to move a directory as content"));
    }
    let parent = target
        .parent()
        .ok_or_else(|| input("Content destination has no parent"))?;
    fs::create_dir_all(parent).await?;
    if fs::symlink_metadata(target).await.is_ok() {
        return Err(input("Content destination already exists"));
    }
    fs::rename(source, target).await?;
    if let Some(source_parent) = source.parent() {
        sync_directory(source_parent).await?;
    }
    if source.parent() != Some(parent) {
        sync_directory(parent).await?;
    }
    Ok(())
}

async fn materialize_catalog_symlink(
    blob: &CatalogBlob,
    target: &Path,
) -> crate::Result<()> {
    let parent = target
        .parent()
        .ok_or_else(|| input("Content destination has no parent"))?;
    fs::create_dir_all(parent).await?;
    if fs::symlink_metadata(target).await.is_ok() {
        return Err(input("Content destination already exists"));
    }
    let temporary =
        parent.join(format!(".modrinth-{}.tmp", uuid::Uuid::new_v4()));
    let source = relative_link(&blob.path, parent);
    #[cfg(unix)]
    let result = fs::symlink(&source, &temporary).await;
    #[cfg(windows)]
    let result = fs::symlink_file(&source, &temporary).await;
    if let Err(error) = result {
        return Err(error.into());
    }
    if let Err(error) = fs::rename(&temporary, target).await {
        let _ = fs::remove_file(&temporary).await;
        return Err(error.into());
    }
    sync_directory(parent).await
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
