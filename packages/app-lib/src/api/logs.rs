use std::fmt::Write as _;
use std::io::{BufRead, SeekFrom};
use std::time::SystemTime;

use futures::TryFutureExt;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt},
};

use crate::{
    State,
    prelude::Credentials,
    util::io::{self, IOError},
};

#[derive(Serialize, Debug)]
pub struct Logs {
    pub log_type: LogType,
    pub filename: String,
    pub age: u64,
    pub output: Option<CensoredString>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum LogType {
    InfoLog,
    CrashReport,
}

const LOG_COMPACTION_THRESHOLD: usize = 20;

#[derive(Serialize, Debug)]
pub struct LatestLogCursor {
    pub cursor: u64,
    pub output: CensoredString,
    pub new_file: bool,
}

#[derive(Serialize, Debug)] // Not deserialize
#[serde(transparent)]
pub struct CensoredString(String);
impl CensoredString {
    pub fn censor(mut s: String, credentials_list: &[Credentials]) -> Self {
        let username = whoami::username();
        s = s
            .replace(&format!("/{username}/"), "/{COMPUTER_USERNAME}/")
            .replace(&format!("\\{username}\\"), "\\{COMPUTER_USERNAME}\\");
        for credentials in credentials_list {
            // Use the offline profile to guarantee that this function does not cause
            // Mojang API request, and is never delayed by a network request. The offline
            // profile is optimistically updated on upsert from time to time anyway
            s = s
                .replace(&credentials.access_token, "{MINECRAFT_ACCESS_TOKEN}")
                .replace(
                    &credentials.offline_profile.name,
                    "{MINECRAFT_USERNAME}",
                )
                .replace(
                    &credentials.offline_profile.id.as_simple().to_string(),
                    "{MINECRAFT_UUID}",
                )
                .replace(
                    &credentials.offline_profile.id.as_hyphenated().to_string(),
                    "{MINECRAFT_UUID}",
                );
        }

        Self(s)
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct LogCompactionStats {
    compacted_runs: usize,
    compacted_lines: usize,
}

struct CompactedLog {
    output: String,
    stats: LogCompactionStats,
}

async fn resolve_instance_path(
    instance: &str,
    state: &State,
) -> crate::Result<String> {
    sqlx::query_scalar::<_, String>(
        "
        SELECT path
        FROM instances
        WHERE id = ? OR path = ?
        ORDER BY CASE WHEN id = ? THEN 0 ELSE 1 END
        LIMIT 1
        ",
    )
    .bind(instance)
    .bind(instance)
    .bind(instance)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| {
        crate::ErrorKind::InputError(format!(
            "Unknown instance id or path: {instance}"
        ))
        .as_error()
    })
}

fn split_line_ending(line: &str) -> (&str, &str) {
    if let Some(line) = line.strip_suffix("\r\n") {
        (line, "\r\n")
    } else if let Some(line) = line.strip_suffix('\n') {
        (line, "\n")
    } else if let Some(line) = line.strip_suffix('\r') {
        (line, "\r")
    } else {
        (line, "")
    }
}

fn push_compacted_log_run(
    output: &mut String,
    stats: &mut LogCompactionStats,
    line: &str,
    line_ending: &str,
    count: usize,
) {
    if count >= LOG_COMPACTION_THRESHOLD {
        output.push_str(line);
        let _ = write!(output, " (x{count} times - compacted by Modrinth App)");
        output.push_str(line_ending);
        stats.compacted_runs += 1;
        stats.compacted_lines += count;
    } else {
        for _ in 0..count {
            output.push_str(line);
            output.push_str(line_ending);
        }
    }
}

fn read_compacted_log<R: BufRead>(
    reader: &mut R,
) -> std::io::Result<CompactedLog> {
    let mut output = String::new();
    let mut stats = LogCompactionStats::default();
    let mut buffer = Vec::new();
    let mut current_line: Option<String> = None;
    let mut current_line_ending = String::new();
    let mut current_count = 0usize;

    loop {
        buffer.clear();
        let bytes_read = reader.read_until(b'\n', &mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        let line = String::from_utf8_lossy(&buffer);
        let (line, line_ending) = split_line_ending(&line);

        match current_line.as_deref() {
            Some(current) if current == line => {
                current_count += 1;
                if current_line_ending.is_empty() && !line_ending.is_empty() {
                    current_line_ending = line_ending.to_string();
                }
            }
            _ => {
                if let Some(current) = current_line.take() {
                    push_compacted_log_run(
                        &mut output,
                        &mut stats,
                        &current,
                        &current_line_ending,
                        current_count,
                    );
                }

                current_line = Some(line.to_string());
                current_line_ending = line_ending.to_string();
                current_count = 1;
            }
        }
    }

    if let Some(current) = current_line {
        push_compacted_log_run(
            &mut output,
            &mut stats,
            &current,
            &current_line_ending,
            current_count,
        );
    }

    Ok(CompactedLog { output, stats })
}

fn compact_duplicate_lines(input: &str) -> CompactedLog {
    let mut reader = std::io::Cursor::new(input.as_bytes());
    read_compacted_log(&mut reader)
        .expect("compacting an in-memory log should not fail")
}

fn format_count(count: usize) -> String {
    let raw = count.to_string();
    let mut formatted = String::with_capacity(raw.len() + raw.len() / 3);
    for (index, character) in raw.chars().enumerate() {
        if index > 0 && (raw.len() - index).is_multiple_of(3) {
            formatted.push(',');
        }
        formatted.push(character);
    }
    formatted
}

async fn maybe_emit_log_compaction_warning(
    file_name: &str,
    stats: LogCompactionStats,
) {
    if stats.compacted_runs == 0 {
        return;
    }

    let _ = crate::event::emit::emit_warning(&format!(
        "Modrinth App has compacted {} repeated log lines in {} before displaying it for performance reasons.",
        format_count(stats.compacted_lines),
        file_name,
    ))
    .await;
}

impl Logs {
    async fn build(
        log_type: LogType,
        age: SystemTime,
        instance_path: &str,
        filename: String,
        clear_contents: Option<bool>,
    ) -> crate::Result<Self> {
        Ok(Self {
            log_type,
            age: age
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_secs(),
            output: if clear_contents.unwrap_or(false) {
                None
            } else {
                Some(
                    get_output_by_filename(instance_path, log_type, &filename)
                        .await?,
                )
            },
            filename,
        })
    }
}

#[tracing::instrument]
pub async fn get_logs_from_type(
    instance_id: &str,
    log_type: LogType,
    clear_contents: Option<bool>,
    logs: &mut Vec<crate::Result<Logs>>,
) -> crate::Result<()> {
    let state = State::get().await?;
    let instance_path = resolve_instance_path(instance_id, &state).await?;

    let logs_folder = match log_type {
        LogType::InfoLog => state.directories.instance_logs_dir(&instance_path),
        LogType::CrashReport => {
            state.directories.crash_reports_dir(&instance_path)
        }
    };

    if logs_folder.exists() {
        for entry in std::fs::read_dir(&logs_folder)
            .map_err(|e| IOError::with_path(e, &logs_folder))?
        {
            let entry: std::fs::DirEntry =
                entry.map_err(|e| IOError::with_path(e, &logs_folder))?;
            let age = entry
                .metadata()?
                .created()
                .unwrap_or(SystemTime::UNIX_EPOCH);
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if let Some(file_name) = path.file_name() {
                let file_name = file_name.to_string_lossy().to_string();
                logs.push(
                    Logs::build(
                        log_type,
                        age,
                        &instance_path,
                        file_name,
                        clear_contents,
                    )
                    .await,
                );
            }
        }
    }
    Ok(())
}

#[tracing::instrument]
pub async fn get_logs(
    instance_id: &str,
    clear_contents: Option<bool>,
) -> crate::Result<Vec<Logs>> {
    let mut logs = Vec::new();
    get_logs_from_type(
        instance_id,
        LogType::InfoLog,
        clear_contents,
        &mut logs,
    )
    .await?;
    get_logs_from_type(
        instance_id,
        LogType::CrashReport,
        clear_contents,
        &mut logs,
    )
    .await?;

    let mut logs = logs.into_iter().collect::<crate::Result<Vec<Logs>>>()?;
    logs.sort_by(|a, b| b.age.cmp(&a.age).then(b.filename.cmp(&a.filename)));
    Ok(logs)
}

#[tracing::instrument]
pub async fn get_logs_by_filename(
    instance_id: &str,
    log_type: LogType,
    filename: String,
) -> crate::Result<Logs> {
    let state = State::get().await?;
    let instance_path = resolve_instance_path(instance_id, &state).await?;

    let path = match log_type {
        LogType::InfoLog => state.directories.instance_logs_dir(&instance_path),
        LogType::CrashReport => {
            state.directories.crash_reports_dir(&instance_path)
        }
    }
    .join(&filename);

    let metadata = std::fs::metadata(&path)?;
    let age = metadata.created().unwrap_or(SystemTime::UNIX_EPOCH);

    Logs::build(log_type, age, &instance_path, filename, Some(true)).await
}

#[tracing::instrument]
pub async fn get_output_by_filename(
    instance_path: &str,
    log_type: LogType,
    file_name: &str,
) -> crate::Result<CensoredString> {
    let state = State::get().await?;

    let logs_folder = match log_type {
        LogType::InfoLog => state.directories.instance_logs_dir(instance_path),
        LogType::CrashReport => {
            state.directories.crash_reports_dir(instance_path)
        }
    };

    let path = logs_folder.join(file_name);

    let credentials = Credentials::get_all(&state.pool)
        .await?
        .into_iter()
        .map(|x| x.1)
        .collect::<Vec<_>>();

    if let Some(ext) = path.extension() {
        if ext == "gz" {
            let file = std::fs::File::open(&path)
                .map_err(|e| IOError::with_path(e, &path))?;
            let gz =
                flate2::read::GzDecoder::new(std::io::BufReader::new(file));
            let mut reader = std::io::BufReader::new(gz);
            let compacted = read_compacted_log(&mut reader)
                .map_err(|e| IOError::with_path(e, &path))?;
            maybe_emit_log_compaction_warning(file_name, compacted.stats).await;
            return Ok(CensoredString::censor(compacted.output, &credentials));
        } else if ext == "log" || ext == "txt" {
            let file = std::fs::File::open(&path)
                .map_err(|e| IOError::with_path(e, &path))?;
            let mut reader = std::io::BufReader::new(file);
            let compacted = read_compacted_log(&mut reader)
                .map_err(|e| IOError::with_path(e, &path))?;
            maybe_emit_log_compaction_warning(file_name, compacted.stats).await;
            return Ok(CensoredString::censor(compacted.output, &credentials));
        }
    }
    Err(crate::ErrorKind::OtherError(format!(
        "File extension not supported: {}",
        path.display()
    ))
    .into())
}

#[tracing::instrument]
pub async fn delete_logs(instance_id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let instance_path = resolve_instance_path(instance_id, &state).await?;

    let logs_folder = state.directories.instance_logs_dir(&instance_path);
    for entry in std::fs::read_dir(&logs_folder)
        .map_err(|e| IOError::with_path(e, &logs_folder))?
    {
        let entry = entry.map_err(|e| IOError::with_path(e, &logs_folder))?;
        let path = entry.path();
        if path.is_dir() {
            io::remove_dir_all(&path).await?;
        }
    }
    Ok(())
}

#[tracing::instrument]
pub async fn delete_logs_by_filename(
    instance_id: &str,
    log_type: LogType,
    filename: &str,
) -> crate::Result<()> {
    let state = State::get().await?;
    let instance_path = resolve_instance_path(instance_id, &state).await?;

    let logs_folder = match log_type {
        LogType::InfoLog => state.directories.instance_logs_dir(&instance_path),
        LogType::CrashReport => {
            state.directories.crash_reports_dir(&instance_path)
        }
    };

    let path = logs_folder.join(filename);
    io::remove_file(&path).await?;
    Ok(())
}

#[tracing::instrument]
pub async fn get_live_log_buffer(
    instance_id: &str,
) -> crate::Result<CensoredString> {
    let state = State::get().await?;
    let lines = crate::state::get_log_buffer(instance_id);
    let joined = lines.join("\n");
    let compacted = compact_duplicate_lines(&joined);

    let credentials = Credentials::get_all(&state.pool)
        .await?
        .into_iter()
        .map(|x| x.1)
        .collect::<Vec<_>>();
    maybe_emit_log_compaction_warning("live log", compacted.stats).await;
    Ok(CensoredString::censor(compacted.output, &credentials))
}

pub fn clear_live_log_buffer(instance_id: &str) {
    crate::state::remove_log_buffer(instance_id);
}

#[tracing::instrument]
pub async fn get_latest_log_cursor(
    instance_id: &str,
    cursor: u64, // 0 to start at beginning of file
) -> crate::Result<LatestLogCursor> {
    get_generic_live_log_cursor(instance_id, "launcher_log.txt", cursor).await
}

#[tracing::instrument]
pub async fn get_generic_live_log_cursor(
    instance_id: &str,
    log_file_name: &str,
    mut cursor: u64, // 0 to start at beginning of file
) -> crate::Result<LatestLogCursor> {
    let state = State::get().await?;
    let instance_path = resolve_instance_path(instance_id, &state).await?;
    let logs_folder = state.directories.instance_logs_dir(&instance_path);
    let path = logs_folder.join(log_file_name);
    if !path.exists() {
        // Allow silent failure if latest.log doesn't exist (as the instance may have been launched, but not yet created the file)
        return Ok(LatestLogCursor {
            cursor: 0,
            new_file: false,
            output: CensoredString("".to_string()),
        });
    }

    let mut file = File::open(&path)
        .await
        .map_err(|e| IOError::with_path(e, &path))?;
    let metadata = file
        .metadata()
        .await
        .map_err(|e| IOError::with_path(e, &path))?;

    let mut new_file = false;
    if cursor > metadata.len() {
        // Cursor is greater than file length, reset cursor to 0
        // Likely cause is that the file was rotated while the log was being read
        cursor = 0;
        new_file = true;
    }

    let mut buffer = Vec::new();
    file.seek(SeekFrom::Start(cursor))
        .map_err(|e| IOError::with_path(e, &path))
        .await?; // Seek to cursor
    let bytes_read = file
        .read_to_end(&mut buffer)
        .map_err(|e| IOError::with_path(e, &path))
        .await?; // Read to end of file
    let output = String::from_utf8_lossy(&buffer); // Convert to String
    let compacted = compact_duplicate_lines(&output);
    let cursor = cursor + bytes_read as u64; // Update cursor

    let credentials = Credentials::get_all(&state.pool)
        .await?
        .into_iter()
        .map(|x| x.1)
        .collect::<Vec<_>>();
    maybe_emit_log_compaction_warning(log_file_name, compacted.stats).await;
    let output = CensoredString::censor(compacted.output, &credentials);
    Ok(LatestLogCursor {
        cursor,
        new_file,
        output,
    })
}
