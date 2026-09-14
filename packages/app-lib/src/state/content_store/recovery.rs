use super::file_changes::{FileChangeJournal, journal_move, journal_noop};
use super::file_io::{move_instance_file, remove_instance_file};
use super::{ContentStore, FileStorageKind, StoredFileStatus, catalog, input};
use crate::state::instances::adapters::sqlite::content_rows;
use std::collections::HashSet;
use tokio::fs;

impl ContentStore {
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

    pub(super) async fn finish_journal(
        &self,
        journal: &FileChangeJournal,
    ) -> crate::Result<()> {
        let mut tx = self.pool.begin().await?;
        catalog::finish_file_change(&mut tx, &journal.id).await?;
        tx.commit().await?;
        Ok(())
    }

    pub(super) async fn rollback_journal(
        &self,
        journal: &FileChangeJournal,
    ) -> crate::Result<()> {
        if let (Some(before), Some(after)) = (&journal.before, &journal.after)
            && before.present
            && after.present
            && before.storage_kind == Some(FileStorageKind::Copy)
            && after.storage_kind.is_none()
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
                if let Some(stored_file) =
                    self.lookup(Some(&before.sha512), None, None).await?
                {
                    let original_path = path.clone();
                    let stored_path = stored_file.path.clone();
                    let shared = tokio::task::spawn_blocking(move || {
                        same_file::is_same_file(original_path, stored_path)
                    })
                    .await??;
                    if shared {
                        let temporary = path.with_file_name(format!(
                            ".modrinth-rollback-{}.tmp",
                            uuid::Uuid::new_v4()
                        ));
                        self.create_instance_file(
                            &stored_file,
                            &temporary,
                            super::FileStoragePolicy::Independent,
                        )
                        .await?;
                        remove_instance_file(&path).await?;
                        move_instance_file(&temporary, &path).await?;
                    }
                }
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
                    let stored_file = self
                        .lookup(Some(&before.sha512), None, None)
                        .await?
                        .ok_or_else(|| {
                            input("Recovery content needs repair or re-import")
                        })?;
                    let policy = before.storage_kind
						.ok_or_else(|| input("Recovery content has no previous storage method"))?
						.restore_policy();
                    let storage_kind = self
                        .create_instance_file(&stored_file, &path, policy)
                        .await?;
                    if let Some(file) =
                        content_rows::get_instance_file_by_relative_path(
                            &journal.instance_id,
                            before.relative_path.trim_end_matches(".disabled"),
                            &self.pool,
                        )
                        .await?
                        && let Some(binding) =
                            catalog::file_storage(&self.pool, &file.id).await?
                    {
                        let mut tx = self.pool.begin().await?;
                        catalog::set_file_storage(
                            &mut tx,
                            &file.id,
                            &binding.blob_sha512,
                            storage_kind,
                        )
                        .await?;
                        catalog::finish_file_change(&mut tx, &journal.id)
                            .await?;
                        tx.commit().await?;
                        return Ok(());
                    }
                }
            }
        }
        self.finish_journal(journal).await
    }

    pub(crate) async fn remove_abandoned_staging(&self) -> crate::Result<()> {
        let mut entries = fs::read_dir(&self.staging).await?;
        while let Some(entry) = entries.next_entry().await? {
            let name = entry.file_name();
            if name.to_string_lossy().starts_with(".tmp")
                && entry.file_type().await?.is_file()
            {
                fs::remove_file(entry.path()).await?;
            }
        }
        Ok(())
    }

    /// Registers payloads published before a crash interrupted their catalog write.
    pub(crate) async fn recover_unregistered_files(&self) -> crate::Result<()> {
        let known = catalog::stored_files(&self.pool)
            .await?
            .into_iter()
            .map(|stored_file| stored_file.sha512)
            .collect::<HashSet<_>>();
        let objects = self.root.join("objects/sha512");
        self.validate_object_parent(&objects.join("payload.jar"))
            .await?;
        let mut prefixes = match fs::read_dir(&objects).await {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(());
            }
            Err(error) => return Err(error.into()),
        };
        while let Some(prefix) = prefixes.next_entry().await? {
            let name = prefix.file_name().to_string_lossy().into_owned();
            if super::validate_digest(&name, 2).is_err()
                || !prefix.file_type().await?.is_dir()
            {
                continue;
            }
            let mut entries = fs::read_dir(prefix.path()).await?;
            while let Some(entry) = entries.next_entry().await? {
                let hash = entry.file_name().to_string_lossy().into_owned();
                if super::validate_digest(&hash, 128).is_err()
                    || !hash.starts_with(&name)
                    || known.contains(&hash)
                    || !entry.file_type().await?.is_dir()
                {
                    continue;
                }
                for filename in ["payload.jar", "payload.bin"] {
                    let path = entry.path().join(filename);
                    let metadata = match fs::symlink_metadata(&path).await {
                        Ok(metadata) if metadata.is_file() => metadata,
                        Ok(_) => continue,
                        Err(error)
                            if error.kind() == std::io::ErrorKind::NotFound =>
                        {
                            continue;
                        }
                        Err(error) => return Err(error.into()),
                    };
                    let super::FileHashes { sha512, sha1, size } =
                        super::hash_file(&path).await?;
                    if sha512 != hash {
                        tracing::warn!(path = %path.display(), "Preserving an unregistered store object with an unexpected hash");
                        continue;
                    }
                    let mut permissions = metadata.permissions();
                    permissions.set_readonly(true);
                    fs::set_permissions(&path, permissions).await?;
                    catalog::save_file(
                        &self.pool,
                        &super::StoredFileMetadata {
                            sha512,
                            sha1,
                            size: size.try_into().map_err(|_| {
                                input("Content file is too large")
                            })?,
                            relative_path: format!(
                                "objects/sha512/{name}/{hash}/{filename}"
                            ),
                            status: StoredFileStatus::Ready,
                            modified_at_ns: crate::state::file_modified_at_ns(
                                &metadata,
                            )?
                                as i64,
                            last_used_at: chrono::Utc::now().timestamp(),
                            sources: "[]".to_string(),
                        },
                    )
                    .await?;
                    break;
                }
            }
        }
        Ok(())
    }
}
