use super::{
    ContentStore, FileContent, InstanceFileKind, InstanceFileStorage,
    StoredFileHandle, StoredFileRecord, catalog, hash_file, input, normalize,
    relative_link, sync_directory, writable_copy,
};
use crate::state::instances::adapters::sqlite::content_rows;
use crate::state::{Instance, InstanceFile};
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, Transaction};
use std::path::Path;
use tokio::fs;

pub(crate) fn content_file_path(file: &InstanceFile) -> String {
    file_path_on_disk(&file.relative_path, file.enabled)
}

pub(crate) fn file_path_on_disk(relative_path: &str, enabled: bool) -> String {
    let canonical = relative_path.trim_end_matches(".disabled");
    if enabled {
        canonical.to_string()
    } else {
        format!("{canonical}.disabled")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InstanceFileStatus {
    Healthy,
    Missing,
    Conflict,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct FileState {
    relative_path: String,
    sha512: String,
    present: bool,
    mode: InstanceFileKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct FileChangeJournal {
    id: String,
    instance_id: String,
    instance_path: String,
    before: Option<FileState>,
    after: Option<FileState>,
}

pub(crate) struct PendingFileChange {
    journal: FileChangeJournal,
    pub(crate) stored_file: Option<StoredFileHandle>,
    _before: Option<StoredFileHandle>,
    /// Set only after a failed apply has cleared its journal and preserved or restored the file.
    pub(crate) safe_to_defer: bool,
}

impl ContentStore {
    pub(crate) async fn supports_content_links(
        &self,
        source: &Path,
    ) -> crate::Result<bool> {
        let parent = source
            .parent()
            .ok_or_else(|| input("Content destination has no parent"))?;
        let mut supported = self.link_support.lock().await;
        if let Some(supported) = supported.get(parent) {
            return Ok(*supported);
        }
        let temporary =
            parent.join(format!(".modrinth-link-{}.tmp", uuid::Uuid::new_v4()));
        let target = relative_link(source, parent);
        #[cfg(unix)]
        let result = fs::symlink(&target, &temporary).await;
        #[cfg(windows)]
        let result = fs::symlink_file(&target, &temporary).await;
        match result {
            Ok(()) => {
                fs::remove_file(temporary).await?;
                supported.insert(parent.to_path_buf(), true);
                Ok(true)
            }
            Err(error) if link_unavailable(&error) => {
                supported.insert(parent.to_path_buf(), false);
                Ok(false)
            }
            Err(error) => Err(error.into()),
        }
    }

    pub(crate) async fn check_instance_file(
        &self,
        instance: &Instance,
        file: &InstanceFile,
        binding: &super::InstanceFileStorage,
    ) -> crate::Result<InstanceFileStatus> {
        let opposite = self
            .instance_path(
                &instance.path,
                &file_path_on_disk(&file.relative_path, !file.enabled),
            )
            .await?;
        match fs::symlink_metadata(&opposite).await {
            Ok(_) => return Ok(InstanceFileStatus::Conflict),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        let path = self
            .instance_path(&instance.path, &content_file_path(file))
            .await?;
        match fs::symlink_metadata(&path).await {
            Ok(_)
                if self
                    .instance_file_matches(&path, &binding.blob_sha512)
                    .await? =>
            {
                Ok(InstanceFileStatus::Healthy)
            }
            Ok(_) => Ok(InstanceFileStatus::Conflict),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(InstanceFileStatus::Missing)
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
            let Some(binding) =
                catalog::file_storage(&self.pool, &file.id).await?
            else {
                continue;
            };
            if !file.enabled {
                let active_path = self
                    .instance_path(
                        &instance.path,
                        &file_path_on_disk(&file.relative_path, true),
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
            let file_status =
                self.check_instance_file(instance, &file, &binding).await?;
            if !matches!(content, FileContent::Stored { .. })
                || file_status != InstanceFileStatus::Healthy
            {
                return Err(input(format!(
                    "{} needs repair or re-import before launching this instance",
                    file.relative_path
                )));
            }
        }
        Ok(())
    }

    pub(crate) async fn restore_instance_files(
        &self,
        instance: &Instance,
        files: &[InstanceFile],
        bindings: &[super::InstanceFileStorage],
    ) -> crate::Result<()> {
        let mut restored = Vec::new();
        for binding in bindings {
            let file = files
                .iter()
                .find(|file| file.id == binding.file_id)
                .ok_or_else(|| {
                    input("Rollback content reference has no file record")
                })?;
            let stored_file = self
                .lookup(Some(&binding.blob_sha512), None, Some(file.size))
                .await?
                .ok_or_else(|| {
                    input("Rollback content needs repair or re-import")
                })?;
            let path = self
                .instance_path(&instance.path, &content_file_path(file))
                .await?;
            if fs::symlink_metadata(&path).await.is_ok() {
                if !self
                    .instance_file_matches(&path, &stored_file.metadata.sha512)
                    .await?
                {
                    return Err(input(
                        "Rollback destination contains different content",
                    ));
                }
                remove_instance_file(&path).await?;
            }
            let mode = self
                .create_instance_file(
                    &stored_file,
                    &path,
                    binding.materialization_kind == InstanceFileKind::Copy,
                )
                .await?;
            restored.push((&binding.file_id, &binding.blob_sha512, mode));
        }
        let mut tx = self.pool.begin().await?;
        for (id, stored_file, mode) in restored {
            catalog::set_file_storage(&mut tx, id, stored_file, mode).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// The caller has recovered pending changes and holds the instance and store locks until commit.
    pub(crate) async fn prepare_file_change(
        &self,
        instance: &Instance,
        relative_path: &str,
        stored_file: Option<&StoredFileHandle>,
        enabled: bool,
        legacy_path: Option<&str>,
        source_file: Option<&StoredFileHandle>,
    ) -> crate::Result<PendingFileChange> {
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
        let source_relative = if source_file.is_some() {
            requested_source.to_string()
        } else {
            existing
                .as_ref()
                .map(content_file_path)
                .unwrap_or_else(|| requested_source.to_string())
        };
        let target_relative = file_path_on_disk(canonical_path, enabled);
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
            Some(file) => catalog::file_storage(&self.pool, &file.id).await?,
            None => None,
        };
        let metadata = match fs::symlink_metadata(&source).await {
            Ok(metadata) => Some(metadata),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        };
        if source_file.is_some() && metadata.is_none() {
            return Err(input(
                "Legacy content disappeared before it could be adopted",
            ));
        }
        let mut previous_lease = None;
        let before = if let Some(metadata) = &metadata {
            if metadata.file_type().is_symlink() {
                let binding = binding.as_ref().ok_or_else(|| {
					input(
						"The content symlink is unowned; import it explicitly before changing it",
					)
				})?;
                if !self
                    .instance_file_matches(&source, &binding.blob_sha512)
                    .await?
                {
                    return Err(input(
                        "The content link was changed outside the app; resolve the conflict first",
                    ));
                }
                self.get_file_record(&binding.blob_sha512)
                    .await?
                    .ok_or_else(|| {
                        input("Managed content is missing from the catalog")
                    })?;
                Some(FileState {
                    relative_path: source_relative.clone(),
                    sha512: binding.blob_sha512.clone(),
                    present: true,
                    mode: InstanceFileKind::Symlink,
                })
            } else if metadata.is_file() {
                let previous = if let Some(known) = source_file {
                    known.clone()
                } else {
                    self.store_file(&source).await?
                };
                if let Some(file) = &existing
                    && binding.is_some()
                    && previous.metadata.sha1 != file.sha1
                {
                    return Err(input(
                        "The content file was changed outside the app; preserve or re-import it before continuing",
                    ));
                }
                let file_status = FileState {
                    relative_path: source_relative.clone(),
                    sha512: previous.metadata.sha512.clone(),
                    present: true,
                    mode: InstanceFileKind::Copy,
                };
                previous_lease = Some(previous);
                Some(file_status)
            } else {
                return Err(input(
                    "A managed content file was replaced by a directory",
                ));
            }
        } else if let Some(binding) = &binding {
            self.get_file_record(&binding.blob_sha512)
                .await?
                .ok_or_else(|| {
                    input("Managed content is missing from the catalog")
                })?;
            Some(FileState {
                relative_path: source_relative.clone(),
                sha512: binding.blob_sha512.clone(),
                present: false,
                mode: binding.materialization_kind,
            })
        } else {
            None
        };
        let after = stored_file.map(|stored_file| {
            let mode = before
                .as_ref()
                .filter(|previous| {
                    previous.present
                        && previous.sha512 == stored_file.metadata.sha512
                        && previous.relative_path != target_relative
                })
                .map(|previous| previous.mode)
                .unwrap_or(InstanceFileKind::Symlink);
            FileState {
                relative_path: target_relative,
                sha512: stored_file.metadata.sha512.clone(),
                present: true,
                mode,
            }
        });
        let journal = FileChangeJournal {
            id: uuid::Uuid::new_v4().to_string(),
            instance_id: instance.id.clone(),
            instance_path: instance.path.clone(),
            before,
            after,
        };
        self.save_journal(&journal).await?;
        Ok(PendingFileChange {
            journal,
            stored_file: stored_file.cloned(),
            _before: previous_lease,
            safe_to_defer: false,
        })
    }

    /// The caller has recovered pending changes and holds the instance and store locks until commit.
    pub(crate) async fn prepare_file_move(
        &self,
        instance: &Instance,
        file: &InstanceFile,
        binding: &InstanceFileStorage,
        enabled: bool,
    ) -> crate::Result<PendingFileChange> {
        if binding.file_id != file.id || !super::eligible(&file.relative_path) {
            return Err(input("Invalid managed content move"));
        }
        if self.check_instance_file(instance, file, binding).await?
            != InstanceFileStatus::Healthy
        {
            return Err(input(
                "Content cannot be toggled because its instance path is missing or changed",
            ));
        }
        let source_relative = content_file_path(file);
        let target_relative = file_path_on_disk(&file.relative_path, enabled);
        let journal = FileChangeJournal {
            id: uuid::Uuid::new_v4().to_string(),
            instance_id: instance.id.clone(),
            instance_path: instance.path.clone(),
            before: Some(FileState {
                relative_path: source_relative,
                sha512: binding.blob_sha512.clone(),
                present: true,
                mode: binding.materialization_kind,
            }),
            after: Some(FileState {
                relative_path: target_relative,
                sha512: binding.blob_sha512.clone(),
                present: true,
                mode: binding.materialization_kind,
            }),
        };
        self.save_journal(&journal).await?;
        Ok(PendingFileChange {
            journal,
            stored_file: None,
            _before: None,
            safe_to_defer: false,
        })
    }

    async fn save_journal(
        &self,
        journal: &FileChangeJournal,
    ) -> crate::Result<()> {
        let mut tx = self.pool.begin().await?;
        for file_status in
            [&journal.before, &journal.after].into_iter().flatten()
        {
            catalog::retain(
                &mut tx,
                "operation",
                &journal.id,
                &file_status.sha512,
            )
            .await?;
        }
        catalog::save_file_change(
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
        for payload in
            catalog::pending_file_changes(&self.pool, instance_id).await?
        {
            let journal: FileChangeJournal =
                match serde_json::from_str(&payload) {
                    Ok(journal) => journal,
                    Err(error) if instance_id.is_none() => {
                        tracing::warn!(
                            "Invalid content recovery journal retained: {error}"
                        );
                        continue;
                    }
                    Err(error) => return Err(error.into()),
                };
            if let Err(error) = self.rollback_journal(&journal).await {
                if instance_id.is_some() {
                    return Err(error);
                }
                tracing::warn!(
                    instance_id = %journal.instance_id, operation_id = %journal.id,
                    "Content recovery deferred for this instance: {error}",
                );
            }
        }
        Ok(())
    }

    async fn finish_journal(
        &self,
        journal: &FileChangeJournal,
    ) -> crate::Result<()> {
        let mut tx = self.pool.begin().await?;
        catalog::finish_file_change(&mut tx, &journal.id).await?;
        tx.commit().await?;
        Ok(())
    }

    async fn rollback_journal(
        &self,
        journal: &FileChangeJournal,
    ) -> crate::Result<()> {
        if let (Some(before), Some(after)) = (&journal.before, &journal.after)
            && before.present
            && after.present
            && before.mode == InstanceFileKind::Copy
            && after.mode == InstanceFileKind::Symlink
            && before.relative_path == after.relative_path
            && before.sha512 == after.sha512
        {
            let file = content_rows::get_instance_file_by_relative_path(
                &journal.instance_id,
                before.relative_path.trim_end_matches(".disabled"),
                &self.pool,
            )
            .await?;
            let unmanaged = match file {
                Some(file) => {
                    catalog::file_storage(&self.pool, &file.id).await?.is_none()
                }
                None => true,
            };
            let path = self
                .instance_path(&journal.instance_path, &before.relative_path)
                .await?;
            if unmanaged
                && fs::symlink_metadata(&path)
                    .await
                    .is_ok_and(|metadata| metadata.is_file())
            {
                tracing::debug!(
                    instance_id = %journal.instance_id,
                    path = %before.relative_path,
                    "Preserving the legacy file and abandoning its interrupted adoption",
                );
                return self.finish_journal(journal).await;
            }
        }
        if journal_noop(journal) {
            return self.finish_journal(journal).await;
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
                    if !self
                        .instance_file_matches(&before_path, &before.sha512)
                        .await?
                    {
                        return Err(input(format!(
                            "Cannot recover {}: its contents were changed outside the app",
                            before.relative_path
                        )));
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    if !self
                        .instance_file_matches(&after_path, &after.sha512)
                        .await?
                    {
                        return Err(input(format!(
                            "Cannot recover {}: the toggled content is missing or changed",
                            after.relative_path
                        )));
                    }
                    move_instance_file(&after_path, &before_path).await?;
                }
                Err(error) => return Err(error.into()),
            }
            return self.finish_journal(journal).await;
        }
        if let Some(after) = &journal.after {
            let path = self
                .instance_path(&journal.instance_path, &after.relative_path)
                .await?;
            if fs::symlink_metadata(&path).await.is_ok() {
                if self.instance_file_matches(&path, &after.sha512).await? {
                    remove_instance_file(&path).await?;
                } else if !matches!(&journal.before, Some(before) if before.relative_path == after.relative_path && self.instance_file_matches(&path, &before.sha512).await?)
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
                    if !self
                        .instance_file_matches(&path, &before.sha512)
                        .await?
                    {
                        return Err(input(
                            "Recovery would overwrite externally changed content",
                        ));
                    }
                } else {
                    match before.mode {
                        InstanceFileKind::Symlink => {
                            let stored_file = self
								.get_file_record(&before.sha512)
								.await?
								.ok_or_else(|| {
									input(
										"Recovery content is missing from the catalog",
									)
								})?;
                            create_recovery_link(&stored_file, &path).await?;
                        }
                        InstanceFileKind::Copy => {
                            let stored_file = self
								.lookup(Some(&before.sha512), None, None)
								.await?
								.ok_or_else(|| {
									input(
										"Recovery content needs repair or re-import",
									)
								})?;
                            self.create_instance_file(
                                &stored_file,
                                &path,
                                true,
                            )
                            .await?;
                        }
                    }
                }
            }
        }
        self.finish_journal(journal).await
    }

    pub(crate) async fn instance_file_matches(
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
            let rows =
                catalog::find_files(&self.pool, Some(sha512), None).await?;
            let Some(stored_file) = rows.first() else {
                return Ok(false);
            };
            let link = fs::read_link(path).await?;
            let resolved = normalize(
                &path
                    .parent()
                    .ok_or_else(|| input("Invalid link path"))?
                    .join(link),
            );
            return Ok(resolved == self.path(stored_file)?);
        }
        Ok(metadata.is_file() && hash_file(path).await?.0 == sha512)
    }

    pub(crate) async fn create_instance_file(
        &self,
        stored_file: &StoredFileHandle,
        target: &Path,
        force_copy: bool,
    ) -> crate::Result<InstanceFileKind> {
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
            writable_copy(&stored_file.path, &temporary).await?;
            InstanceFileKind::Copy
        } else {
            let source = relative_link(&stored_file.path, parent);
            #[cfg(unix)]
            let result = fs::symlink(&source, &temporary).await;
            #[cfg(windows)]
            let result = fs::symlink_file(&source, &temporary).await;
            match result {
                Ok(()) => InstanceFileKind::Symlink,
                Err(error) if link_unavailable(&error) => {
                    writable_copy(&stored_file.path, &temporary).await?;
                    InstanceFileKind::Copy
                }
                Err(error) => return Err(error.into()),
            }
        };
        if mode == InstanceFileKind::Copy {
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

impl PendingFileChange {
    pub(crate) async fn apply(
        &mut self,
        store: &ContentStore,
    ) -> crate::Result<()> {
        let mut mutation_started = false;
        let result = self.apply_inner(store, &mut mutation_started).await;
        if result.is_err() {
            if mutation_started {
                store.rollback_journal(&self.journal).await?;
            } else {
                store.finish_journal(&self.journal).await?;
            }
            self.safe_to_defer = true;
        }
        result
    }

    async fn apply_inner(
        &mut self,
        store: &ContentStore,
        mutation_started: &mut bool,
    ) -> crate::Result<()> {
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
            if !store.instance_file_matches(&source, &before.sha512).await? {
                return Err(input(
                    "Content changed before the toggle could be applied",
                ));
            }
            if fs::symlink_metadata(&target).await.is_ok() {
                return Err(input(
                    "The content toggle destination already exists",
                ));
            }
            *mutation_started = true;
            move_instance_file(&source, &target).await?;
            return Ok(());
        }
        if let Some(before) = &self.journal.before {
            let path = store
                .instance_path(
                    &self.journal.instance_path,
                    &before.relative_path,
                )
                .await?;
            match fs::symlink_metadata(&path).await {
                Ok(_) => {
                    if !store
                        .instance_file_matches(&path, &before.sha512)
                        .await?
                    {
                        return Err(input(
                            "Content changed before the operation could be applied",
                        ));
                    }
                    *mutation_started = true;
                    remove_instance_file(&path).await?;
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    if before.present {
                        return Err(input(
                            "Content disappeared before the operation could be applied",
                        ));
                    }
                }
                Err(error) => return Err(error.into()),
            }
        }
        if let Some(after) = &mut self.journal.after
            && after.present
        {
            let stored_file = self
                .stored_file
                .as_ref()
                .ok_or_else(|| input("Missing content lease"))?;
            let path = store
                .instance_path(
                    &self.journal.instance_path,
                    &after.relative_path,
                )
                .await?;
            *mutation_started = true;
            after.mode = store
                .create_instance_file(stored_file, &path, false)
                .await?;
        }
        Ok(())
    }

    pub(crate) async fn commit(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        file_id: Option<&str>,
    ) -> crate::Result<()> {
        if let (Some(file_id), Some(after)) = (file_id, &self.journal.after) {
            catalog::set_file_storage(tx, file_id, &after.sha512, after.mode)
                .await?;
        }
        catalog::finish_file_change(tx, &self.journal.id).await
    }

    pub(crate) async fn rollback(
        &self,
        store: &ContentStore,
    ) -> crate::Result<()> {
        store.rollback_journal(&self.journal).await
    }
}

fn journal_move(
    journal: &FileChangeJournal,
) -> Option<(&FileState, &FileState)> {
    let before = journal.before.as_ref()?;
    let after = journal.after.as_ref()?;
    (before.present
        && after.present
        && before.sha512 == after.sha512
        && before.mode == after.mode
        && before.relative_path != after.relative_path)
        .then_some((before, after))
}

fn journal_noop(journal: &FileChangeJournal) -> bool {
    matches!(
        (&journal.before, &journal.after),
        (Some(before), Some(after))
            if before.present == after.present
                && before.sha512 == after.sha512
                && before.mode == after.mode
                && before.relative_path == after.relative_path
    )
}

async fn move_instance_file(source: &Path, target: &Path) -> crate::Result<()> {
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

async fn create_recovery_link(
    stored_file: &StoredFileRecord,
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
    let source = relative_link(&stored_file.path, parent);
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

pub(crate) async fn remove_instance_file(path: &Path) -> crate::Result<()> {
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
