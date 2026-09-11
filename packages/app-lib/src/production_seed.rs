use crate::event::emit::{emit_loading, init_loading};
use crate::state::instances::adapters::sqlite::instance_rows;
use crate::state::{DirectoryInfo, JavaVersion, Settings, db, db_backup};
use crate::util::io::{self, IOError};
use crate::{ErrorKind, LoadingBarType};
use sqlx::{Connection, SqlitePool};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;

const COPY_DIRECTORY: &str = "production-data";
const COMPLETE_MARKER: &str = ".production-copy-complete";
const DATA_DIRECTORIES: &[&str] =
    &["profiles", "meta", "caches", "icons", "synced-options"];
static SEED_LOCK: Mutex<()> = Mutex::const_new(());

fn input_error(message: impl Into<String>) -> crate::Error {
    ErrorKind::InputError(message.into()).into()
}

/// Checks the test identity before logging or initialization can write app data.
pub fn validate_production_seed_target(
    app_identifier: &str,
) -> crate::Result<()> {
    if !app_identifier.starts_with("ModrinthApp-dev-")
        || !app_identifier.ends_with("-seeded")
        || std::env::var_os("THESEUS_CONFIG_DIR").is_some()
    {
        return Err(input_error(
            "Production data copying requires a seeded dev build with THESEUS_CONFIG_DIR unset.",
        ));
    }
    let destination = DirectoryInfo::initial_settings_dir_path(app_identifier)
        .ok_or_else(|| {
            input_error("Could not locate the test app data directory.")
        })?;
    if destination.try_exists()? {
        let parent = destination
            .parent()
            .ok_or_else(|| input_error("Invalid test app data directory."))?;
        if io::canonicalize(&destination)?
            != io::canonicalize(parent)?.join(app_identifier)
        {
            return Err(input_error(
                "The test app data directory must not be a link to another directory.",
            ));
        }
    }
    Ok(())
}

/// Copies a production installation before the test build opens its own database.
pub async fn seed_production_data_once(
    app_identifier: &str,
) -> crate::Result<()> {
    let _guard = SEED_LOCK.lock().await;
    validate_production_seed_target(app_identifier)?;
    let destination = DirectoryInfo::initial_settings_dir_path(app_identifier)
        .ok_or_else(|| {
            input_error("Could not locate the test app data directory.")
        })?;
    io::create_dir_all(&destination).await?;
    let destination = io::canonicalize(destination)?;
    let copied_data = destination.join(COPY_DIRECTORY);
    let copied_database = destination.join("app.db");
    let marker = copied_data.join(COMPLETE_MARKER);
    if copied_database.try_exists()? {
        if marker.try_exists()? {
            return Ok(());
        }
        return Err(input_error(format!(
            "The test directory already contains app data: {}. Use a new test build to create a production copy.",
            destination.display(),
        )));
    }
    if marker.try_exists()? {
        tokio::fs::rename(copied_data.join("app.db"), copied_database).await?;
        return Ok(());
    }
    if copied_data.try_exists()? {
        return Err(input_error(format!(
            "An incomplete production copy exists at {}. Move it aside before trying again.",
            copied_data.display(),
        )));
    }
    let source = dirs::data_dir()
        .ok_or_else(|| input_error("Could not locate production app data."))?
        .join("ModrinthApp");
    let source = io::canonicalize(source)?;
    ensure_separate(&source, &destination)?;
    ensure_production_closed().await?;

    let progress = Arc::new(
        init_loading(
            LoadingBarType::ConfigChange {
                new_path: copied_data.to_string_lossy().into_owned(),
            },
            100.0,
            "Preparing a copy of your production app data...",
        )
        .await?,
    );
    let temporary = tempfile::Builder::new()
        .prefix(".production-copy-")
        .tempdir_in(&destination)?;
    let staging = temporary.path().join(COPY_DIRECTORY);
    io::create_dir_all(&staging).await?;
    let snapshot = staging.join("app.db");
    let mut production_db =
        db_backup::open_read_only_db(&source.join("app.db")).await?;
    db_backup::create_sqlite_snapshot(&mut production_db, &snapshot).await?;
    production_db.close().await?;
    let pool = db::open_migrated_app_db(&snapshot).await?;
    let result = prepare_copy(
        &source,
        &staging,
        &copied_data,
        &destination,
        &pool,
        progress.clone(),
    )
    .await;
    pool.close().await;
    result?;
    let mut prepared_db = db_backup::open_read_only_db(&snapshot).await?;
    let ready_database = temporary.path().join("ready.db");
    db_backup::create_sqlite_snapshot(&mut prepared_db, &ready_database)
        .await?;
    prepared_db.close().await?;
    for name in ["app.db", "app.db-wal", "app.db-shm"] {
        let path = staging.join(name);
        if path.try_exists()? {
            io::remove_file(path).await?;
        }
    }
    tokio::fs::rename(ready_database, &snapshot).await?;

    io::write(staging.join(COMPLETE_MARKER), b"1").await?;
    tokio::fs::rename(&staging, &copied_data).await?;
    tokio::fs::rename(copied_data.join("app.db"), &copied_database).await?;
    emit_loading(
        &progress,
        5.0,
        Some("Production data copied. Starting the test app..."),
    )?;
    tracing::info!(
        "Seeded test app from {} into {}",
        source.display(),
        copied_data.display(),
    );
    Ok(())
}

async fn prepare_copy(
    source: &Path,
    staging: &Path,
    copied_data: &Path,
    destination: &Path,
    pool: &SqlitePool,
    progress: Arc<crate::event::LoadingBarId>,
) -> crate::Result<()> {
    let mut settings = Settings::get(pool).await?;
    if settings
        .prev_custom_dir
        .as_ref()
        .is_some_and(|previous| settings.custom_dir.as_ref() != Some(previous))
    {
        return Err(input_error(
            "Finish the directory move in the production app, then close it before copying its data.",
        ));
    }
    let source_data = io::canonicalize(
        settings
            .custom_dir
            .as_deref()
            .map(Path::new)
            .unwrap_or(source),
    )?;
    ensure_separate(&source_data, destination)?;
    ensure_instances_closed(pool).await?;
    let mappings = PathMappings::new(
        source.to_owned(),
        source_data.clone(),
        copied_data.to_owned(),
    );
    let copy_mappings = mappings.clone();
    let staging = staging.to_owned();
    tokio::task::spawn_blocking(move || {
        let mut plan = CopyPlan::default();
        for entry in std::fs::read_dir(&copy_mappings.settings)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_text = name.to_string_lossy();
            if matches!(
                name_text.as_ref(),
                "app.db" | "app.db-wal" | "app.db-shm" | "app.db-journal"
            ) || (copy_mappings.settings != copy_mappings.data
                && DATA_DIRECTORIES.contains(&name_text.as_ref()))
            {
                continue;
            }
            plan.collect(
                &entry.path(),
                Path::new(&name),
                &mut Vec::new(),
                &copy_mappings,
                copy_mappings.settings != copy_mappings.data,
            )?;
        }
        if copy_mappings.settings != copy_mappings.data {
            for name in DATA_DIRECTORIES {
                let source = copy_mappings.data.join(name);
                if source.try_exists()? {
                    plan.collect(
                        &source,
                        Path::new(name),
                        &mut Vec::new(),
                        &copy_mappings,
                        false,
                    )?;
                }
            }
        }
        plan.copy(&staging, &copy_mappings, &progress)
    })
    .await??;
    ensure_production_closed().await?;
    ensure_instances_closed(pool).await?;

    settings.custom_dir = Some(copied_data.to_string_lossy().into_owned());
    settings.prev_custom_dir = None;
    settings.skipped_update = None;
    settings.pending_update_toast_for_version = None;
    settings.auto_download_updates = Some(false);
    let mut tx = pool.begin().await?;
    settings.update(&mut *tx).await?;
    for (_, mut java) in JavaVersion::get_all(&mut *tx).await? {
        java.path = mappings.remap_string(&java.path);
        java.upsert(&mut *tx).await?;
    }
    for mut instance in instance_rows::list_instances(pool).await? {
        instance.icon_path =
            instance.icon_path.map(|path| mappings.remap_string(&path));
        instance_rows::update_instance(&instance, &mut tx).await?;
        if let Some(mut overrides) =
            instance_rows::get_instance_launch_overrides(&instance.id, &mut *tx)
                .await?
        {
            overrides.java_path =
                overrides.java_path.map(|path| mappings.remap_string(&path));
            instance_rows::upsert_instance_launch_overrides(
                &overrides, &mut tx,
            )
            .await?;
        }
    }
    tx.commit().await?;
    Ok(())
}

fn ensure_separate(source: &Path, destination: &Path) -> crate::Result<()> {
    if destination.starts_with(source) || source.starts_with(destination) {
        return Err(input_error(
            "The production and test data directories must be separate.",
        ));
    }
    Ok(())
}

async fn ensure_production_closed() -> crate::Result<()> {
    let running = tokio::task::spawn_blocking(|| {
        let system = sysinfo::System::new_all();
        system.processes().values().any(|process| {
            let name = process.name().to_string_lossy();
            name.eq_ignore_ascii_case("Modrinth App.exe")
                || name.eq_ignore_ascii_case("ModrinthApp.exe")
        })
    })
    .await?;
    if running {
        return Err(input_error(
            "Close the production Modrinth App and Minecraft, then restart this test build to copy your data.",
        ));
    }
    Ok(())
}

async fn ensure_instances_closed(pool: &SqlitePool) -> crate::Result<()> {
    let system = tokio::task::spawn_blocking(sysinfo::System::new_all).await?;
    for instance in instance_rows::list_instances(pool).await? {
        let instance_id = instance.id;
        let processes = sqlx::query!(
            "
		SELECT pid, start_time
		FROM processes
		WHERE instance_id = ?
		",
            instance_id,
        )
        .fetch_all(pool)
        .await?;
        for process in processes {
            if u32::try_from(process.pid)
                .ok()
                .and_then(|pid| system.process(sysinfo::Pid::from_u32(pid)))
                .is_some_and(|running| {
                    (running.start_time() as i64).abs_diff(process.start_time)
                        <= 2
                })
            {
                return Err(input_error(
                    "Close Minecraft before copying production app data, then restart this test build.",
                ));
            }
        }
    }
    Ok(())
}

#[derive(Clone)]
struct PathMappings {
    settings: PathBuf,
    data: PathBuf,
    destination: PathBuf,
}

impl PathMappings {
    fn new(settings: PathBuf, data: PathBuf, destination: PathBuf) -> Self {
        Self {
            settings,
            data,
            destination,
        }
    }

    fn relative(&self, path: &Path) -> Option<PathBuf> {
        let roots = if self.settings.starts_with(&self.data) {
            [&self.settings, &self.data]
        } else {
            [&self.data, &self.settings]
        };
        roots
            .into_iter()
            .find_map(|root| path.strip_prefix(root).ok().map(Path::to_owned))
    }

    fn remap_string(&self, path: &str) -> String {
        let relative = self.relative(Path::new(path)).or_else(|| {
            io::canonicalize(path)
                .ok()
                .and_then(|path| self.relative(&path))
        });
        relative.map_or_else(
            || path.to_owned(),
            |relative| {
                self.destination
                    .join(relative)
                    .to_string_lossy()
                    .into_owned()
            },
        )
    }
}

#[derive(PartialEq)]
struct FileStamp {
    length: u64,
    modified: SystemTime,
}

impl FileStamp {
    fn read(path: &Path) -> crate::Result<Self> {
        let metadata = std::fs::metadata(path)
            .map_err(|error| IOError::with_path(error, path))?;
        Ok(Self {
            length: metadata.len(),
            modified: metadata.modified()?,
        })
    }
}

struct CopyFile {
    source: PathBuf,
    relative: PathBuf,
    stamp: FileStamp,
    link_target: Option<PathBuf>,
}

#[derive(Default)]
struct CopyPlan {
    directories: Vec<PathBuf>,
    files: Vec<CopyFile>,
    links: Vec<CopyFile>,
}

impl CopyPlan {
    fn collect(
        &mut self,
        source: &Path,
        relative: &Path,
        ancestors: &mut Vec<PathBuf>,
        mappings: &PathMappings,
        skip_data_root: bool,
    ) -> crate::Result<()> {
        let metadata = std::fs::symlink_metadata(source)
            .map_err(|error| IOError::with_path(error, source))?;
        let resolved = io::canonicalize(source)?;
        if skip_data_root && resolved == mappings.data {
            return Ok(());
        }
        if resolved.is_dir() {
            if let Some(destination) = mappings.destination.parent() {
                ensure_separate(&resolved, destination)?;
            }
            if ancestors.contains(&resolved) {
                return Err(input_error(format!(
                    "A directory link forms a loop at {}.",
                    source.display()
                )));
            }
            ancestors.push(resolved);
            self.directories.push(relative.to_owned());
            for entry in std::fs::read_dir(source)? {
                let entry = entry?;
                self.collect(
                    &entry.path(),
                    &relative.join(entry.file_name()),
                    ancestors,
                    mappings,
                    skip_data_root,
                )?;
            }
            ancestors.pop();
        } else if resolved.is_file() {
            let link_target = metadata
                .file_type()
                .is_symlink()
                .then(|| mappings.relative(&resolved))
                .flatten();
            let file = CopyFile {
                source: source.to_owned(),
                relative: relative.to_owned(),
                stamp: FileStamp::read(source)?,
                link_target,
            };
            if file.link_target.is_some() {
                self.links.push(file);
            } else {
                self.files.push(file);
            }
        } else {
            return Err(input_error(format!(
                "Cannot copy {} as app data.",
                source.display()
            )));
        }
        Ok(())
    }

    fn copy(
        self,
        staging: &Path,
        mappings: &PathMappings,
        progress: &crate::event::LoadingBarId,
    ) -> crate::Result<()> {
        let total_bytes = self
            .files
            .iter()
            .chain(&self.links)
            .map(|file| file.stamp.length)
            .sum::<u64>();
        let disks = sysinfo::Disks::new_with_refreshed_list();
        if let Some(disk) = disks
            .iter()
            .filter(|disk| staging.starts_with(disk.mount_point()))
            .max_by_key(|disk| disk.mount_point().components().count())
            && total_bytes > disk.available_space()
        {
            return Err(input_error(
                "There is not enough free space to copy your production app data.",
            ));
        }
        for directory in &self.directories {
            std::fs::create_dir_all(staging.join(directory))?;
        }
        let total = self.files.len() + self.links.len();
        emit_loading(
            progress,
            5.0,
            Some("Copying your production app data..."),
        )?;
        for file in self.files.iter().chain(&self.links) {
            let destination = staging.join(&file.relative);
            if FileStamp::read(&file.source)? != file.stamp {
                return Err(input_error(
                    "Production files changed during the copy. Close Modrinth and Minecraft, then retry.",
                ));
            }
            let linked = file.link_target.as_ref().is_some_and(|relative| {
                let target = staging.join(relative);
                target.is_file()
                    && create_file_link(
                        &target,
                        &destination,
                        &mappings.destination.join(relative),
                    )
            });
            if !linked {
                std::fs::copy(&file.source, &destination)
                    .map_err(|error| IOError::with_path(error, &file.source))?;
            }
            if FileStamp::read(&file.source)? != file.stamp {
                return Err(input_error(
                    "Production files changed during the copy. Close Modrinth and Minecraft, then retry.",
                ));
            }
            let fraction = if total_bytes == 0 {
                1.0 / total.max(1) as f64
            } else {
                file.stamp.length as f64 / total_bytes as f64
            };
            emit_loading(progress, 90.0 * fraction, None)?;
        }
        for file in self.files.iter().chain(&self.links) {
            if FileStamp::read(&file.source)? != file.stamp {
                return Err(input_error(
                    "Production files changed during the copy. Close Modrinth and Minecraft, then retry.",
                ));
            }
        }
        if total == 0 {
            emit_loading(progress, 90.0, None)?;
        }
        Ok(())
    }
}

fn create_file_link(
    target: &Path,
    destination: &Path,
    final_target: &Path,
) -> bool {
    #[cfg(windows)]
    if std::os::windows::fs::symlink_file(final_target, destination).is_ok() {
        return true;
    }
    #[cfg(unix)]
    if std::os::unix::fs::symlink(final_target, destination).is_ok() {
        return true;
    }
    std::fs::hard_link(target, destination).is_ok()
}
