use super::content_mutation::{
    ContentChange, ContentChangeResult, InstanceContent,
};
use super::sync_content_files::instance_file_id;
use super::sync_content_files::normalize_legacy_content_files;
use crate::state::content_store::file_path_on_disk;
use crate::state::instances::adapters::{filesystem, sqlite};
use crate::state::{InstanceFile, InstanceInstallStage, State};
use chrono::Utc;
use std::collections::{HashMap, HashSet};

/// Moves existing mods and packs into shared storage while preserving their enabled state.
///
/// Background migration pauses between files when Play needs to prepare an instance.
/// Running instances are skipped so migration does not replace files Minecraft is using.
pub(crate) async fn migrate_legacy_content(
    instance_id: &str,
    state: &State,
    background: bool,
) -> crate::Result<()> {
    let (scanned, mut changed) = {
        let _turn = if background {
            Some(state.content_store.legacy_migration_priority.read().await)
        } else {
            None
        };
        let instance_content =
            InstanceContent::lock(instance_id, state).await?;
        let instance = instance_content.instance();
        if instance.install_stage != InstanceInstallStage::Installed
            || crate::state::instance_has_running_process(instance_id, state)
                .await?
            || sqlite::instance_rows::is_instance_quarantined(
                instance_id,
                &state.pool,
            )
            .await?
        {
            return Ok(());
        }
        let existing =
            sqlite::content_rows::get_instance_files(instance_id, &state.pool)
                .await?;
        let bindings = crate::state::content_store::instance_storage(
            &state.pool,
            instance_id,
        )
        .await?
        .into_iter()
        .map(|binding| (binding.file_id.clone(), binding))
        .collect::<HashMap<_, _>>();
        let mut scanned = filesystem::scan_content_files(
            &state.directories.instances_dir(),
            &instance.path,
        )?;
        let normalized = normalize_legacy_content_files(
            instance, &existing, &bindings, &scanned, state,
        )
        .await?;
        let managed_paths = existing
            .iter()
            .filter(|file| bindings.contains_key(&file.id))
            .map(|file| file.relative_path.trim_end_matches(".disabled"))
            .collect::<HashSet<_>>();
        let mut canonical_paths = HashSet::new();
        let mut duplicates = HashSet::new();
        for file in &scanned {
            let canonical = file.relative_path.trim_end_matches(".disabled");
            if !canonical_paths.insert(canonical.to_string()) {
                duplicates.insert(canonical.to_string());
            }
        }
        scanned.retain(|file| {
            let canonical = file.relative_path.trim_end_matches(".disabled");
            !file.is_symlink
                && !file.has_linked_parent
                && crate::state::content_store::is_managed_content_path(
                    &file.relative_path,
                )
                && !duplicates.contains(canonical)
                && !managed_paths.contains(canonical)
        });
        (scanned, normalized)
    };
    for scanned in scanned {
        let _turn = if background {
            Some(state.content_store.legacy_migration_priority.read().await)
        } else {
            None
        };
        let instance_content =
            InstanceContent::lock(instance_id, state).await?;
        let instance = instance_content.instance();
        if instance.install_stage != InstanceInstallStage::Installed
            || crate::state::instance_has_running_process(instance_id, state)
                .await?
            || sqlite::instance_rows::is_instance_quarantined(
                instance_id,
                &state.pool,
            )
            .await?
        {
            break;
        }
        let canonical = scanned.relative_path.trim_end_matches(".disabled");
        let previous =
            sqlite::content_rows::get_instance_file_by_relative_path(
                instance_id,
                canonical,
                &state.pool,
            )
            .await?;
        if let Some(previous) = &previous
            && crate::state::content_store::file_storage(
                &state.pool,
                &previous.id,
            )
            .await?
            .is_some()
        {
            continue;
        }
        let inactive = file_path_on_disk(canonical, !scanned.enabled);
        let inactive = state
            .content_store
            .instance_path(&instance.path, &inactive)
            .await?;
        match tokio::fs::symlink_metadata(inactive).await {
            Ok(_) => continue,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        let source = state
            .content_store
            .instance_path(&instance.path, &scanned.relative_path)
            .await?;
        let metadata = match tokio::fs::symlink_metadata(source).await {
            Ok(metadata)
                if metadata.is_file() && !metadata.file_type().is_symlink() =>
            {
                metadata
            }
            Ok(_) => continue,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                continue;
            }
            Err(error) => return Err(error.into()),
        };
        let file = InstanceFile {
            id: previous
                .as_ref()
                .map(|file| file.id.clone())
                .unwrap_or_else(instance_file_id),
            instance_id: instance_id.to_string(),
            relative_path: scanned.relative_path,
            file_name: scanned.file_name,
            enabled: scanned.enabled,
            sha1: String::new(),
            size: metadata.len(),
            missing: false,
            added_at: previous
                .as_ref()
                .map(|file| file.added_at)
                .unwrap_or_else(Utc::now),
            modified_at: Utc::now(),
        };
        match instance_content
            .apply_change(ContentChange::Adopt { file: &file })
            .await?
        {
            ContentChangeResult::File(_) => changed = true,
            ContentChangeResult::Deferred { reason } => tracing::warn!(
                instance_id, path = %file.relative_path, reason,
                "Legacy file adoption deferred",
            ),
            ContentChangeResult::Removed => {
                unreachable!("adoption cannot remove content")
            }
        }
    }
    if changed {
        super::mark_shared_instance_stale(instance_id, &state.pool).await?;
        crate::api::instance::queue_game_locale_index();
        crate::event::emit::emit_instance(
            instance_id,
            crate::event::InstancePayloadType::Synced,
        )
        .await?;
    }
    Ok(())
}
