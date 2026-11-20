use super::instance::{ContentListFormat, InstanceContentSnapshot};
use crate::state::InstanceLaunchOverrides;
use crate::state::db_backup::{
    copy_db_files, create_sqlite_snapshot, remove_partial_backup,
};
use crate::util::io::IOError;
use crate::util::redactions::{Redactor, scrub_database_snapshot};
use crate::{ErrorKind, State};
use flate2::read::GzDecoder;
use serde::Serialize;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DebugInfoExportStage {
    Preparing,
    Exporting,
    Finishing,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugInfoExportProgress {
    pub stage: DebugInfoExportStage,
    pub processed_bytes: u64,
    pub total_bytes: Option<u64>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Redaction {
    None,
    Log,
    Config,
}

struct ExportFile {
    path: PathBuf,
    name: String,
    length: u64,
    redaction: Redaction,
    gzip: bool,
}

struct ExportEntry {
    name: &'static str,
    contents: Vec<u8>,
}

const CONFIG_DIRECTORIES: [&str; 2] = ["config", "defaultconfigs"];
const CONFIG_EXTENSIONS: [&str; 13] = [
    "toml",
    "json",
    "json5",
    "jsonc",
    "cfg",
    "conf",
    "properties",
    "txt",
    "yml",
    "yaml",
    "ini",
    "snbt",
    "hjson",
];
const MAX_CONFIG_FILE_BYTES: u64 = 2 * 1024 * 1024;
const OPTIONS_FILES: [&str; 3] =
    ["options.txt", "optionsof.txt", "optionsshaders.txt"];
const REDACTED: &str = "{REDACTED}";

pub async fn export_debug_info(
    export_path: PathBuf,
    cancel: CancellationToken,
    on_progress: impl Fn(DebugInfoExportProgress) + Send + Sync + 'static,
) -> crate::Result<bool> {
    on_progress(preparing());
    let state = tokio::select! {
        _ = cancel.cancelled() => return Ok(false),
        state = State::get() => state?,
    };
    let directories = state.directories.clone();
    let redactor = Redactor::load(&state.pool).await?;
    let database_files = tempfile::tempdir()?;
    let database_snapshot = database_files.path().join("app.db");
    let mut connection = tokio::select! {
        _ = cancel.cancelled() => return Ok(false),
        connection = state.pool.acquire() => connection?,
    };
    if let Err(error) =
        create_sqlite_snapshot(&mut connection, &database_snapshot).await
    {
        tracing::warn!(
            "Failed to snapshot app database for debug info export, copying the database files instead: {error}"
        );
        remove_partial_backup(&database_snapshot).await;
        copy_db_files(
            &directories.settings_dir.join("app.db"),
            &database_snapshot,
        )
        .await?;
    }
    drop(connection);
    if cancel.is_cancelled() {
        return Ok(false);
    }
    let include_database = match scrub_database_snapshot(&database_snapshot)
        .await
    {
        Ok(()) => true,
        Err(error) => {
            tracing::warn!(
                "Failed to remove credentials from the app database snapshot, leaving it out of the debug info export: {error}"
            );
            false
        }
    };
    if cancel.is_cancelled() {
        return Ok(false);
    }
    let environment = environment_entry(
        serde_json::json!({
            "app_identifier": directories.app_identifier,
            "settings_dir": redactor.apply_path(&directories.settings_dir),
            "config_dir": redactor.apply_path(&directories.config_dir),
        }),
        &redactor,
    )?;
    let database_dir = database_files.path().to_path_buf();

    write_archive(
        export_path,
        redactor,
        vec![environment],
        Some(database_files),
        cancel,
        on_progress,
        move |files, excluded_paths, cancel| {
            if include_database {
                collect_files(
                    files,
                    &database_dir,
                    "database",
                    Redaction::None,
                    excluded_paths,
                    cancel,
                )?;
            }
            if let Some(logs_dir) = directories.launcher_logs_dir() {
                collect_files(
                    files,
                    &logs_dir,
                    "launcher_logs",
                    Redaction::Log,
                    excluded_paths,
                    cancel,
                )?;
            }
            let instances_dir = directories.instances_dir();
            let entries = match std::fs::read_dir(&instances_dir) {
                Ok(entries) => entries,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    return Ok(());
                }
                Err(error) => {
                    return Err(
                        IOError::with_path(error, &instances_dir).into()
                    );
                }
            };
            for entry in entries {
                if cancel.is_cancelled() {
                    return Ok(());
                }
                let entry = entry?;
                if !entry.file_type()?.is_dir() {
                    continue;
                }
                collect_instance_logs(
                    files,
                    &entry.path(),
                    &format!(
                        "instances/{}/",
                        entry.file_name().to_string_lossy()
                    ),
                    excluded_paths,
                    cancel,
                )?;
            }
            Ok(())
        },
    )
    .await
}

/// Exports the logs, crash reports, configs, game options, content list and settings of one
/// instance, with credentials, IP addresses, computer usernames and config secrets removed.
pub async fn export_instance_debug_info(
    instance_id: &str,
    export_path: PathBuf,
    cancel: CancellationToken,
    on_progress: impl Fn(DebugInfoExportProgress) + Send + Sync + 'static,
) -> crate::Result<bool> {
    on_progress(preparing());
    let state = tokio::select! {
        _ = cancel.cancelled() => return Ok(false),
        state = State::get() => state?,
    };
    let directories = state.directories.clone();
    let redactor = Redactor::load(&state.pool).await?;
    let mut metadata = crate::state::get_instance(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            ErrorKind::InputError(format!(
                "Instance {instance_id} does not exist"
            ))
        })?;
    if cancel.is_cancelled() {
        return Ok(false);
    }
    redact_launch_overrides(&mut metadata.launch_overrides);
    let content = match tokio::select! {
        _ = cancel.cancelled() => return Ok(false),
        content = InstanceContentSnapshot::load(instance_id) => content,
    } {
        Ok(content) => Some(content),
        Err(error) => {
            tracing::warn!(
                "Failed to list content for instance debug info export, leaving it out: {error}"
            );
            None
        }
    };
    let instance_dir =
        directories.instances_dir().join(&metadata.instance.path);
    let mut entries = vec![
        environment_entry(
            serde_json::json!({
                "app_identifier": directories.app_identifier,
                "instance_dir": redactor.apply_path(&instance_dir),
            }),
            &redactor,
        )?,
        json_entry("instance.json", &metadata, &redactor)?,
    ];
    if let Some(content) = content {
        entries.push(json_entry("content.json", &content, &redactor)?);
        entries.push(ExportEntry {
            name: "content.txt",
            contents: redactor.apply(
                content
                    .format(&metadata, ContentListFormat::Detailed)
                    .into_bytes(),
            ),
        });
    }
    let launcher_logs_dir = directories.launcher_logs_dir();

    write_archive(
        export_path,
        redactor,
        entries,
        None,
        cancel,
        on_progress,
        move |files, excluded_paths, cancel| {
            collect_instance_logs(
                files,
                &instance_dir,
                "",
                excluded_paths,
                cancel,
            )?;
            for name in OPTIONS_FILES {
                collect_files(
                    files,
                    &instance_dir.join(name),
                    name,
                    Redaction::Log,
                    excluded_paths,
                    cancel,
                )?;
            }
            for name in CONFIG_DIRECTORIES {
                let mut configs = Vec::new();
                collect_files(
                    &mut configs,
                    &instance_dir.join(name),
                    name,
                    Redaction::Config,
                    excluded_paths,
                    cancel,
                )?;
                files.extend(configs.into_iter().filter(is_config_file));
            }
            if let Some(path) = launcher_logs_dir
                .as_deref()
                .and_then(latest_file)
                .transpose()?
            {
                let name = path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default();
                collect_files(
                    files,
                    &path,
                    &format!("launcher_logs/{name}"),
                    Redaction::Log,
                    excluded_paths,
                    cancel,
                )?;
            }
            Ok(())
        },
    )
    .await
}

/// Masks the free-form launch settings that can hold credentials: environment variable values,
/// hook commands, and launch arguments other than `-X` JVM options and `-D` property names.
fn redact_launch_overrides(overrides: &mut InstanceLaunchOverrides) {
    for (_, value) in overrides.custom_env_vars.iter_mut().flatten() {
        *value = REDACTED.to_string();
    }
    for argument in overrides.extra_launch_args.iter_mut().flatten() {
        *argument = redact_launch_argument(argument);
    }
    let hooks = &mut overrides.hooks;
    for hook in [
        &mut hooks.pre_launch,
        &mut hooks.wrapper,
        &mut hooks.post_exit,
    ]
    .into_iter()
    .flatten()
    {
        *hook = REDACTED.to_string();
    }
}

fn redact_launch_argument(argument: &str) -> String {
    if argument.starts_with("-X") {
        return argument.to_string();
    }
    match argument
        .strip_prefix("-D")
        .and_then(|property| property.split_once('='))
    {
        Some((name, _)) => format!("-D{name}={REDACTED}"),
        None => REDACTED.to_string(),
    }
}

fn preparing() -> DebugInfoExportProgress {
    DebugInfoExportProgress {
        stage: DebugInfoExportStage::Preparing,
        processed_bytes: 0,
        total_bytes: None,
    }
}

fn environment_entry(
    extra: serde_json::Value,
    redactor: &Redactor,
) -> crate::Result<ExportEntry> {
    let mut environment = serde_json::json!({
        "exported_at": chrono::Utc::now(),
        "app_version": env!("CARGO_PKG_VERSION"),
        "os": std::env::consts::OS,
        "os_version": sysinfo::System::long_os_version(),
        "kernel_version": sysinfo::System::kernel_version(),
        "architecture": std::env::consts::ARCH,
    });
    if let (Some(environment), serde_json::Value::Object(extra)) =
        (environment.as_object_mut(), extra)
    {
        environment.extend(extra);
    }
    json_entry("environment.json", &environment, redactor)
}

fn json_entry(
    name: &'static str,
    value: &impl Serialize,
    redactor: &Redactor,
) -> crate::Result<ExportEntry> {
    Ok(ExportEntry {
        name,
        contents: redactor.apply(serde_json::to_vec_pretty(value)?),
    })
}

fn is_config_file(file: &ExportFile) -> bool {
    file.length <= MAX_CONFIG_FILE_BYTES
        && file
            .path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                CONFIG_EXTENSIONS
                    .iter()
                    .any(|allowed| extension.eq_ignore_ascii_case(allowed))
            })
}

fn latest_file(directory: &Path) -> Option<crate::Result<PathBuf>> {
    let entries = match std::fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return None;
        }
        Err(error) => {
            return Some(Err(IOError::with_path(error, directory).into()));
        }
    };
    entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let metadata = entry
                .metadata()
                .ok()
                .filter(|metadata| metadata.is_file())?;
            Some((metadata.modified().ok()?, entry.path()))
        })
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| Ok(path))
}

fn collect_instance_logs(
    files: &mut Vec<ExportFile>,
    instance_dir: &Path,
    prefix: &str,
    excluded_paths: &[PathBuf],
    cancel: &CancellationToken,
) -> crate::Result<()> {
    for name in ["logs", "crash-reports"] {
        collect_files(
            files,
            &instance_dir.join(name),
            &format!("{prefix}{name}"),
            Redaction::Log,
            excluded_paths,
            cancel,
        )?;
    }
    let entries = match std::fs::read_dir(instance_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(());
        }
        Err(error) => {
            return Err(IOError::with_path(error, instance_dir).into());
        }
    };
    for file in entries {
        if cancel.is_cancelled() {
            return Ok(());
        }
        let file = file?;
        let name = file.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("hs_err_pid") && name.ends_with(".log") {
            collect_files(
                files,
                &file.path(),
                &format!("{prefix}{name}"),
                Redaction::Log,
                excluded_paths,
                cancel,
            )?;
        }
    }
    Ok(())
}

async fn write_archive(
    export_path: PathBuf,
    redactor: Redactor,
    entries: Vec<ExportEntry>,
    source_files: Option<tempfile::TempDir>,
    cancel: CancellationToken,
    on_progress: impl Fn(DebugInfoExportProgress) + Send + Sync + 'static,
    collect: impl FnOnce(
        &mut Vec<ExportFile>,
        &[PathBuf],
        &CancellationToken,
    ) -> crate::Result<()>
    + Send
    + 'static,
) -> crate::Result<bool> {
    let mut buffer = vec![0; 64 * 1024];

    tokio::task::spawn_blocking(move || -> crate::Result<bool> {
        let _source_files = source_files;
        let parent = export_path.parent().ok_or(ErrorKind::InputError(
            "export path has no parent directory".to_string(),
        ))?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
        let excluded_paths = [
            std::fs::canonicalize(temporary.path())?,
            std::fs::canonicalize(&export_path)
                .unwrap_or_else(|_| export_path.clone()),
        ];
        let mut files = Vec::new();
        collect(&mut files, &excluded_paths, &cancel)?;
        if cancel.is_cancelled() {
            return Ok(false);
        }

        let total_bytes = files.iter().map(|file| file.length).sum();
        let mut progress = DebugInfoExportProgress {
            stage: DebugInfoExportStage::Exporting,
            processed_bytes: 0,
            total_bytes: Some(total_bytes),
        };
        on_progress(progress);
        let mut last_sent = Instant::now();
        let mut archive =
            ZipWriter::new(BufWriter::new(temporary.as_file_mut()));
        for file in files {
            if cancel.is_cancelled() {
                return Ok(false);
            }
            let input = match File::open(&file.path) {
                Ok(input) => input,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    progress.total_bytes =
                        progress.total_bytes.map(|total| total - file.length);
                    continue;
                }
                Err(error) => {
                    return Err(IOError::with_path(error, &file.path).into());
                }
            };
            let mut input = input.take(file.length);
            if file.redaction != Redaction::None {
                let mut reader: Box<dyn BufRead> = if file.gzip {
                    Box::new(BufReader::new(GzDecoder::new(input)))
                } else {
                    Box::new(BufReader::new(input))
                };
                if file.redaction == Redaction::Config {
                    let mut contents = Vec::new();
                    reader.read_to_end(&mut contents).map_err(|error| {
                        IOError::with_path(error, &file.path)
                    })?;
                    progress.processed_bytes += file.length;
                    let Some(contents) = redactor.apply_config(contents) else {
                        tracing::warn!(
                            "Leaving {} out of the debug info export since it has a multiline secret value",
                            file.name
                        );
                        continue;
                    };
                    archive
                        .start_file(&file.name, file_options())
                        .map_err(std::io::Error::from)?;
                    archive.write_all(&contents)?;
                    continue;
                }
                archive
                    .start_file(
                        &file.name,
                        file_options().large_file(
                            file.gzip
                                || file.length >= zip::ZIP64_BYTES_THR / 2,
                        ),
                    )
                    .map_err(std::io::Error::from)?;
                let mut line = Vec::new();
                loop {
                    if cancel.is_cancelled() {
                        return Ok(false);
                    }
                    match reader.read_until(b'\n', &mut line) {
                        Ok(0) => break,
                        Ok(_) => {}
                        Err(error) if file.gzip => {
                            tracing::warn!(
                                "Failed to decompress {} for debug info export, keeping the part read before the error: {error}",
                                file.path.display()
                            );
                            break;
                        }
                        Err(error) => {
                            return Err(
                                IOError::with_path(error, &file.path).into()
                            );
                        }
                    }
                    archive
                        .write_all(&redactor.apply(std::mem::take(&mut line)))?;
                }
                progress.processed_bytes += file.length;
                if last_sent.elapsed() >= Duration::from_millis(200) {
                    on_progress(progress);
                    last_sent = Instant::now();
                }
                continue;
            }
            archive
                .start_file(
                    &file.name,
                    file_options()
                        .large_file(file.length >= zip::ZIP64_BYTES_THR),
                )
                .map_err(std::io::Error::from)?;
            let mut copied = 0;
            loop {
                if cancel.is_cancelled() {
                    return Ok(false);
                }
                let count = std::io::Read::read(&mut input, &mut buffer)
                    .map_err(|error| IOError::with_path(error, &file.path))?;
                if count == 0 {
                    break;
                }
                archive.write_all(&buffer[..count])?;
                copied += count as u64;
                progress.processed_bytes += count as u64;
                if last_sent.elapsed() >= Duration::from_millis(200) {
                    on_progress(progress);
                    last_sent = Instant::now();
                }
            }
            progress.total_bytes = progress
                .total_bytes
                .map(|total| total - (file.length - copied));
        }
        if cancel.is_cancelled() {
            return Ok(false);
        }
        progress.stage = DebugInfoExportStage::Finishing;
        on_progress(progress);
        for entry in entries {
            archive
                .start_file(entry.name, file_options())
                .map_err(std::io::Error::from)?;
            archive.write_all(&entry.contents)?;
        }
        archive.finish().map_err(std::io::Error::from)?.flush()?;
        temporary.as_file().sync_all()?;
        if cancel.is_cancelled() {
            return Ok(false);
        }
        temporary
            .persist(&export_path)
            .map_err(|error| IOError::with_path(error.error, &export_path))?;
        Ok(true)
    })
    .await?
}

fn file_options() -> SimpleFileOptions {
    SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o600)
}

fn collect_files(
    files: &mut Vec<ExportFile>,
    path: &Path,
    archive_name: &str,
    redaction: Redaction,
    excluded_paths: &[PathBuf],
    cancel: &CancellationToken,
) -> crate::Result<()> {
    if cancel.is_cancelled() {
        return Ok(());
    }
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(());
        }
        Err(error) => return Err(IOError::with_path(error, path).into()),
    };
    if metadata.is_dir() {
        let entries = std::fs::read_dir(path)
            .map_err(|error| IOError::with_path(error, path))?;
        for entry in entries {
            if cancel.is_cancelled() {
                return Ok(());
            }
            let entry = entry?;
            collect_files(
                files,
                &entry.path(),
                &format!(
                    "{archive_name}/{}",
                    entry.file_name().to_string_lossy()
                ),
                redaction,
                excluded_paths,
                cancel,
            )?;
        }
    } else if metadata.is_file()
        && !std::fs::canonicalize(path)
            .is_ok_and(|path| excluded_paths.contains(&path))
    {
        let gzip_name = archive_name
            .strip_suffix(".gz")
            .filter(|_| redaction != Redaction::None);
        files.push(ExportFile {
            path: path.to_path_buf(),
            name: gzip_name.unwrap_or(archive_name).to_string(),
            length: metadata.len(),
            redaction,
            gzip: gzip_name.is_some(),
        });
    }
    Ok(())
}
