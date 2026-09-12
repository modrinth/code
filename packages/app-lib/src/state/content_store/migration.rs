use super::{
    catalog, hash_file, input, normalize, relative_link, sync_directory,
};
use crate::state::instances::adapters::sqlite::instance_rows;
use crate::state::{Instance, JavaVersion, State};
use serde_json::Value;
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use tokio::fs;

const MOVED_APP_DIRECTORIES: [&str; 6] = [
    "store",
    "profiles",
    "meta",
    "caches",
    "icons",
    "synced-options",
];

pub(crate) async fn migrate(state: &State) -> crate::Result<()> {
    let store = &state.content_store;
    if catalog::setting(&state.pool, "game_locales_layout_version")
        .await?
        .as_deref()
        != Some("1")
    {
        let source = state.directories.metadata_dir().join("game-locales");
        let destination = state.directories.caches_dir().join("game-locales");
        relocate_tree(&source, &destination).await?;
        if fs::try_exists(&source).await?
            && fs::try_exists(&destination).await?
        {
            remove_migrated_tree(&source).await?;
        }
        catalog::set_setting(&state.pool, "game_locales_layout_version", "1")
            .await?;
    }
    catalog::set_setting(&state.pool, "store_layout_version", "1").await?;
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
    store.recover_unregistered_files().await?;
    Ok(())
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

async fn copy_tree(
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

fn remap(path: &Path, mappings: &[(PathBuf, PathBuf)]) -> PathBuf {
    let map = |path: &Path| {
        mappings.iter().find_map(|(from, to)| {
            strip_directory_prefix(path, from).map(|suffix| to.join(suffix))
        })
    };
    map(&normalize(path))
        .or_else(|| {
            std::fs::canonicalize(path).ok().and_then(|path| map(&path))
        })
        .unwrap_or_else(|| path.to_path_buf())
}

fn strip_directory_prefix<'a>(
    path: &'a Path,
    directory: &Path,
) -> Option<&'a Path> {
    let mut components = path.components();
    for expected in directory.components() {
        let actual = components.next()?;
        #[cfg(windows)]
        let matches = {
            use std::path::{Component, Prefix};
            match (actual, expected) {
                (Component::Prefix(actual), Component::Prefix(expected)) => {
                    match (actual.kind(), expected.kind()) {
                        (
                            Prefix::Disk(a) | Prefix::VerbatimDisk(a),
                            Prefix::Disk(b) | Prefix::VerbatimDisk(b),
                        ) => a.eq_ignore_ascii_case(&b),
                        (
                            Prefix::UNC(a, share_a)
                            | Prefix::VerbatimUNC(a, share_a),
                            Prefix::UNC(b, share_b)
                            | Prefix::VerbatimUNC(b, share_b),
                        ) => {
                            a.eq_ignore_ascii_case(b)
                                && share_a.eq_ignore_ascii_case(share_b)
                        }
                        _ => actual == expected,
                    }
                }
                _ => actual == expected,
            }
        };
        #[cfg(not(windows))]
        let matches = actual == expected;
        if !matches {
            return None;
        }
    }
    Some(components.as_path())
}

fn rewrite_value(value: &mut Value, mappings: &[(PathBuf, PathBuf)]) {
    match value {
        Value::String(value) => {
            let path = Path::new(value);
            if path.is_absolute() {
                *value = dunce::simplified(&remap(path, mappings))
                    .to_string_lossy()
                    .into_owned();
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
    for (_, mut java) in JavaVersion::get_all_registered(pool).await? {
        java.path = dunce::simplified(&remap(Path::new(&java.path), mappings))
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
    let original_from = normalize(from);
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
    let mappings = [&from, &original_from]
        .into_iter()
        .flat_map(|source| {
            let destination = &to;
            MOVED_APP_DIRECTORIES.iter().map(move |directory| {
                (source.join(directory), destination.join(directory))
            })
        })
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
    for directory in MOVED_APP_DIRECTORIES {
        let source = from.join(directory);
        match fs::symlink_metadata(&source).await {
            Ok(metadata)
                if !metadata.is_dir() || metadata.file_type().is_symlink() =>
            {
                return Err(input(format!(
                    "The shared-data root {} must be a directory without a symbolic link before moving the app directory",
                    source.display(),
                )));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                continue;
            }
            Err(error) => return Err(error.into()),
        }
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
    for directory in MOVED_APP_DIRECTORIES {
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
    for directory in MOVED_APP_DIRECTORIES {
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
        let committed = match settings.prev_custom_dir.as_deref() {
            Some(previous) => fs::canonicalize(previous).await? == to,
            None => false,
        };
        if committed {
            return Err(input(
                "The move has committed; finish its cleanup before moving back",
            ));
        }
        let mappings = MOVED_APP_DIRECTORIES
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
