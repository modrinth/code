use crate::config;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

static LAUNCHER_LOG: LazyLock<Mutex<Option<File>>> = LazyLock::new(|| Mutex::new(None));

pub fn logs_dir() -> Result<PathBuf, String> {
    let dir = config::ensure_layout()?.join("logs");
    fs::create_dir_all(&dir).map_err(|e| format!("Create logs dir: {e}"))?;
    Ok(dir)
}

fn current_log_path() -> Result<PathBuf, String> {
    Ok(logs_dir()?.join("owyx.log"))
}

/// Archive previous `owyx.log` into a zip, keep last 10 archives, open a fresh log.
pub fn init_launcher_logging() -> Result<PathBuf, String> {
    let dir = logs_dir()?;
    let current = current_log_path()?;
    if current.is_file() {
        archive_previous_log(&dir, &current)?;
    }
    prune_archives(&dir, 10)?;

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&current)
        .map_err(|e| format!("Open launcher log: {e}"))?;

    {
        let mut guard = LAUNCHER_LOG
            .lock()
            .map_err(|_| "Launcher log lock poisoned".to_string())?;
        *guard = Some(file);
    }

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    info(&format!("=== Owyx launcher start ts={stamp} ==="));
    Ok(current)
}

fn archive_previous_log(dir: &Path, current: &Path) -> Result<(), String> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let archive_path = dir.join(format!("owyx-{stamp}.zip"));
    let archive_file =
        File::create(&archive_path).map_err(|e| format!("Create log archive: {e}"))?;
    let mut zip = ZipWriter::new(archive_file);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    zip.start_file("owyx.log", opts)
        .map_err(|e| format!("Zip start: {e}"))?;
    let bytes = fs::read(current).map_err(|e| format!("Read previous log: {e}"))?;
    zip.write_all(&bytes)
        .map_err(|e| format!("Zip write: {e}"))?;
    zip.finish().map_err(|e| format!("Zip finish: {e}"))?;
    let _ = fs::remove_file(current);
    Ok(())
}

fn prune_archives(dir: &Path, keep: usize) -> Result<(), String> {
    let mut archives: Vec<(u64, PathBuf)> = fs::read_dir(dir)
        .map_err(|e| format!("Read logs dir: {e}"))?
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            let name = path.file_name()?.to_str()?.to_string();
            if !(name.starts_with("owyx-") && name.ends_with(".zip")) {
                return None;
            }
            let meta = entry.metadata().ok()?;
            let modified = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            Some((modified, path))
        })
        .collect();
    archives.sort_by(|a, b| b.0.cmp(&a.0));
    for (_m, path) in archives.into_iter().skip(keep) {
        let _ = fs::remove_file(path);
    }
    Ok(())
}

pub fn info(message: &str) {
    let line = format_line("INFO", message);
    eprintln!("{line}");
    append_launcher_line(&line);
}

pub fn warn(message: &str) {
    let line = format_line("WARN", message);
    eprintln!("{line}");
    append_launcher_line(&line);
}

pub fn error(message: &str) {
    let line = format_line("ERROR", message);
    eprintln!("{line}");
    append_launcher_line(&line);
}

fn format_line(level: &str, message: &str) -> String {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("[{stamp}] [{level}] {message}")
}

fn append_launcher_line(line: &str) {
    if let Ok(mut guard) = LAUNCHER_LOG.lock() {
        if let Some(file) = guard.as_mut() {
            let _ = writeln!(file, "{line}");
            let _ = file.flush();
        }
    }
}

pub fn create_launch_log(pack_id: &str) -> Result<(PathBuf, File), String> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let path = logs_dir()?.join(format!("launch-{pack_id}-{stamp}.log"));
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("Open launch log: {e}"))?;
    Ok((path, file))
}

pub fn write_line(file: &mut File, line: &str) -> Result<(), String> {
    writeln!(file, "{line}").map_err(|e| format!("Write log: {e}"))?;
    file.flush().map_err(|e| format!("Flush log: {e}"))
}
