use crate::state::content_store::InstanceFileStatus;
use crate::state::content_store::adapters::filesystem;
use crate::state::content_store::adapters::filesystem::symlink_metadata_if_exists;
use crate::state::content_store::adapters::filesystem::{
    move_instance_file, remove_instance_file,
};
use crate::state::content_store::adapters::sqlite as catalog;
use crate::state::content_store::adapters::stored_objects::UnregisteredFiles;
use crate::state::content_store::domain::{journal_move, journal_noop};
use crate::state::content_store::model::FileChangeJournal;
use crate::state::content_store::{
    ContentStore, FileStorageKind, FileStoragePolicy, input,
};
use crate::state::instances as content_rows;

impl ContentStore {
    pub(crate) async fn recover(
        &self,
        instance_id: Option<&str>,
    ) -> crate::Result<()> {
        for payload in
            catalog::pending_file_changes(&self.pool, instance_id).await?
        {
            let journal: FileChangeJournal =
                match catalog::decode_journal(&payload) {
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

    pub(in crate::state::content_store) async fn finish_journal(
        &self,
        journal: &FileChangeJournal,
    ) -> crate::Result<()> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        catalog::finish_file_change(&mut tx, &journal.id).await?;
        tx.commit().await?;
        Ok(())
    }

    pub(in crate::state::content_store) async fn rollback_journal(
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
                && symlink_metadata_if_exists(&path)
                    .await?
                    .is_some_and(|metadata| metadata.is_file())
            {
                if let Some(stored_file) =
                    self.lookup(Some(&before.sha512), None).await?
                {
                    let shared = filesystem::matches_any_file(
                        &path,
                        std::slice::from_ref(&stored_file.path),
                    )
                    .await?;
                    if shared {
                        let temporary = path.with_file_name(format!(
                            ".modrinth-rollback-{}.tmp",
                            uuid::Uuid::new_v4()
                        ));
                        self.create_instance_file(
                            &stored_file,
                            &temporary,
                            FileStoragePolicy::Independent,
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
            match self
                .check_instance_path(&before_path, &before.sha512)
                .await?
            {
                InstanceFileStatus::Healthy => {}
                InstanceFileStatus::Conflict => {
                    return Err(input(format!(
                        "Cannot recover {}: its contents were changed outside the app",
                        before.relative_path
                    )));
                }
                InstanceFileStatus::Missing => {
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
            }
            return self.finish_journal(journal).await;
        }
        if let Some(after) = &journal.after {
            let path = self
                .instance_path(&journal.instance_path, &after.relative_path)
                .await?;
            let content = self.inspect_instance_path(&path).await?;
            match content.status(&after.sha512) {
                InstanceFileStatus::Healthy => {
                    remove_instance_file(&path).await?
                }
                InstanceFileStatus::Missing => {}
                InstanceFileStatus::Conflict => {
                    if matches!(&journal.before, Some(before) if before.relative_path == after.relative_path && content.status(&before.sha512) == InstanceFileStatus::Healthy)
                    {
                        return self.finish_journal(journal).await;
                    }
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
                let status =
                    self.check_instance_path(&path, &before.sha512).await?;
                if status == InstanceFileStatus::Conflict {
                    return Err(input(
                        "Recovery would overwrite externally changed content",
                    ));
                }
                if status == InstanceFileStatus::Missing {
                    let stored_file = self
                        .lookup(Some(&before.sha512), None)
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
                        let mut tx =
                            self.pool.begin_with("BEGIN IMMEDIATE").await?;
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

    pub(crate) async fn recover_unregistered_files(&self) -> crate::Result<()> {
        let known = catalog::known_hashes(&self.pool).await?;
        let Some(mut files) =
            UnregisteredFiles::open(&self.root, known).await?
        else {
            return Ok(());
        };
        while let Some(file) = files.next_file().await? {
            catalog::save_file(&self.pool, &file).await?;
        }
        Ok(())
    }
    pub(crate) async fn remove_abandoned_staging(&self) -> crate::Result<()> {
        filesystem::remove_abandoned_staging(&self.staging).await
    }
}
