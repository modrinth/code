use crate::State;
use crate::state::content_store::adapters::filesystem;
use crate::state::content_store::adapters::sqlite as catalog;
use crate::state::content_store::{
    ContentStore, FileStorageKind, StoreUsage, StoredFileStatus, input,
};
use std::collections::{HashMap, HashSet};

impl ContentStore {
    pub(crate) async fn retain(
        &self,
        kind: &str,
        owner: &str,
        stored_files: &[String],
    ) -> crate::Result<()> {
        let _lease = self.lease().await;
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        for stored_file in stored_files {
            catalog::retain(&mut tx, kind, owner, stored_file).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub(crate) async fn replace_retained(
        &self,
        kind: &str,
        owner: &str,
        stored_files: &[String],
    ) -> crate::Result<()> {
        let _lease = self.lease().await;
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        catalog::release(&mut tx, kind, owner).await?;
        for stored_file in stored_files {
            catalog::retain(&mut tx, kind, owner, stored_file).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub(crate) async fn release(
        &self,
        kind: &str,
        owner: &str,
    ) -> crate::Result<()> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        catalog::release(&mut tx, kind, owner).await?;
        tx.commit().await?;
        Ok(())
    }
    pub async fn usage(&self, state: &State) -> crate::Result<StoreUsage> {
        let _lease = self.lease().await;
        let _runtime_lease = self.runtime_cache_lock.read().await;
        let runtime =
            crate::state::runtime_cache::RuntimeStorage::read(state).await?;
        let stored_files = catalog::stored_files(&self.pool).await?;
        let referenced_hashes = catalog::referenced_files(&self.pool)
            .await?
            .into_iter()
            .collect::<HashSet<_>>();
        let instance_file_bindings =
            catalog::installed_storage(&self.pool).await?;
        let installed = instance_file_bindings
            .iter()
            .map(|binding| binding.blob_sha512.as_str())
            .collect::<HashSet<_>>();
        let logical_bytes = instance_file_bindings
            .iter()
            .map(|binding| binding.size.max(0) as u64)
            .sum::<u64>();
        let private_copy_bytes = instance_file_bindings
            .iter()
            .filter(|binding| {
                binding.materialization_kind == FileStorageKind::Copy
            })
            .map(|binding| binding.size.max(0) as u64)
            .sum::<u64>();
        let referenced_unique = stored_files
            .iter()
            .filter(|stored_file| {
                installed.contains(stored_file.sha512.as_str())
            })
            .map(|stored_file| stored_file.size as u64)
            .sum::<u64>();
        let mut shared_placements = HashMap::new();
        for binding in &instance_file_bindings {
            if binding.materialization_kind == FileStorageKind::Reflink
                || binding.materialization_kind == FileStorageKind::Hardlink
            {
                *shared_placements
                    .entry(binding.blob_sha512.as_str())
                    .or_insert(0usize) += 1;
            }
        }
        Ok(StoreUsage {
            unique_bytes: stored_files
                .iter()
                .map(|stored_file| stored_file.size as u64)
                .sum::<u64>()
                + runtime.total_bytes(),
            shared_bytes: stored_files
                .iter()
                .filter(|stored_file| {
                    shared_placements
                        .get(stored_file.sha512.as_str())
                        .copied()
                        .unwrap_or(0)
                        > 1
                })
                .map(|stored_file| stored_file.size as u64)
                .sum::<u64>()
                + runtime.shared_bytes(),
            unused_cache_bytes: stored_files
                .iter()
                .filter(|stored_file| {
                    !referenced_hashes.contains(&stored_file.sha512)
                })
                .map(|stored_file| stored_file.size as u64)
                .sum::<u64>()
                + runtime.unused_bytes(),
            estimated_saved_bytes: logical_bytes.saturating_sub(
                referenced_unique.saturating_add(private_copy_bytes),
            ) + runtime.saved_bytes(),
            private_copy_bytes,
            object_count: stored_files.len(),
            damaged_objects: stored_files
                .iter()
                .filter(|stored_file| {
                    stored_file.status == StoredFileStatus::Quarantined
                })
                .count(),
            cache_limit_bytes: self.cache_limit().await?,
        })
    }

    async fn cache_limit(&self) -> crate::Result<u64> {
        match catalog::setting(&self.pool, "store_cache_limit_bytes").await? {
            Some(value) => value.parse().map_err(|_| {
                input("The shared content cache limit is invalid")
            }),
            None => Ok(5_368_709_120),
        }
    }

    pub async fn set_cache_limit(&self, bytes: u64) -> crate::Result<()> {
        if bytes > i64::MAX as u64 {
            return Err(input("The shared content cache limit is too large"));
        }
        catalog::set_setting(
            &self.pool,
            "store_cache_limit_bytes",
            &bytes.to_string(),
        )
        .await
    }

    pub async fn cleanup(
        &self,
        state: &State,
        purge_unused: bool,
    ) -> crate::Result<u64> {
        let _exclusive = self.cleanup_lock.try_write().map_err(|_| input("The shared store is busy; try cleanup again after content operations finish"))?;
        if catalog::setting(&self.pool, "store_layout_version")
            .await?
            .as_deref()
            != Some("1")
        {
            return Err(input(
                "Finish shared-store migration before cleaning its cache",
            ));
        }
        let _runtime_exclusive = self.runtime_cache_lock.try_write()
			.map_err(|_| input("Game or Java files are in use; try cleanup again after the operation finishes"))?;
        let runtime =
            crate::state::runtime_cache::RuntimeStorage::read(state).await?;
        let candidates = catalog::cleanup_candidates(&self.pool).await?;
        let mut unused = candidates
            .iter()
            .map(|stored_file| stored_file.size as u64)
            .sum::<u64>()
            + runtime.unused_bytes();
        let limit = if purge_unused {
            0
        } else {
            self.cache_limit().await?
        };
        let mut reclaimed = 0;
        for stored_file in candidates {
            if unused <= limit
                && stored_file.status != StoredFileStatus::Deleting
            {
                continue;
            }
            if !purge_unused
                && chrono::Utc::now()
                    .timestamp()
                    .saturating_sub(stored_file.last_used_at)
                    < 60
            {
                continue;
            }
            catalog::set_file_status(
                &self.pool,
                &stored_file.sha512,
                StoredFileStatus::Deleting,
            )
            .await?;
            let path = self.path(&stored_file)?;
            self.validate_object_parent(&path).await?;
            filesystem::remove_unused_file(&path).await?;
            catalog::delete_file(&self.pool, &stored_file.sha512).await?;
            unused = unused.saturating_sub(stored_file.size as u64);
            reclaimed += stored_file.size as u64;
        }
        if self.any_instance_running(state).await? {
            return Ok(reclaimed);
        }
        for job in crate::install::store::list(false, state).await? {
            if matches!(
                job.status,
                crate::install::InstallJobStatus::Queued
                    | crate::install::InstallJobStatus::Running
            ) {
                return Ok(reclaimed);
            }
            if let Some(backup) = job.state.paths.staging_dir
                && filesystem::path_exists(&backup).await?
            {
                return Ok(reclaimed);
            }
        }
        let mut unused_runtime = runtime
            .files
            .into_iter()
            .filter(|file| file.references == 0)
            .collect::<Vec<_>>();
        unused_runtime.sort_by_key(|file| file.last_used_at);
        for file in unused_runtime {
            if !purge_unused
                && (unused <= limit
                    || chrono::Utc::now()
                        .timestamp()
                        .saturating_sub(file.last_used_at)
                        < 60)
            {
                continue;
            }
            let removed =
                crate::state::runtime_cache::remove_file(&file, &runtime.root)
                    .await?;
            unused = unused.saturating_sub(removed);
            reclaimed += removed;
        }
        Ok(reclaimed)
    }
}
