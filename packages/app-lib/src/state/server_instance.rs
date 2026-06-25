//! Locally-hosted dedicated Minecraft server instances.
//!
//! Unlike [`Profile`](crate::state::Profile)s, which represent Minecraft
//! *clients* and are tracked in the SQLite database, server instances are
//! intentionally lightweight: each one is a directory under
//! [`DirectoryInfo::servers_dir`](crate::state::DirectoryInfo::servers_dir)
//! holding the server files (jar, world, `server.properties`, ...) alongside a
//! single JSON metadata sidecar. This keeps the feature fully decoupled from
//! the client launch path.

use crate::State;
use crate::util::io::{self, IOError};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::process::Stdio;
use std::sync::LazyLock;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

/// Name of the JSON metadata sidecar stored inside each server directory.
pub const SERVER_METADATA_FILE: &str = "modrinth.server.json";

const SERVER_LOG_CAPACITY: usize = 10_000;

/// The server software a [`ServerInstance`] runs.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServerSoftware {
    /// The official, unmodified Mojang server.
    Vanilla,
    /// PaperMC — a high-performance fork of Spigot with plugin support.
    Paper,
    /// Purpur — a fork of Paper with additional configuration.
    Purpur,
    /// The Fabric mod loader's dedicated server.
    Fabric,
}

impl ServerSoftware {
    /// Whether this software loads Bukkit/Spigot-style plugins (as opposed to
    /// Fabric mods). Used by the UI to decide which content to surface.
    pub fn uses_plugins(&self) -> bool {
        matches!(self, ServerSoftware::Paper | ServerSoftware::Purpur)
    }
}

/// The installation state of a [`ServerInstance`].
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServerInstallStage {
    /// The metadata exists but the server jar has not been downloaded yet.
    NotInstalled,
    /// The server jar is currently being downloaded/set up.
    Installing,
    /// The server is ready to start.
    Installed,
    /// Installation failed; the server should be retried or removed.
    Failed,
}

/// A locally-hosted dedicated server, persisted as a JSON sidecar.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerInstance {
    /// Filesystem-safe identifier; also the directory name.
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    pub software: ServerSoftware,
    pub minecraft_version: String,
    /// The resolved software build/loader version (e.g. the Paper build number
    /// or the Fabric loader version), populated during installation.
    #[serde(default)]
    pub software_version: Option<String>,
    /// The downloaded jar's filename, relative to the server directory.
    #[serde(default)]
    pub jar_file: Option<String>,
    pub install_stage: ServerInstallStage,
    /// Optional Java binary override; falls back to an auto-resolved JRE.
    #[serde(default)]
    pub java_path: Option<String>,
    /// Max heap size in MiB.
    pub memory_mb: u32,
    #[serde(default)]
    pub extra_java_args: Vec<String>,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    #[serde(default)]
    pub last_started: Option<DateTime<Utc>>,
    /// Icon path (absolute), if the user has set one.
    #[serde(default)]
    pub icon_path: Option<String>,
}

impl ServerInstance {
    /// Build a fresh, not-yet-installed server instance.
    pub fn new(
        id: String,
        name: String,
        software: ServerSoftware,
        minecraft_version: String,
        memory_mb: u32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            software,
            minecraft_version,
            software_version: None,
            jar_file: None,
            install_stage: ServerInstallStage::NotInstalled,
            java_path: None,
            memory_mb,
            extra_java_args: Vec::new(),
            created: now,
            modified: now,
            last_started: None,
            icon_path: None,
        }
    }

    /// List all server instances stored on disk, sorted by creation date.
    pub async fn list() -> crate::Result<Vec<ServerInstance>> {
        let state = State::get().await?;
        let dir = state.directories.servers_dir();

        let mut servers = Vec::new();
        if !dir.exists() {
            return Ok(servers);
        }

        let mut entries = io::read_dir(&dir).await?;
        while let Some(entry) =
            entries.next_entry().await.map_err(IOError::from)?
        {
            let metadata_path = entry.path().join(SERVER_METADATA_FILE);
            if !metadata_path.exists() {
                continue;
            }
            match io::read(&metadata_path).await {
                Ok(bytes) => match serde_json::from_slice::<ServerInstance>(
                    &bytes,
                ) {
                    Ok(server) => servers.push(server),
                    Err(e) => tracing::warn!(
                        "Skipping malformed server metadata at {}: {e}",
                        metadata_path.display()
                    ),
                },
                Err(e) => tracing::warn!(
                    "Failed to read server metadata at {}: {e}",
                    metadata_path.display()
                ),
            }
        }

        servers.sort_by_key(|s| s.created);
        Ok(servers)
    }

    /// Fetch a single server instance by id.
    pub async fn get(id: &str) -> crate::Result<Option<ServerInstance>> {
        let state = State::get().await?;
        let metadata_path =
            state.directories.server_dir(id).join(SERVER_METADATA_FILE);
        if !metadata_path.exists() {
            return Ok(None);
        }
        let bytes = io::read(&metadata_path).await?;
        Ok(Some(serde_json::from_slice(&bytes)?))
    }

    /// Like [`Self::get`], but returns a descriptive error if absent.
    pub async fn get_or_err(id: &str) -> crate::Result<ServerInstance> {
        Self::get(id).await?.ok_or_else(|| {
            crate::ErrorKind::InputError(format!(
                "No local server with id {id}"
            ))
            .into()
        })
    }

    /// Persist this instance's metadata to disk, bumping `modified`.
    pub async fn save(&mut self) -> crate::Result<()> {
        let state = State::get().await?;
        self.modified = Utc::now();
        let dir = state.directories.server_dir(&self.id);
        io::create_dir_all(&dir).await?;
        let bytes = serde_json::to_vec_pretty(self)?;
        io::write(dir.join(SERVER_METADATA_FILE), bytes).await?;
        Ok(())
    }

    /// Delete this server instance and all of its files from disk.
    pub async fn remove(id: &str) -> crate::Result<()> {
        if running_server(id).is_some() {
            return Err(crate::ErrorKind::InputError(
                "Cannot delete a server while it is running".to_string(),
            )
            .into());
        }
        let state = State::get().await?;
        let dir = state.directories.server_dir(id);
        if dir.exists() {
            io::remove_dir_all(&dir).await?;
        }
        clear_server_log(id);
        Ok(())
    }
}

/// Information about a currently-running server process.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RunningServerInfo {
    pub id: String,
    pub pid: Option<u32>,
    pub start_time: DateTime<Utc>,
}

struct RunningServer {
    info: RunningServerInfo,
    stdin_tx: mpsc::UnboundedSender<String>,
    kill_tx: mpsc::UnboundedSender<()>,
}

static RUNNING_SERVERS: LazyLock<DashMap<String, RunningServer>> =
    LazyLock::new(DashMap::new);

fn running_server(id: &str) -> Option<RunningServerInfo> {
    RUNNING_SERVERS.get(id).map(|s| s.info.clone())
}

/// Whether the given server currently has a running process.
pub fn is_server_running(id: &str) -> bool {
    RUNNING_SERVERS.contains_key(id)
}

/// Snapshot of all currently-running servers.
pub fn get_running_servers() -> Vec<RunningServerInfo> {
    RUNNING_SERVERS.iter().map(|s| s.info.clone()).collect()
}

/// Send a console command to a running server's standard input.
///
/// Returns `false` if the server is not running.
pub fn send_server_command(id: &str, command: &str) -> bool {
    if let Some(server) = RUNNING_SERVERS.get(id) {
        server.stdin_tx.send(command.to_string()).is_ok()
    } else {
        false
    }
}

/// Force-kill a running server process.
///
/// Returns `false` if the server is not running.
pub fn force_kill_server(id: &str) -> bool {
    if let Some(server) = RUNNING_SERVERS.get(id) {
        server.kill_tx.send(()).is_ok()
    } else {
        false
    }
}

struct ServerLogBuffer {
    lines: VecDeque<String>,
}

static SERVER_LOGS: LazyLock<DashMap<String, ServerLogBuffer>> =
    LazyLock::new(DashMap::new);

fn push_server_log(id: &str, line: String) {
    let mut buffer = SERVER_LOGS
        .entry(id.to_string())
        .or_insert_with(|| ServerLogBuffer {
            lines: VecDeque::new(),
        });
    if buffer.lines.len() >= SERVER_LOG_CAPACITY {
        buffer.lines.pop_front();
    }
    buffer.lines.push_back(line);
}

/// Get the buffered console output for a server.
pub fn get_server_log(id: &str) -> Vec<String> {
    SERVER_LOGS
        .get(id)
        .map(|b| b.lines.iter().cloned().collect())
        .unwrap_or_default()
}

/// Clear the buffered console output for a server.
pub fn clear_server_log(id: &str) {
    SERVER_LOGS.remove(id);
}

/// Spawn a dedicated server process from a prepared [`Command`].
///
/// The command's stdio is overridden so the launcher can stream the console
/// and forward commands. The process is tracked in [`RUNNING_SERVERS`] until it
/// exits.
pub async fn spawn_server_process(
    id: &str,
    mut command: Command,
) -> crate::Result<RunningServerInfo> {
    if is_server_running(id) {
        return Err(crate::ErrorKind::InputError(
            "Server is already running".to_string(),
        )
        .into());
    }

    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn().map_err(IOError::from)?;

    let pid = child.id();
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let stdin = child.stdin.take();

    let info = RunningServerInfo {
        id: id.to_string(),
        pid,
        start_time: Utc::now(),
    };

    let (stdin_tx, mut stdin_rx) = mpsc::unbounded_channel::<String>();
    let (kill_tx, mut kill_rx) = mpsc::unbounded_channel::<()>();

    clear_server_log(id);

    if let Some(stdout) = stdout {
        let id = id.to_string();
        tokio::spawn(read_server_output(id, stdout));
    }
    if let Some(stderr) = stderr {
        let id = id.to_string();
        tokio::spawn(read_server_output(id, stderr));
    }

    if let Some(mut stdin) = stdin {
        tokio::spawn(async move {
            while let Some(line) = stdin_rx.recv().await {
                if stdin.write_all(line.as_bytes()).await.is_err()
                    || stdin.write_all(b"\n").await.is_err()
                    || stdin.flush().await.is_err()
                {
                    break;
                }
            }
        });
    }

    RUNNING_SERVERS.insert(
        id.to_string(),
        RunningServer {
            info: info.clone(),
            stdin_tx,
            kill_tx,
        },
    );

    emit_server_event(id, ServerProcessEventType::Launched);

    let monitor_id = id.to_string();
    tokio::spawn(async move {
        tokio::select! {
            status = child.wait() => {
                if let Ok(status) = status {
                    push_server_log(
                        &monitor_id,
                        format!("[Modrinth] Server process exited ({status})."),
                    );
                }
            }
            _ = kill_rx.recv() => {
                let _ = child.kill().await;
                let _ = child.wait().await;
                push_server_log(
                    &monitor_id,
                    "[Modrinth] Server process was forcibly stopped."
                        .to_string(),
                );
            }
        }

        RUNNING_SERVERS.remove(&monitor_id);
        emit_server_event(&monitor_id, ServerProcessEventType::Stopped);
    });

    Ok(info)
}

async fn read_server_output<R>(id: String, reader: R)
where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();
    loop {
        line.clear();
        match buf_reader.read_line(&mut line).await {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                let trimmed = line.trim_end().to_string();
                push_server_log(&id, trimmed.clone());
                emit_server_log(&id, &trimmed);
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ServerProcessEventType {
    Launched,
    Stopped,
}

#[cfg(feature = "tauri")]
fn emit_server_log(id: &str, line: &str) {
    use tauri::Emitter;
    if let Ok(event_state) = crate::EventState::get() {
        let _ = event_state.app.emit(
            "server_log",
            serde_json::json!({ "server_id": id, "line": line }),
        );
    }
}

#[cfg(not(feature = "tauri"))]
fn emit_server_log(_id: &str, _line: &str) {}

#[cfg(feature = "tauri")]
fn emit_server_event(id: &str, event: ServerProcessEventType) {
    use tauri::Emitter;
    let event = match event {
        ServerProcessEventType::Launched => "launched",
        ServerProcessEventType::Stopped => "stopped",
    };
    if let Ok(event_state) = crate::EventState::get() {
        let _ = event_state.app.emit(
            "server_process",
            serde_json::json!({ "server_id": id, "event": event }),
        );
    }
}

#[cfg(not(feature = "tauri"))]
fn emit_server_event(_id: &str, _event: ServerProcessEventType) {}
