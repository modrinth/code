use super::model::{
    InstallCleanup, InstallInterruptReason, InstallJobEvent,
    InstallJobEventKind, InstallJobSnapshot, InstallJobState, InstallJobStatus,
    InstallPhaseDetails, InstallPhaseId, InstallProgress,
};
use super::store;
use crate::state::{ModrinthCredentials, State};
use regex::{Captures, Regex};
use std::fmt::Write as _;
use std::io::{Read, Seek, SeekFrom};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const INSTALL_SUPPORT_LOG_TAIL_BYTES: u64 = 128 * 1024;

pub async fn build_job_support_details(
    job: &store::InstallJobRecord,
    state: &State,
) -> crate::Result<String> {
    let snapshot = job.snapshot();
    let mut details = String::new();
    let title = snapshot
        .display
        .as_ref()
        .map(|display| display.title.as_str())
        .unwrap_or("Unknown");

    let _ = writeln!(details, "Install report: {title}");
    let _ =
        writeln!(details, "Result: {}", result_summary(&snapshot, &job.state));
    let _ = writeln!(details, "Job ID: {}", snapshot.job_id);
    let _ = writeln!(details, "Request: {}", json_string(&snapshot.kind));
    let _ = writeln!(details, "Status: {}", json_string(&snapshot.status));
    let _ = writeln!(details, "Current phase: {}", phase_label(snapshot.phase));
    if let Some(progress) = &snapshot.progress {
        let _ = writeln!(
            details,
            "Current progress: {}",
            progress_summary(progress)
        );
    }

    write_environment_details(&mut details);
    write_timeline(&mut details, &job.state.events);
    write_content_summary(&mut details, &job.state.events);
    write_errors(&mut details, &snapshot);
    write_raw_snapshot(&mut details, &snapshot);
    write_latest_log(&mut details, state).await;

    censor_support_text(details, state).await
}

fn result_summary(
    snapshot: &InstallJobSnapshot,
    state: &InstallJobState,
) -> String {
    match snapshot.status {
        InstallJobStatus::Queued => "queued".to_string(),
        InstallJobStatus::Running => {
            format!("running while {}", phase_label(snapshot.phase))
        }
        InstallJobStatus::Succeeded => "succeeded".to_string(),
        InstallJobStatus::Canceled => snapshot
            .error
            .as_ref()
            .and_then(|error| error.phase)
            .map(|phase| format!("canceled while {}", phase_label(phase)))
            .unwrap_or_else(|| "canceled".to_string()),
        InstallJobStatus::Failed => snapshot
            .error
            .as_ref()
            .and_then(|error| {
                error.phase.map(|phase| {
                    format!(
                        "failed while {} ({})",
                        phase_label(phase),
                        error.code
                    )
                })
            })
            .unwrap_or_else(|| "failed".to_string()),
        InstallJobStatus::Interrupted => latest_interruption(&state.events)
            .map(|(reason, phase)| match reason {
                InstallInterruptReason::AppClosed => format!(
                    "interrupted because the app closed while {}",
                    phase_label(phase)
                ),
                InstallInterruptReason::Unknown => {
                    format!("interrupted while {}", phase_label(phase))
                }
            })
            .unwrap_or_else(|| "interrupted".to_string()),
    }
}

fn write_environment_details(details: &mut String) {
    let _ = writeln!(details);
    let _ = writeln!(details, "Environment");
    let _ = writeln!(details, "App version: {}", env!("CARGO_PKG_VERSION"));
    let _ = writeln!(
        details,
        "OS: {}",
        sysinfo::System::long_os_version()
            .or_else(sysinfo::System::name)
            .unwrap_or_else(|| std::env::consts::OS.to_string())
    );
    let _ = writeln!(details, "OS kind: {}", std::env::consts::OS);
    let _ = writeln!(details, "OS family: {}", std::env::consts::FAMILY);
    let _ = writeln!(details, "Architecture: {}", std::env::consts::ARCH);
    if let Some(kernel_version) = sysinfo::System::kernel_version() {
        let _ = writeln!(details, "Kernel: {kernel_version}");
    }
}

fn latest_interruption(
    events: &[InstallJobEvent],
) -> Option<(InstallInterruptReason, InstallPhaseId)> {
    events.iter().rev().find_map(|event| match &event.kind {
        InstallJobEventKind::Interrupted { reason, phase } => {
            Some((*reason, *phase))
        }
        _ => None,
    })
}

fn write_timeline(details: &mut String, events: &[InstallJobEvent]) {
    let _ = writeln!(details);
    let _ = writeln!(details, "Timeline");

    let mut index = 1;
    for event in events {
        let Some(description) = timeline_event_description(event) else {
            continue;
        };
        let _ = writeln!(
            details,
            "{index}. {} {description}",
            event.at.to_rfc3339()
        );
        index += 1;
    }

    if index == 1 {
        let _ = writeln!(details, "No install events were recorded.");
    }
}

fn timeline_event_description(event: &InstallJobEvent) -> Option<String> {
    match &event.kind {
        InstallJobEventKind::JobQueued { kind } => {
            Some(format!("Queued {} install", json_string(kind)))
        }
        InstallJobEventKind::JobStarted => {
            Some("Started install job".to_string())
        }
        InstallJobEventKind::JobSucceeded { instance_id } => {
            Some(match instance_id {
                Some(instance_id) => {
                    format!("Finished install for instance {instance_id}")
                }
                None => "Finished install".to_string(),
            })
        }
        InstallJobEventKind::JobCanceled { phase } => {
            Some(format!("Canceled while {}", phase_label(*phase)))
        }
        InstallJobEventKind::PhaseStarted { phase, details } => Some(format!(
            "Started {}{}",
            phase_label(*phase),
            phase_details_suffix(details)
        )),
        InstallJobEventKind::Interrupted { reason, phase } => {
            Some(match reason {
                InstallInterruptReason::AppClosed => {
                    format!("App closed while {}", phase_label(*phase))
                }
                InstallInterruptReason::Unknown => {
                    format!("Interrupted while {}", phase_label(*phase))
                }
            })
        }
        InstallJobEventKind::Failed {
            phase,
            code,
            message,
        } => Some(format!(
            "Failed while {} ({code}): {message}",
            phase_label(*phase)
        )),
        InstallJobEventKind::RollbackStarted { cleanup } => {
            Some(format!("Started rollback ({})", cleanup_summary(cleanup)))
        }
        InstallJobEventKind::RollbackCompleted => {
            Some("Rollback completed".to_string())
        }
        InstallJobEventKind::RollbackFailed { message } => {
            Some(format!("Rollback failed: {message}"))
        }
        InstallJobEventKind::ContentDownloadStarted { .. }
        | InstallJobEventKind::ContentFileSkipped { .. }
        | InstallJobEventKind::ContentFileCompleted { .. } => None,
    }
}

fn write_content_summary(details: &mut String, events: &[InstallJobEvent]) {
    let started = events.iter().rev().find_map(|event| match &event.kind {
        InstallJobEventKind::ContentDownloadStarted { files, bytes } => {
            Some((*files, *bytes))
        }
        _ => None,
    });
    let completed = events
        .iter()
        .filter_map(|event| match &event.kind {
            InstallJobEventKind::ContentFileCompleted { path, bytes } => {
                Some((path.as_str(), *bytes))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let skipped = events
        .iter()
        .filter_map(|event| match &event.kind {
            InstallJobEventKind::ContentFileSkipped { path, reason } => {
                Some((path.as_str(), reason.as_str()))
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    if started.is_none() && completed.is_empty() && skipped.is_empty() {
        return;
    }

    let _ = writeln!(details);
    let _ = writeln!(details, "Content activity");
    if let Some((files, bytes)) = started {
        let _ = writeln!(
            details,
            "Completed files: {} / {files}, skipped files: {}",
            completed.len(),
            skipped.len()
        );
        if let Some(bytes) = bytes {
            let _ = writeln!(
                details,
                "Expected content size: {}",
                format_bytes(bytes)
            );
        }
    } else {
        let _ = writeln!(
            details,
            "Completed files: {}, skipped files: {}",
            completed.len(),
            skipped.len()
        );
    }

    if !completed.is_empty() {
        let _ = writeln!(details);
        let _ = writeln!(details, "Recently completed files");
        for (path, bytes) in completed.iter().rev().take(20) {
            let _ = writeln!(details, "- {path} ({})", format_bytes(*bytes));
        }
        if completed.len() > 20 {
            let _ = writeln!(details, "- ... {} more", completed.len() - 20);
        }
    }

    if !skipped.is_empty() {
        let _ = writeln!(details);
        let _ = writeln!(details, "Skipped files");
        for (path, reason) in skipped.iter().rev().take(20) {
            let _ = writeln!(details, "- {path} ({reason})");
        }
        if skipped.len() > 20 {
            let _ = writeln!(details, "- ... {} more", skipped.len() - 20);
        }
    }
}

fn write_errors(details: &mut String, snapshot: &InstallJobSnapshot) {
    if let Some(error) = &snapshot.error {
        let _ = writeln!(details);
        let _ = writeln!(details, "Failure");
        let _ = writeln!(details, "Code: {}", error.code);
        if let Some(phase) = error.phase {
            let _ = writeln!(details, "Phase: {}", phase_label(phase));
        }
        let _ = writeln!(details, "Message: {}", error.message);
        write_api_error_details(details, error);
        write_error_context(details, error);
    }

    if let Some(error) = &snapshot.rollback_error {
        let _ = writeln!(details);
        let _ = writeln!(details, "Rollback error");
        let _ = writeln!(details, "Code: {}", error.code);
        if let Some(phase) = error.phase {
            let _ = writeln!(details, "Phase: {}", phase_label(phase));
        }
        let _ = writeln!(details, "Message: {}", error.message);
        write_api_error_details(details, error);
        write_error_context(details, error);
    }
}

fn write_api_error_details(
    details: &mut String,
    error: &super::model::InstallErrorView,
) {
    let Some(api) = &error.api else {
        return;
    };

    let _ = writeln!(details, "API error: {}", api.error);
    if let Some(status) = api.status {
        let _ = writeln!(details, "HTTP status: {status}");
    }
    if api.method.is_some() || api.url.is_some() {
        let method = api.method.as_deref().unwrap_or("unknown method");
        let url = api.url.as_deref().unwrap_or("unknown URL");
        let _ = writeln!(details, "Request: {method} {url}");
    }
    if let Some(route) = &api.route {
        let _ = writeln!(details, "Route: {route}");
    }
}

fn write_error_context(
    details: &mut String,
    error: &super::model::InstallErrorView,
) {
    let Some(context) = &error.context else {
        return;
    };

    let _ = writeln!(details, "Operation: {}", context.operation);
    if let Some(source_path) = &context.source_path {
        let _ = writeln!(details, "Source path: {source_path}");
    }
    if let Some(target_path) = &context.target_path {
        let _ = writeln!(details, "Target path: {target_path}");
    }
    if let Some(file_path) = &context.file_path {
        let _ = writeln!(details, "File path: {file_path}");
    }
    if let Some(entry_path) = &context.entry_path {
        let _ = writeln!(details, "Archive entry: {entry_path}");
    }
    if !context.urls.is_empty() {
        let _ = writeln!(details, "URLs:");
        for url in &context.urls {
            let _ = writeln!(details, "- {url}");
        }
    }
    if let Some(expected_hash) = &context.expected_hash {
        let _ = writeln!(details, "Expected hash: {expected_hash}");
    }
    if let Some(expected_size) = context.expected_size {
        let _ =
            writeln!(details, "Expected size: {}", format_bytes(expected_size));
    }
    if let Some(project_id) = &context.project_id {
        let _ = writeln!(details, "Project ID: {project_id}");
    }
    if let Some(version_id) = &context.version_id {
        let _ = writeln!(details, "Version ID: {version_id}");
    }
    if let Some(minecraft_version) = &context.minecraft_version {
        let _ = writeln!(details, "Minecraft version: {minecraft_version}");
    }
    if let Some(loader) = &context.loader {
        let _ = writeln!(details, "Loader: {loader}");
    }
    if let Some(java_version) = context.java_version {
        let _ = writeln!(details, "Java version: {java_version}");
    }
    if let Some(os) = &context.os {
        let _ = writeln!(details, "OS: {os}");
    }
    if let Some(arch) = &context.arch {
        let _ = writeln!(details, "Architecture: {arch}");
    }
}

fn write_raw_snapshot(details: &mut String, snapshot: &InstallJobSnapshot) {
    let _ = writeln!(details);
    let _ = writeln!(details, "Raw snapshot");
    match serde_json::to_string_pretty(snapshot) {
        Ok(snapshot_json) => {
            let _ = writeln!(details, "{snapshot_json}");
        }
        Err(error) => {
            let _ = writeln!(details, "Unable to serialize snapshot: {error}");
        }
    }
}

async fn write_latest_log(details: &mut String, state: &State) {
    let _ = writeln!(details);
    let _ = writeln!(details, "Latest launcher log excerpt");
    match latest_launcher_log_tail(state).await {
        Ok(Some((path, output))) => {
            let _ = writeln!(details, "File: {}", path.display());
            details.push_str(&output);
        }
        Ok(None) => {
            let _ = writeln!(details, "No launcher log found.");
        }
        Err(error) => {
            let _ = writeln!(details, "Unable to read launcher log: {error}");
        }
    }
}

fn phase_label(phase: InstallPhaseId) -> &'static str {
    match phase {
        InstallPhaseId::PreparingInstance => "preparing instance",
        InstallPhaseId::ResolvingPack => "resolving pack",
        InstallPhaseId::DownloadingPackFile => "downloading pack file",
        InstallPhaseId::ReadingPackManifest => "reading pack manifest",
        InstallPhaseId::DownloadingContent => "downloading content",
        InstallPhaseId::ExtractingOverrides => "extracting overrides",
        InstallPhaseId::ResolvingMinecraft => "resolving Minecraft",
        InstallPhaseId::ResolvingLoader => "resolving loader",
        InstallPhaseId::PreparingJava => "preparing Java",
        InstallPhaseId::DownloadingMinecraft => "downloading Minecraft",
        InstallPhaseId::RunningLoaderProcessors => "running loader processors",
        InstallPhaseId::Finalizing => "finalizing",
        InstallPhaseId::RollingBack => "rolling back",
    }
}

fn phase_details_suffix(details: &InstallPhaseDetails) -> String {
    match details {
        InstallPhaseDetails::Empty => String::new(),
        InstallPhaseDetails::Instance { name } => format!(" for {name}"),
        InstallPhaseDetails::Minecraft {
            game_version,
            loader,
        } => format!(
            " for Minecraft {game_version} with {}",
            json_string(loader)
        ),
        InstallPhaseDetails::Java {
            major_version,
            step,
        } => format!(": {} Java {major_version}", json_string(step)),
        InstallPhaseDetails::Modpack {
            project_id,
            version_id,
            title,
        } => {
            let mut value = title
                .as_ref()
                .map(|title| format!(" for {title}"))
                .unwrap_or_default();
            if let Some(project_id) = project_id {
                let _ = write!(value, " project={project_id}");
            }
            if let Some(version_id) = version_id {
                let _ = write!(value, " version={version_id}");
            }
            value
        }
        InstallPhaseDetails::Import {
            launcher_type,
            instance_folder,
        } => format!(" from {launcher_type} instance {instance_folder}"),
    }
}

fn cleanup_summary(cleanup: &InstallCleanup) -> String {
    match cleanup {
        InstallCleanup::DeleteNewInstance { instance_id } => {
            match instance_id {
                Some(instance_id) => {
                    format!("delete partially-created instance {instance_id}")
                }
                None => "delete partially-created instance".to_string(),
            }
        }
        InstallCleanup::RestoreExistingInstance { instance_id } => {
            format!("restore existing instance {instance_id}")
        }
    }
}

fn progress_summary(progress: &InstallProgress) -> String {
    let mut value = format!("{} / {}", progress.current, progress.total);
    if let Some(secondary) = &progress.secondary {
        let _ = write!(
            value,
            " ({} / {})",
            format_bytes(secondary.current),
            format_bytes(secondary.total)
        );
    }
    value
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = UNITS[0];
    for next_unit in UNITS.iter().skip(1) {
        if value < 1024.0 {
            break;
        }
        value /= 1024.0;
        unit = next_unit;
    }

    if unit == "B" {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {unit}")
    }
}

fn json_string<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value)
        .map(|value| value.trim_matches('"').to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

async fn latest_launcher_log_tail(
    state: &State,
) -> crate::Result<Option<(PathBuf, String)>> {
    let Some(logs_dir) = state.directories.launcher_logs_dir() else {
        return Ok(None);
    };

    let entries = match std::fs::read_dir(&logs_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(None);
        }
        Err(error) => return Err(error.into()),
    };

    let mut latest: Option<(PathBuf, SystemTime)> = None;
    for entry in entries {
        let entry = entry?;
        let metadata = match entry.metadata() {
            Ok(metadata) if metadata.is_file() => metadata,
            _ => continue,
        };
        let modified = metadata
            .modified()
            .or_else(|_| metadata.created())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        let path = entry.path();

        match latest.as_ref() {
            Some((_, latest_modified)) if modified <= *latest_modified => {}
            _ => latest = Some((path, modified)),
        }
    }

    let Some((path, _)) = latest else {
        return Ok(None);
    };

    let output = read_file_tail(&path, INSTALL_SUPPORT_LOG_TAIL_BYTES)?;
    Ok(Some((path, output)))
}

fn read_file_tail(path: &Path, max_bytes: u64) -> crate::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let len = file.metadata()?.len();
    let start = len.saturating_sub(max_bytes);
    file.seek(SeekFrom::Start(start))?;

    let mut buffer = Vec::with_capacity((len - start) as usize);
    file.read_to_end(&mut buffer)?;

    let mut output = String::from_utf8_lossy(&buffer).into_owned();
    if start > 0 {
        output = format!("[first {start} bytes omitted]\n{output}");
    }

    Ok(output)
}

async fn censor_support_text(
    mut text: String,
    state: &State,
) -> crate::Result<String> {
    for credentials in ModrinthCredentials::get_all(&state.pool)
        .await?
        .into_iter()
        .map(|credentials| credentials.1)
    {
        replace_nonempty(
            &mut text,
            &credentials.session,
            "{MODRINTH_ACCESS_TOKEN}",
        );
    }

    text = censor_ip_addresses(text);

    Ok(text)
}

fn replace_nonempty(text: &mut String, value: &str, replacement: &str) {
    if !value.is_empty() {
        *text = text.replace(value, replacement);
    }
}

fn censor_ip_addresses(text: String) -> String {
    let text = Regex::new(
        r"\b(?:(?:25[0-5]|2[0-4]\d|1?\d?\d)\.){3}(?:25[0-5]|2[0-4]\d|1?\d?\d)\b",
    )
    .expect("valid IPv4 regex")
    .replace_all(&text, |captures: &Captures<'_>| {
        let value = &captures[0];
        match value.parse::<Ipv4Addr>() {
            Ok(_) => "...".to_string(),
            _ => value.to_string(),
        }
    })
    .into_owned();

    Regex::new(r"(?i)\b[0-9a-f:.%]{3,}\b")
        .expect("valid IPv6 candidate regex")
        .replace_all(&text, |captures: &Captures<'_>| {
            let value = &captures[0];
            if value.matches(':').count() < 2 {
                return value.to_string();
            }

            let candidate = value.split('%').next().unwrap_or(value);
            match candidate.parse::<Ipv6Addr>() {
                Ok(_) => ":::::::".to_string(),
                _ => value.to_string(),
            }
        })
        .into_owned()
}
