use crate::event::emit::{emit_process, emit_profile};
use crate::event::{ProcessPayloadType, ProfilePayloadType};
use crate::profile;
use crate::util::io::IOError;
use chrono::{DateTime, TimeZone, Utc};
use dashmap::DashMap;
use quick_xml::Reader;
use quick_xml::events::Event;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use uuid::Uuid;

const LAUNCHER_LOG_PATH: &str = "launcher_log.txt";

pub struct ProcessManager {
    processes: DashMap<Uuid, Process>,
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            processes: DashMap::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn insert_new_process(
        &self,
        profile_path: &str,
        mut mc_command: Command,
        post_exit_command: Option<String>,
        logs_folder: PathBuf,
        xml_logging: bool,
        main_class_keep_alive: TempDir,
        post_process_init: impl AsyncFnOnce(
            &ProcessMetadata,
            &mut ChildStdin,
        ) -> crate::Result<()>,
    ) -> crate::Result<ProcessMetadata> {
        mc_command.stdout(std::process::Stdio::piped());
        mc_command.stderr(std::process::Stdio::piped());
        mc_command.stdin(std::process::Stdio::piped());

        let mut mc_proc = mc_command.spawn().map_err(IOError::from)?;

        let stdout = mc_proc.stdout.take();
        let stderr = mc_proc.stderr.take();

        let mut process = Process {
            metadata: ProcessMetadata {
                uuid: Uuid::new_v4(),
                start_time: Utc::now(),
                profile_path: profile_path.to_string(),
            },
            child: mc_proc,
            _main_class_keep_alive: main_class_keep_alive,
        };

        if let Err(e) = post_process_init(
            &process.metadata,
            &mut process.child.stdin.as_mut().unwrap(),
        )
        .await
        {
            tracing::error!("Failed to run post-process init: {e}");
            let _ = process.child.kill().await;
            return Err(e);
        }

        let metadata = process.metadata.clone();

        if !logs_folder.exists() {
            tokio::fs::create_dir_all(&logs_folder)
                .await
                .map_err(|e| IOError::with_path(e, &logs_folder))?;
        }

        let log_path = logs_folder.join(LAUNCHER_LOG_PATH);

        {
            let mut log_file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&log_path)
                .map_err(|e| IOError::with_path(e, &log_path))?;

            // Initialize with timestamp header
            let now = chrono::Local::now();
            writeln!(
                log_file,
                "# Minecraft launcher log started at {}",
                now.format("%Y-%m-%d %H:%M:%S")
            )
            .map_err(|e| IOError::with_path(e, &log_path))?;
            writeln!(log_file, "# Profile: {profile_path} \n")
                .map_err(|e| IOError::with_path(e, &log_path))?;
            writeln!(log_file).map_err(|e| IOError::with_path(e, &log_path))?;
        }

        if let Some(stdout) = stdout {
            let log_path_clone = log_path.clone();

            let profile_path = metadata.profile_path.clone();
            tokio::spawn(async move {
                Process::process_output(
                    &profile_path,
                    stdout,
                    log_path_clone,
                    xml_logging,
                )
                .await;
            });
        }

        if let Some(stderr) = stderr {
            let log_path_clone = log_path.clone();

            let profile_path = metadata.profile_path.clone();
            tokio::spawn(async move {
                Process::process_output(
                    &profile_path,
                    stderr,
                    log_path_clone,
                    xml_logging,
                )
                .await;
            });
        }

        tokio::spawn(Process::sequential_process_manager(
            profile_path.to_string(),
            post_exit_command,
            metadata.uuid,
        ));

        self.processes.insert(process.metadata.uuid, process);

        emit_process(
            profile_path,
            metadata.uuid,
            ProcessPayloadType::Launched,
            "Launched Minecraft",
        )
        .await?;

        Ok(metadata)
    }

    pub fn get(&self, id: Uuid) -> Option<ProcessMetadata> {
        self.processes.get(&id).map(|x| x.metadata.clone())
    }

    pub fn get_all(&self) -> Vec<ProcessMetadata> {
        self.processes
            .iter()
            .map(|x| x.value().metadata.clone())
            .collect()
    }

    pub fn try_wait(
        &self,
        id: Uuid,
    ) -> crate::Result<Option<Option<ExitStatus>>> {
        if let Some(mut process) = self.processes.get_mut(&id) {
            Ok(Some(process.child.try_wait()?))
        } else {
            Ok(None)
        }
    }

    pub async fn wait_for(&self, id: Uuid) -> crate::Result<()> {
        if let Some(mut process) = self.processes.get_mut(&id) {
            process.child.wait().await?;
        }
        Ok(())
    }

    pub async fn kill(&self, id: Uuid) -> crate::Result<()> {
        if let Some(mut process) = self.processes.get_mut(&id) {
            process.child.kill().await?;
        }

        Ok(())
    }

    fn remove(&self, id: Uuid) {
        self.processes.remove(&id);
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProcessMetadata {
    pub uuid: Uuid,
    pub profile_path: String,
    pub start_time: DateTime<Utc>,
}

#[derive(Debug)]
struct Process {
    metadata: ProcessMetadata,
    child: Child,
    _main_class_keep_alive: TempDir,
}

#[derive(Debug, Default)]
struct Log4jEvent {
    timestamp: Option<String>,
    logger: Option<String>,
    level: Option<String>,
    thread: Option<String>,
    message: Option<String>,
}

impl Process {
    async fn process_output<R>(
        profile_path: &str,
        reader: R,
        log_path: impl AsRef<Path>,
        xml_logging: bool,
    ) where
        R: tokio::io::AsyncRead + Unpin,
    {
        let mut buf_reader = BufReader::new(reader);

        if xml_logging {
            let mut reader = Reader::from_reader(buf_reader);
            reader.config_mut().enable_all_checks(false);

            let mut buf = Vec::new();
            let mut current_event = Log4jEvent::default();
            let mut in_event = false;
            let mut in_message = false;
            let mut in_throwable = false;
            let mut current_content = String::new();

            loop {
                match reader.read_event_into_async(&mut buf).await {
                    Err(e) => {
                        tracing::error!(
                            "Error at position {}: {:?}",
                            reader.buffer_position(),
                            e
                        );
                        break;
                    }
                    // exits the loop when reaching end of file
                    Ok(Event::Eof) => break,

                    Ok(Event::Start(e)) => {
                        match e.name().as_ref() {
                            b"log4j:Event" => {
                                // Reset for new event
                                current_event = Log4jEvent::default();
                                in_event = true;

                                // Extract attributes
                                for attr in e.attributes().flatten() {
                                    let key = String::from_utf8_lossy(
                                        attr.key.into_inner(),
                                    )
                                    .to_string();
                                    let value =
                                        String::from_utf8_lossy(&attr.value)
                                            .to_string();

                                    match key.as_str() {
                                        "logger" => {
                                            current_event.logger = Some(value)
                                        }
                                        "level" => {
                                            current_event.level = Some(value)
                                        }
                                        "thread" => {
                                            current_event.thread = Some(value)
                                        }
                                        "timestamp" => {
                                            current_event.timestamp =
                                                Some(value)
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            b"log4j:Message" => {
                                in_message = true;
                                current_content = String::new();
                            }
                            b"log4j:Throwable" => {
                                in_throwable = true;
                                current_content = String::new();
                            }
                            _ => {}
                        }
                    }
                    Ok(Event::End(e)) => {
                        match e.name().as_ref() {
                            b"log4j:Message" => {
                                in_message = false;
                                current_event.message =
                                    Some(current_content.clone());
                            }
                            b"log4j:Throwable" => {
                                in_throwable = false;
                                // Process and write the log entry
                                let thread = current_event
                                    .thread
                                    .as_deref()
                                    .unwrap_or("");
                                let level = current_event
                                    .level
                                    .as_deref()
                                    .unwrap_or("");
                                let logger = current_event
                                    .logger
                                    .as_deref()
                                    .unwrap_or("");

                                if let Some(message) = &current_event.message {
                                    let formatted_time =
                                        Process::format_timestamp(
                                            current_event.timestamp.as_deref(),
                                        );
                                    let formatted_log = format!(
                                        "{} [{}] [{}{}]: {}\n",
                                        formatted_time,
                                        thread,
                                        if !logger.is_empty() {
                                            format!("{logger}/")
                                        } else {
                                            String::new()
                                        },
                                        level,
                                        message.trim()
                                    );

                                    // Write the log message
                                    if let Err(e) = Process::append_to_log_file(
                                        &log_path,
                                        &formatted_log,
                                    ) {
                                        tracing::error!(
                                            "Failed to write to log file: {}",
                                            e
                                        );
                                    }

                                    // Write the throwable if present
                                    if !current_content.is_empty() {
                                        if let Err(e) =
                                            Process::append_to_log_file(
                                                &log_path,
                                                &current_content,
                                            )
                                        {
                                            tracing::error!(
                                                "Failed to write throwable to log file: {}",
                                                e
                                            );
                                        }
                                    }
                                }
                            }
                            b"log4j:Event" => {
                                in_event = false;
                                // If no throwable was present, write the log entry at the end of the event
                                if current_event.message.is_some()
                                    && !in_throwable
                                {
                                    let thread = current_event
                                        .thread
                                        .as_deref()
                                        .unwrap_or("");
                                    let level = current_event
                                        .level
                                        .as_deref()
                                        .unwrap_or("");
                                    let logger = current_event
                                        .logger
                                        .as_deref()
                                        .unwrap_or("");
                                    let message = current_event
                                        .message
                                        .as_deref()
                                        .unwrap_or("")
                                        .trim();

                                    let formatted_time =
                                        Process::format_timestamp(
                                            current_event.timestamp.as_deref(),
                                        );
                                    let formatted_log = format!(
                                        "{} [{}] [{}{}]: {}\n",
                                        formatted_time,
                                        thread,
                                        if !logger.is_empty() {
                                            format!("{logger}/")
                                        } else {
                                            String::new()
                                        },
                                        level,
                                        message
                                    );

                                    // Write the log message
                                    if let Err(e) = Process::append_to_log_file(
                                        &log_path,
                                        &formatted_log,
                                    ) {
                                        tracing::error!(
                                            "Failed to write to log file: {}",
                                            e
                                        );
                                    }

                                    if let Some(timestamp) =
                                        current_event.timestamp.as_deref()
                                    {
                                        if let Err(e) = Self::maybe_handle_server_join_logging(
                                            profile_path,
                                            timestamp,
                                            message
                                        ).await {
                                            tracing::error!("Failed to handle server join logging: {e}");
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Ok(Event::Text(mut e)) => {
                        if in_message || in_throwable {
                            if let Ok(text) = e.unescape() {
                                current_content.push_str(&text);
                            }
                        } else if !in_event
                            && !e.inplace_trim_end()
                            && !e.inplace_trim_start()
                        {
                            if let Ok(text) = e.unescape() {
                                if let Err(e) = Process::append_to_log_file(
                                    &log_path,
                                    &format!("{text}\n"),
                                ) {
                                    tracing::error!(
                                        "Failed to write to log file: {}",
                                        e
                                    );
                                }
                            }
                        }
                    }
                    Ok(Event::CData(e)) => {
                        if in_message || in_throwable {
                            if let Ok(text) = e
                                .escape()
                                .map_err(|x| x.into())
                                .and_then(|x| x.unescape())
                            {
                                current_content.push_str(&text);
                            }
                        }
                    }
                    _ => (),
                }

                buf.clear();
            }
        } else {
            let mut line = String::new();

            while let Ok(bytes_read) = buf_reader.read_line(&mut line).await {
                if bytes_read == 0 {
                    break; // End of stream
                }

                if !line.is_empty() {
                    if let Err(e) = Self::append_to_log_file(&log_path, &line) {
                        tracing::warn!("Failed to write to log file: {}", e);
                    }
                }

                line.clear();
            }
        }
    }

    fn format_timestamp(timestamp: Option<&str>) -> String {
        if let Some(timestamp_str) = timestamp {
            if let Ok(timestamp_val) = timestamp_str.parse::<i64>() {
                let datetime_utc = if timestamp_val > i32::MAX as i64 {
                    let secs = timestamp_val / 1000;
                    let nsecs = ((timestamp_val % 1000) * 1_000_000) as u32;

                    chrono::DateTime::<Utc>::from_timestamp(secs, nsecs)
                        .unwrap_or_default()
                } else {
                    chrono::DateTime::<Utc>::from_timestamp(timestamp_val, 0)
                        .unwrap_or_default()
                };

                let datetime_local = datetime_utc.with_timezone(&chrono::Local);
                format!("[{}]", datetime_local.format("%H:%M:%S"))
            } else {
                "[??:??:??]".to_string()
            }
        } else {
            "[??:??:??]".to_string()
        }
    }

    fn append_to_log_file(
        path: impl AsRef<Path>,
        line: &str,
    ) -> std::io::Result<()> {
        let mut file =
            OpenOptions::new().append(true).create(true).open(path)?;

        file.write_all(line.as_bytes())?;
        Ok(())
    }

    async fn maybe_handle_server_join_logging(
        profile_path: &str,
        timestamp: &str,
        message: &str,
    ) -> crate::Result<()> {
        let Some(host_port_string) = message.strip_prefix("Connecting to ")
        else {
            return Ok(());
        };
        let Some((host, port_string)) = host_port_string.rsplit_once(", ")
        else {
            return Ok(());
        };
        let Some(port) = port_string.parse::<u16>().ok() else {
            return Ok(());
        };
        let timestamp = timestamp
            .parse::<i64>()
            .map(|x| x / 1000)
            .map_err(|x| {
                crate::ErrorKind::OtherError(format!(
                    "Failed to parse timestamp: {x}"
                ))
            })
            .and_then(|x| {
                Utc.timestamp_opt(x, 0).single().ok_or_else(|| {
                    crate::ErrorKind::OtherError(
                        "Failed to convert timestamp to DateTime".to_string(),
                    )
                })
            })?;

        let state = crate::State::get().await?;
        crate::state::server_join_log::JoinLogEntry {
            profile_path: profile_path.to_owned(),
            host: host.to_string(),
            port,
            join_time: timestamp,
        }
        .upsert(&state.pool)
        .await?;
        {
            let profile_path = profile_path.to_owned();
            let host = host.to_owned();
            tokio::spawn(async move {
                let _ = emit_profile(
                    &profile_path,
                    ProfilePayloadType::ServerJoined {
                        host,
                        port,
                        timestamp,
                    },
                )
                .await;
            });
        }

        Ok(())
    }

    // Spawns a new child process and inserts it into the hashmap
    // Also, as the process ends, it spawns the follow-up process if it exists
    // By convention, ExitStatus is last command's exit status, and we exit on the first non-zero exit status
    async fn sequential_process_manager(
        profile_path: String,
        post_exit_command: Option<String>,
        uuid: Uuid,
    ) -> crate::Result<()> {
        async fn update_playtime(
            last_updated_playtime: &mut DateTime<Utc>,
            profile_path: &str,
            force_update: bool,
        ) {
            let diff = Utc::now()
                .signed_duration_since(*last_updated_playtime)
                .num_seconds();
            if diff >= 60 || force_update {
                if let Err(e) = profile::edit(profile_path, |prof| {
                    prof.recent_time_played += diff as u64;
                    async { Ok(()) }
                })
                .await
                {
                    tracing::warn!(
                        "Failed to update playtime for profile {}: {}",
                        &profile_path,
                        e
                    );
                }
                *last_updated_playtime = Utc::now();
            }
        }

        // Wait on current Minecraft Child
        let mc_exit_status;
        let mut last_updated_playtime = Utc::now();

        let state = crate::State::get().await?;
        loop {
            if let Some(process) = state.process_manager.try_wait(uuid)? {
                if let Some(t) = process {
                    mc_exit_status = t;
                    break;
                }
            } else {
                mc_exit_status = ExitStatus::default();
                break;
            }

            // sleep for 10ms
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

            // Auto-update playtime every minute
            update_playtime(&mut last_updated_playtime, &profile_path, false)
                .await;
        }

        state.process_manager.remove(uuid);
        emit_process(
            &profile_path,
            uuid,
            ProcessPayloadType::Finished,
            "Exited process",
        )
        .await?;

        // Now fully complete- update playtime one last time
        update_playtime(&mut last_updated_playtime, &profile_path, true).await;

        // Publish play time update
        // Allow failure, it will be stored locally and sent next time
        // Sent in another thread as first call may take a couple seconds and hold up process ending
        let profile = profile_path.clone();
        tokio::spawn(async move {
            if let Err(e) = profile::try_update_playtime(&profile).await {
                tracing::warn!(
                    "Failed to update playtime for profile {}: {}",
                    profile,
                    e
                );
            }
        });

        let logs_folder = state.directories.profile_logs_dir(&profile_path);
        let log_path = logs_folder.join(LAUNCHER_LOG_PATH);

        if log_path.exists() {
            if let Err(e) = Process::append_to_log_file(
                &log_path,
                &format!("\n# Process exited with status: {mc_exit_status}\n"),
            ) {
                tracing::warn!(
                    "Failed to write exit status to log file: {}",
                    e
                );
            }
        }

        let _ = state.discord_rpc.clear_to_default(true).await;

        let _ = state.friends_socket.update_status(None).await;

        // If in tauri, window should show itself again after process exists if it was hidden
        #[cfg(feature = "tauri")]
        {
            let window = crate::EventState::get_main_window().await?;
            if let Some(window) = window {
                window.unminimize()?;
                window.set_focus()?;
            }
        }

        if mc_exit_status.success() {
            // We do not wait on the post exist command to finish running! We let it spawn + run on its own.
            // This behaviour may be changed in the future
            if let Some(hook) = post_exit_command {
                let mut cmd = hook.split(' ');
                if let Some(command) = cmd.next() {
                    let mut command = Command::new(command);
                    command.args(cmd).current_dir(
                        profile::get_full_path(&profile_path).await?,
                    );
                    command.spawn().map_err(IOError::from)?;
                }
            }
        }

        Ok(())
    }
}
