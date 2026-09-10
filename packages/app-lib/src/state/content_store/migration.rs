use super::{
    catalog, eligible, hash_file, input, normalize, relative_link,
    sync_directory,
};
use crate::state::instances::adapters::{
    filesystem,
    sqlite::{content_rows, instance_rows},
};
use crate::state::{Instance, InstanceFile, JavaVersion, State};
use serde_json::Value;
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use tokio::fs;

pub(crate) async fn migrate(state: &State) -> crate::Result<()> {
    let store = &state.content_store;
    if catalog::setting(&state.pool, "store_layout_version")
        .await?
        .as_deref()
        != Some("1")
    {
        let instances = instance_rows::list_instances(&state.pool).await?;
        for instance in &instances {
            if crate::state::instance_has_running_process(&instance.id, state)
                .await?
            {
                return Err(input(
                    "Close running Minecraft instances before migrating the shared store",
                ));
            }
        }
        let root = &state.directories.config_dir;
        let mut mappings = [
            ("meta/java_versions", "store/java"),
            ("meta/install_job_backups", "store/install-backups"),
            ("caches", "store/cache"),
			("meta/game-locales", "store/cache/game-locales"),
            ("icons", "store/icons"),
            ("synced-options", "store/synced-options"),
        ]
        .into_iter()
        .map(|(from, to)| (root.join(from), root.join(to)))
        .collect::<Vec<_>>();
        for directory in [
            "versions",
            "libraries",
            "assets",
            "log_configs",
            "resources",
            "natives",
        ] {
            mappings.push((
                root.join("meta").join(directory),
                state.directories.minecraft_dir().join(directory),
            ));
        }
        for (from, to) in &mappings {
            relocate_tree(from, to).await?;
        }
        relocate_links(&state.directories.store_dir(), &mappings).await?;
        relocate_links(&state.directories.instances_dir(), &mappings).await?;
        rewrite_database_paths(&state.pool, &mappings).await?;
        for (from, to) in &mappings {
            if fs::try_exists(from).await? && fs::try_exists(to).await? {
                remove_migrated_tree(from).await?;
            }
        }
        let legacy_meta = root.join("meta");
        if fs::try_exists(&legacy_meta).await? {
            if fs::read_dir(&legacy_meta)
                .await?
                .next_entry()
                .await?
                .is_none()
            {
                fs::remove_dir(&legacy_meta).await?;
            } else {
                tracing::warn!(path = %legacy_meta.display(), "Preserving unrecognized legacy metadata");
            }
        }
        catalog::set_setting(&state.pool, "store_layout_version", "1").await?;
    }
    for instance in instance_rows::list_instances(&state.pool).await? {
        for binding in catalog::bindings(&state.pool, &instance.id).await? {
            if let Some(file) =
                content_rows::get_instance_files(&instance.id, &state.pool)
                    .await?
                    .into_iter()
                    .find(|file| file.id == binding.file_id)
            {
                let path = store
                    .instance_path(&instance.path, &file.relative_path)
                    .await?;
                if let Ok(metadata) = fs::symlink_metadata(&path).await {
                    let mode = if metadata.file_type().is_symlink() {
                        "symlink"
                    } else {
                        "copy"
                    };
                    if mode != binding.materialization_kind
                        && store.matches(&path, &binding.blob_sha512).await?
                    {
                        let mut tx = state.pool.begin().await?;
                        catalog::bind(
                            &mut tx,
                            &binding.file_id,
                            &binding.blob_sha512,
                            mode,
                        )
                        .await?;
                        tx.commit().await?;
                    }
                }
            }
        }
        if crate::state::instance_has_running_process(&instance.id, state)
            .await?
        {
            continue;
        }
        let key = format!("store_instance_migrated:{}", instance.id);
        if catalog::setting(&state.pool, &key).await?.as_deref() == Some("1") {
            continue;
        }
        let _lock = state.lock_instance_content(&instance.id).await;
        let existing =
            content_rows::get_instance_files(&instance.id, &state.pool).await?;
        let scanned = filesystem::scan_content_files(
            &state.directories.instances_dir(),
            &instance.path,
        )?;
        for scanned in scanned {
            if !eligible(&scanned.relative_path) {
                continue;
            }
            let previous = existing
                .iter()
                .find(|file| file.relative_path == scanned.relative_path);
            if let Some(previous) = previous
                && catalog::binding(&state.pool, &previous.id).await?.is_some()
            {
                continue;
            }
            if scanned.is_symlink {
                tracing::warn!(path = %scanned.relative_path, "Leaving an external content symlink unmanaged");
                continue;
            }
            let path = store
                .instance_path(&instance.path, &scanned.relative_path)
                .await?;
            let blob = store.ingest_file(&path).await?;
            let file = previous.cloned().unwrap_or_else(|| InstanceFile {
                id: format!("instance-file:{}", uuid::Uuid::new_v4()),
                instance_id: instance.id.clone(),
                relative_path: scanned.relative_path.clone(),
                file_name: scanned.file_name.clone(),
                enabled: scanned.enabled,
                sha1: blob.blob.sha1.clone(),
                size: blob.blob.size as u64,
                missing: false,
                added_at: chrono::Utc::now(),
                modified_at: chrono::Utc::now(),
            });
            adopt_file(&instance, &file, state).await?;
        }
        catalog::set_setting(&state.pool, &key, "1").await?;
    }
    crate::api::instance::synced_packs::migrate_store(state).await?;
    for owner in catalog::retained_owners(&state.pool, "synced-cache").await? {
        store.release("synced-cache", &owner).await?;
    }
    for owner in catalog::retained_owners(&state.pool, "rollback").await? {
        if uuid::Uuid::parse_str(&owner).is_ok()
            && !fs::try_exists(
                state.directories.install_backups_dir().join(&owner),
            )
            .await?
        {
            store.release("rollback", &owner).await?;
        }
    }
    store.remove_abandoned_staging().await?;
    Ok(())
}

/// Adopts a regular file while preserving its identity and every content-entry relationship.
pub(crate) async fn adopt_file(
    instance: &Instance,
    file: &InstanceFile,
    state: &State,
) -> crate::Result<InstanceFile> {
    let store = &state.content_store;
    let path = store
        .instance_path(&instance.path, &file.relative_path)
        .await?;
    if fs::symlink_metadata(&path).await?.file_type().is_symlink() {
        return Err(input("External symlinks must be imported explicitly"));
    }
    let blob = store.ingest_file(&path).await?;
    let canonical = file.relative_path.trim_end_matches(".disabled");
    let enabled = file.enabled && canonical == file.relative_path;
    let legacy = (canonical != file.relative_path)
        .then_some(file.relative_path.as_str());
    if legacy.is_some()
        && content_rows::get_instance_file_by_relative_path(
            &instance.id,
            canonical,
            &state.pool,
        )
        .await?
        .is_some_and(|other| other.id != file.id)
    {
        return Err(input(format!(
            "Both enabled and disabled records exist for {canonical}; preserve both files and resolve the duplicate first"
        )));
    }
    let mut operation = store
        .prepare(instance, canonical, Some(&blob), enabled, legacy)
        .await?;
    operation.apply(store).await?;
    let result = async {
        let mut adopted = file.clone();
        adopted.relative_path = canonical.to_string();
        adopted.file_name = Path::new(canonical)
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| input("Invalid content filename"))?
            .to_string();
        adopted.enabled = enabled;
        adopted.sha1 = blob.blob.sha1.clone();
        adopted.size = blob.blob.size as u64;
        adopted.missing = false;
        let mut tx = state.pool.begin().await?;
        if let Some(legacy) = legacy {
            content_rows::rename_instance_file(
                &instance.id,
                legacy,
                canonical,
                &adopted.file_name,
                enabled,
                &mut tx,
            )
            .await?;
        }
        let adopted =
            content_rows::upsert_instance_file(&adopted, &mut tx).await?;
        operation.commit(&mut tx, Some(&adopted.id)).await?;
        if let Some(content_set_id) = &instance.applied_content_set_id {
            content_rows::set_content_entry_enabled_for_file(
                content_set_id,
                &adopted.id,
                enabled,
                &mut tx,
            )
            .await?;
        }
        tx.commit().await?;
        Ok::<_, crate::Error>(adopted)
    }
    .await;
    if result.is_err() {
        operation.rollback(store).await?;
    }
    result
}

async fn relocate_tree(from: &Path, to: &Path) -> crate::Result<()> {
    if !fs::try_exists(from).await? {
        return Ok(());
    }
    if fs::symlink_metadata(from).await?.file_type().is_symlink() {
        return Err(input(
            "A shared-data root must not be a symlink during migration",
        ));
    }
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent).await?;
    }
    if !fs::try_exists(to).await?
        && crate::util::io::is_same_disk(from, to.parent().unwrap_or(to))
            .unwrap_or(false)
    {
        fs::rename(from, to).await?;
        if let Some(parent) = to.parent() {
            sync_directory(parent).await?;
        }
        return Ok(());
    }
    copy_tree(from, to, &[(from.to_path_buf(), to.to_path_buf())]).await
}

pub(crate) async fn copy_tree(
    from: &Path,
    to: &Path,
    mappings: &[(PathBuf, PathBuf)],
) -> crate::Result<()> {
    let mut pending = vec![(from.to_path_buf(), to.to_path_buf())];
    while let Some((source, target)) = pending.pop() {
        let metadata = fs::symlink_metadata(&source).await?;
        if metadata.file_type().is_symlink() {
            let link = fs::read_link(&source).await?;
            let resolved = normalize(
                &source
                    .parent()
                    .ok_or_else(|| input("Invalid symlink"))?
                    .join(&link),
            );
            let mapped = remap(&resolved, mappings);
            let link = relative_link(
                &mapped,
                target
                    .parent()
                    .ok_or_else(|| input("Invalid symlink destination"))?,
            );
            if let Ok(existing) = fs::symlink_metadata(&target).await {
                if existing.file_type().is_symlink()
                    && fs::read_link(&target).await? == link
                {
                    continue;
                }
                if existing.is_file()
                    && !existing.file_type().is_symlink()
                    && hash_file(&resolved).await? == hash_file(&target).await?
                {
                    continue;
                }
                return Err(input(format!(
                    "Migration destination conflicts with {}",
                    target.display()
                )));
            }
            create_link(&link, &target, &resolved).await?;
        } else if metadata.is_dir() {
            if let Ok(existing) = fs::symlink_metadata(&target).await
                && (!existing.is_dir() || existing.file_type().is_symlink())
            {
                return Err(input(
                    "Migration destination contains a directory link or conflicting file",
                ));
            }
            fs::create_dir_all(&target).await?;
            let mut entries = fs::read_dir(&source).await?;
            while let Some(entry) = entries.next_entry().await? {
                pending.push((entry.path(), target.join(entry.file_name())));
            }
        } else if metadata.is_file() {
            if let Ok(existing) = fs::symlink_metadata(&target).await {
                if !existing.is_file()
                    || existing.file_type().is_symlink()
                    || hash_file(&source).await? != hash_file(&target).await?
                {
                    return Err(input(format!(
                        "Migration would overwrite different data at {}",
                        target.display()
                    )));
                }
                continue;
            }
            let parent = target
                .parent()
                .ok_or_else(|| input("Invalid migration target"))?;
            fs::create_dir_all(parent).await?;
            let temporary = parent
                .join(format!(".modrinth-move-{}.tmp", uuid::Uuid::new_v4()));
            super::writable_copy(&source, &temporary).await?;
            fs::File::options()
                .write(true)
                .open(&temporary)
                .await?
                .sync_all()
                .await?;
            fs::set_permissions(&temporary, metadata.permissions()).await?;
            if hash_file(&source).await? != hash_file(&temporary).await? {
                return Err(input(
                    "File changed or failed verification during directory migration",
                ));
            }
            fs::rename(&temporary, &target).await?;
            sync_directory(parent).await?;
        }
    }
    Ok(())
}

async fn create_link(
    link: &Path,
    target: &Path,
    original: &Path,
) -> crate::Result<()> {
    #[cfg(unix)]
    {
        let _ = original;
        fs::symlink(link, target).await?;
    }
    #[cfg(windows)]
    {
        if fs::metadata(original).await?.is_dir() {
            fs::symlink_dir(link, target).await?;
        } else {
            match fs::symlink_file(link, target).await {
                Ok(()) => {}
                Err(error) if super::operations::link_unavailable(&error) => {
                    super::writable_copy(original, target).await?;
                    fs::File::options()
                        .write(true)
                        .open(target)
                        .await?
                        .sync_all()
                        .await?;
                    if hash_file(original).await? != hash_file(target).await? {
                        return Err(input(
                            "Symlink fallback copy changed during migration",
                        ));
                    }
                }
                Err(error) => return Err(error.into()),
            }
        }
    }
    Ok(())
}

pub(crate) async fn relocate_links(
    root: &Path,
    mappings: &[(PathBuf, PathBuf)],
) -> crate::Result<()> {
    if !fs::try_exists(root).await? {
        return Ok(());
    }
    let mut pending = vec![root.to_path_buf()];
    while let Some(path) = pending.pop() {
        let metadata = fs::symlink_metadata(&path).await?;
        if metadata.file_type().is_symlink() {
            let link = fs::read_link(&path).await?;
            let old_path = mappings
                .iter()
                .find_map(|(from, to)| {
                    path.strip_prefix(to).ok().map(|suffix| from.join(suffix))
                })
                .unwrap_or_else(|| path.clone());
            let current_target = normalize(
                &path
                    .parent()
                    .ok_or_else(|| input("Invalid symlink"))?
                    .join(&link),
            );
            if mappings
                .iter()
                .any(|(_, to)| current_target.starts_with(to))
            {
                continue;
            }
            let old_target = normalize(
                &old_path
                    .parent()
                    .ok_or_else(|| input("Invalid symlink"))?
                    .join(&link),
            );
            let target = remap(&old_target, mappings);
            let desired = relative_link(
                &target,
                path.parent().ok_or_else(|| input("Invalid symlink"))?,
            );
            if link != desired {
                let staged = path.with_file_name(format!(
                    ".modrinth-link-{}.tmp",
                    uuid::Uuid::new_v4()
                ));
                create_link(&desired, &staged, &target).await?;
                fs::rename(staged, &path).await?;
            }
        } else if metadata.is_dir() {
            let mut entries = fs::read_dir(&path).await?;
            while let Some(entry) = entries.next_entry().await? {
                pending.push(entry.path());
            }
        }
    }
    Ok(())
}

fn remap(path: &Path, mappings: &[(PathBuf, PathBuf)]) -> PathBuf {
    mappings
        .iter()
        .find_map(|(from, to)| {
            path.strip_prefix(from).ok().map(|suffix| to.join(suffix))
        })
        .unwrap_or_else(|| path.to_path_buf())
}

fn rewrite_value(value: &mut Value, mappings: &[(PathBuf, PathBuf)]) {
    match value {
        Value::String(value) => {
            let path = Path::new(value);
            if path.is_absolute() {
                *value = remap(path, mappings).to_string_lossy().into_owned();
            }
        }
        Value::Array(values) => {
            for value in values {
                rewrite_value(value, mappings);
            }
        }
        Value::Object(values) => {
            for value in values.values_mut() {
                rewrite_value(value, mappings);
            }
        }
        _ => {}
    }
}

async fn rewrite_database_paths(
    pool: &SqlitePool,
    mappings: &[(PathBuf, PathBuf)],
) -> crate::Result<()> {
    for (_, mut java) in JavaVersion::get_all(pool).await? {
        java.path = remap(Path::new(&java.path), mappings)
            .to_string_lossy()
            .into_owned();
        java.upsert(pool).await?;
    }
    for instance in instance_rows::list_instances(pool).await? {
        let mut value = serde_json::to_value(&instance)?;
        rewrite_value(&mut value, mappings);
        let updated: Instance = serde_json::from_value(value)?;
        let overrides =
            instance_rows::get_instance_launch_overrides(&instance.id, pool)
                .await?;
        let mut tx = pool.begin().await?;
        instance_rows::update_instance(&updated, &mut tx).await?;
        if let Some(overrides) = overrides {
            let mut value = serde_json::to_value(&overrides)?;
            rewrite_value(&mut value, mappings);
            instance_rows::upsert_instance_launch_overrides(
                &serde_json::from_value(value)?,
                &mut tx,
            )
            .await?;
        }
        tx.commit().await?;
    }
    let jobs = sqlx::query!(
        "SELECT id, json(state) AS \"state!: String\" FROM install_jobs"
    )
    .fetch_all(pool)
    .await?;
    for job in jobs {
        let mut state: Value = serde_json::from_str(&job.state)?;
        rewrite_value(&mut state, mappings);
        let state = serde_json::to_string(&state)?;
        sqlx::query!(
            "UPDATE install_jobs SET state = jsonb(?) WHERE id = ?",
            state,
            job.id
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub(crate) async fn move_app_directory(
    from: &Path,
    to: &Path,
    pool: &SqlitePool,
) -> crate::Result<()> {
    fs::create_dir_all(to).await?;
    let from = fs::canonicalize(from).await?;
    let to = fs::canonicalize(to).await?;
    if from == to {
        return Ok(());
    }
    if from.starts_with(&to) || to.starts_with(&from) {
        return Err(input(
            "The new app directory cannot contain, or be inside, the old app directory",
        ));
    }
    let processes = sqlx::query!("SELECT pid, start_time FROM processes")
        .fetch_all(pool)
        .await?;
    let system = sysinfo::System::new_all();
    if processes.iter().any(|process| {
        u32::try_from(process.pid)
            .ok()
            .and_then(|pid| system.process(sysinfo::Pid::from_u32(pid)))
            .is_some_and(|running| {
                (running.start_time() as i64).abs_diff(process.start_time) <= 2
            })
    }) {
        return Err(input(
            "Close Minecraft instances before moving the app directory",
        ));
    }
    let directories = [
        "store",
        "profiles",
        "meta",
        "caches",
        "icons",
        "synced-options",
    ];
    let mappings = directories
        .iter()
        .map(|directory| (from.join(directory), to.join(directory)))
        .collect::<Vec<_>>();
    let checkpoint = serde_json::to_string(&(from.clone(), to.clone()))?;
    if let Some(previous) =
        catalog::setting(pool, "store_directory_move").await?
        && !previous.is_empty()
        && previous != checkpoint
    {
        return Err(input(
            "Finish or cancel the previous app-directory move before choosing another destination",
        ));
    }
    catalog::set_setting(pool, "store_directory_move", &checkpoint).await?;
    let mut required = 0_u64;
    for directory in directories {
        let source = from.join(directory);
        if fs::try_exists(&source).await? {
            required = required.saturating_add(
                required_copy_bytes(&source, &to.join(directory)).await?,
            );
        }
    }
    if fs4::available_space(&to)? < required.saturating_add(64 * 1024 * 1024) {
        return Err(input(format!(
            "The destination needs at least {required} bytes plus working space for the app-directory move"
        )));
    }
    for directory in directories {
        let source = from.join(directory);
        if fs::try_exists(&source).await? {
            copy_tree(&source, &to.join(directory), &mappings).await?;
        }
    }
    rewrite_database_paths(pool, &mappings).await?;
    catalog::set_setting(pool, "store_directory_move_copied", &checkpoint)
        .await?;
    Ok(())
}

pub(crate) async fn finish_app_directory_move(
    from: &Path,
    to: &Path,
    pool: &SqlitePool,
) -> crate::Result<()> {
    let from = fs::canonicalize(from).await?;
    let to = fs::canonicalize(to).await?;
    if from == to {
        return Ok(());
    }
    if from.starts_with(&to) || to.starts_with(&from) {
        return Err(input("Invalid app-directory cleanup roots"));
    }
    let expected = serde_json::to_string(&(from.clone(), to.clone()))?;
    if catalog::setting(pool, "store_directory_move_copied")
        .await?
        .as_deref()
        != Some(expected.as_str())
    {
        return Err(input(
            "The app-directory copy has not completed; preserving the source",
        ));
    }
    for directory in [
        "store",
        "profiles",
        "meta",
        "caches",
        "icons",
        "synced-options",
    ] {
        let source = from.join(directory);
        if fs::try_exists(&source).await?
            && fs::try_exists(to.join(directory)).await?
        {
            remove_migrated_tree(&source).await?;
        }
    }
    catalog::set_setting(pool, "store_directory_move", "").await?;
    catalog::set_setting(pool, "store_directory_move_copied", "").await?;
    Ok(())
}

async fn required_copy_bytes(from: &Path, to: &Path) -> crate::Result<u64> {
    let mut pending = vec![(from.to_path_buf(), to.to_path_buf())];
    let mut bytes = 0_u64;
    while let Some((source, target)) = pending.pop() {
        let metadata = fs::symlink_metadata(&source).await?;
        if metadata.is_dir() && !metadata.file_type().is_symlink() {
            let mut entries = fs::read_dir(&source).await?;
            while let Some(entry) = entries.next_entry().await? {
                pending.push((entry.path(), target.join(entry.file_name())));
            }
        } else if metadata.is_file()
            && fs::symlink_metadata(&target).await.is_err()
        {
            bytes = bytes.saturating_add(metadata.len());
        }
    }
    Ok(bytes)
}

pub(crate) async fn resume_completed_move(
    destination: &Path,
    pool: &SqlitePool,
) -> crate::Result<()> {
    if let Some(checkpoint) =
        catalog::setting(pool, "store_directory_move").await?
        && !checkpoint.is_empty()
    {
        let (from, to): (PathBuf, PathBuf) = serde_json::from_str(&checkpoint)?;
        if fs::canonicalize(destination).await? == to {
            finish_app_directory_move(&from, &to, pool).await?;
        }
    }
    Ok(())
}

pub(crate) async fn cancel_move(pool: &SqlitePool) -> crate::Result<()> {
    if let Some(checkpoint) =
        catalog::setting(pool, "store_directory_move").await?
        && !checkpoint.is_empty()
    {
        let (from, to): (PathBuf, PathBuf) = serde_json::from_str(&checkpoint)?;
        let settings = crate::state::Settings::get(pool).await?;
        if settings.prev_custom_dir.as_deref()
            == Some(to.to_string_lossy().as_ref())
        {
            return Err(input(
                "The move has committed; finish its cleanup before moving back",
            ));
        }
        let mappings = [
            "store",
            "profiles",
            "meta",
            "caches",
            "icons",
            "synced-options",
        ]
        .iter()
        .map(|directory| (to.join(directory), from.join(directory)))
        .collect::<Vec<_>>();
        rewrite_database_paths(pool, &mappings).await?;
        catalog::set_setting(pool, "store_directory_move", "").await?;
        catalog::set_setting(pool, "store_directory_move_copied", "").await?;
    }
    Ok(())
}

async fn remove_migrated_tree(root: &Path) -> crate::Result<()> {
    if fs::symlink_metadata(root).await?.file_type().is_symlink() {
        return Err(input(
            "A migrated directory was replaced by a link; preserving it",
        ));
    }
    #[cfg(windows)]
    {
        let mut pending = vec![root.to_path_buf()];
        while let Some(path) = pending.pop() {
            let metadata = fs::symlink_metadata(&path).await?;
            if metadata.file_type().is_symlink() {
                continue;
            }
            if metadata.is_dir() {
                let mut entries = fs::read_dir(&path).await?;
                while let Some(entry) = entries.next_entry().await? {
                    pending.push(entry.path());
                }
            } else if metadata.is_file() && metadata.permissions().readonly() {
                let mut permissions = metadata.permissions();
                permissions.set_readonly(false);
                fs::set_permissions(path, permissions).await?;
            }
        }
    }
    fs::remove_dir_all(root).await?;
    Ok(())
}
