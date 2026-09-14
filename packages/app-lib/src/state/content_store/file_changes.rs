use super::file_io::{move_instance_file, remove_instance_file};
use super::{
    ContentStore, FileStorageKind, InstanceFileStatus, InstanceFileStorage,
    StoredFileHandle, catalog, content_file_path, file_path_on_disk, input,
};
use crate::state::instances::adapters::sqlite::content_rows;
use crate::state::{Instance, InstanceFile};
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, Transaction};
use tokio::fs;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct FileState {
    pub(super) relative_path: String,
    pub(super) sha512: String,
    pub(super) present: bool,
    pub(super) storage_kind: Option<FileStorageKind>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct FileChangeJournal {
    pub(super) id: String,
    pub(super) instance_id: String,
    pub(super) instance_path: String,
    pub(super) before: Option<FileState>,
    pub(super) after: Option<FileState>,
}

pub(crate) struct FileChangeRequest<'a> {
    pub relative_path: &'a str,
    pub replacement: Option<&'a StoredFileHandle>,
    pub enabled: bool,
    pub legacy_path: Option<&'a str>,
    pub previous_content: Option<&'a StoredFileHandle>,
}

pub(crate) struct PendingFileChange {
    journal: FileChangeJournal,
    pub(crate) stored_file: Option<StoredFileHandle>,
    _previous_file_lease: Option<StoredFileHandle>,
    /// Set only after a failed apply has cleared its journal and preserved or restored the file.
    pub(crate) safe_to_defer: bool,
}

impl ContentStore {
    /// The caller holds the instance and store locks until this change is committed.
    pub(crate) async fn prepare_file_change(
        &self,
        instance: &Instance,
        request: FileChangeRequest<'_>,
    ) -> crate::Result<PendingFileChange> {
        let FileChangeRequest {
            relative_path,
            replacement: stored_file,
            enabled,
            legacy_path,
            previous_content,
        } = request;
        if !super::is_managed_content_path(relative_path) {
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
        let source_relative = if previous_content.is_some() {
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
        if previous_content.is_some() && metadata.is_none() {
            return Err(input(
                "Legacy content disappeared before it could be adopted",
            ));
        }
        let mut previous_lease = None;
        let before = if let Some(metadata) = &metadata {
            if metadata.file_type().is_symlink() {
                return Err(input(
                    "External symlinks must be imported explicitly before changing content",
                ));
            } else if metadata.is_file() {
                let previous = if let Some(known) = previous_content {
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
                    storage_kind: Some(
                        binding
                            .as_ref()
                            .map_or(FileStorageKind::Copy, |binding| {
                                binding.storage_kind
                            }),
                    ),
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
                storage_kind: Some(binding.storage_kind),
            })
        } else {
            None
        };
        let after = stored_file.map(|stored_file| {
            let storage_kind = before
                .as_ref()
                .filter(|previous| {
                    previous.present
                        && previous.sha512 == stored_file.metadata.sha512
                        && previous.relative_path != target_relative
                })
                .and_then(|previous| previous.storage_kind);
            FileState {
                relative_path: target_relative,
                sha512: stored_file.metadata.sha512.clone(),
                present: true,
                storage_kind,
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
            _previous_file_lease: previous_lease,
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
        if binding.file_id != file.id
            || !super::is_managed_content_path(&file.relative_path)
        {
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
                storage_kind: Some(binding.storage_kind),
            }),
            after: Some(FileState {
                relative_path: target_relative,
                sha512: binding.blob_sha512.clone(),
                present: true,
                storage_kind: Some(binding.storage_kind),
            }),
        };
        self.save_journal(&journal).await?;
        Ok(PendingFileChange {
            journal,
            stored_file: None,
            _previous_file_lease: None,
            safe_to_defer: false,
        })
    }

    pub(super) async fn save_journal(
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

    pub(super) async fn apply_inner(
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
            after.storage_kind = Some(
                store
                    .create_instance_file(
                        stored_file,
                        &path,
                        super::FileStoragePolicy::Shared,
                    )
                    .await?,
            );
        }
        Ok(())
    }

    pub(crate) async fn commit(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        file_id: Option<&str>,
    ) -> crate::Result<()> {
        if let (Some(file_id), Some(after)) = (file_id, &self.journal.after) {
            let storage_kind = after.storage_kind.ok_or_else(|| {
                input("Content storage method was not selected")
            })?;
            catalog::set_file_storage(tx, file_id, &after.sha512, storage_kind)
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

pub(super) fn journal_move(
    journal: &FileChangeJournal,
) -> Option<(&FileState, &FileState)> {
    let before = journal.before.as_ref()?;
    let after = journal.after.as_ref()?;
    (before.present
        && after.present
        && before.sha512 == after.sha512
        && before.storage_kind == after.storage_kind
        && before.relative_path != after.relative_path)
        .then_some((before, after))
}

pub(super) fn journal_noop(journal: &FileChangeJournal) -> bool {
    matches!(
        (&journal.before, &journal.after),
        (Some(before), Some(after))
            if before.present == after.present
                && before.sha512 == after.sha512
                && before.storage_kind == after.storage_kind
                && before.relative_path == after.relative_path
    )
}
