use crate::install::diagnostics::{
    censor_support_text, write_environment_details, write_latest_log,
};
use crate::util::io::format_bytes;
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::sync::{LazyLock, Mutex};

static PUBLISH_REPORTS: LazyLock<Mutex<HashMap<String, PublishReport>>> =
    LazyLock::new(Default::default);

pub(super) struct PublishReportFile {
    pub(super) file_name: String,
    pub(super) file_type: String,
    pub(super) size: u64,
    pub(super) sha512: String,
    pub(super) executable: bool,
}

pub(super) struct PublishReport {
    instance_id: String,
    started_at: DateTime<Utc>,
    pub(super) instance_name: Option<String>,
    pub(super) shared_instance_id: Option<String>,
    pub(super) applied_version: Option<i32>,
    pub(super) latest_version: Option<i32>,
    pub(super) created_version: Option<i32>,
    pub(super) game_version: Option<String>,
    pub(super) loader: Option<String>,
    pub(super) loader_version: Option<String>,
    pub(super) modpack_id: Option<String>,
    pub(super) selected_paths: Vec<String>,
    pub(super) modrinth_ids: Vec<String>,
    pub(super) files: Vec<PublishReportFile>,
    pub(super) requested_files: Option<HashSet<String>>,
    timeline: Vec<(DateTime<Utc>, String)>,
    error: Option<(&'static str, String)>,
}

impl PublishReport {
    pub(super) fn new(instance_id: &str, selected_paths: &[String]) -> Self {
        let mut report = Self {
            instance_id: instance_id.to_string(),
            started_at: Utc::now(),
            instance_name: None,
            shared_instance_id: None,
            applied_version: None,
            latest_version: None,
            created_version: None,
            game_version: None,
            loader: None,
            loader_version: None,
            modpack_id: None,
            selected_paths: selected_paths.to_vec(),
            modrinth_ids: Vec::new(),
            files: Vec::new(),
            requested_files: None,
            timeline: Vec::new(),
            error: None,
        };
        report.step("Started push");
        report
    }

    pub(super) fn step(&mut self, description: impl Into<String>) {
        self.timeline.push((Utc::now(), description.into()));
    }

    pub(super) fn finish(mut self, result: &crate::Result<()>) {
        match result {
            Ok(()) => self.step("Push succeeded"),
            Err(error) => {
                self.step("Push failed");
                self.error = Some((
                    crate::error::error_code(error),
                    format!("{error:#}"),
                ));
            }
        }
        if let Ok(mut reports) = PUBLISH_REPORTS.lock() {
            reports.insert(self.instance_id.clone(), self);
        }
    }

    fn render(&self) -> String {
        let mut details = String::new();
        let name = self.instance_name.as_deref().unwrap_or("Unknown");
        let _ = writeln!(details, "Shared instance push report: {name}");
        let _ = writeln!(
            details,
            "Result: {}",
            match &self.error {
                Some((code, _)) => format!("failed ({code})"),
                None => "succeeded".to_string(),
            }
        );
        let _ = writeln!(details, "Started: {}", self.started_at.to_rfc3339());
        let _ = writeln!(details, "Instance ID: {}", self.instance_id);
        let _ = writeln!(
            details,
            "Shared instance ID: {}",
            self.shared_instance_id.as_deref().unwrap_or("unknown")
        );
        let _ = writeln!(
            details,
            "Applied version before push: {}",
            optional_version(self.applied_version)
        );
        let _ = writeln!(
            details,
            "Latest version before push: {}",
            optional_version(self.latest_version)
        );
        let _ = writeln!(
            details,
            "Version being pushed: {}",
            optional_version(self.created_version)
        );
        let _ = writeln!(
            details,
            "Minecraft version: {}",
            self.game_version.as_deref().unwrap_or("unknown")
        );
        let _ = writeln!(
            details,
            "Loader: {} {}",
            self.loader.as_deref().unwrap_or("unknown"),
            self.loader_version.as_deref().unwrap_or("")
        );
        let _ = writeln!(
            details,
            "Modpack version ID: {}",
            self.modpack_id.as_deref().unwrap_or("none")
        );

        write_environment_details(&mut details);

        let _ = writeln!(details);
        let _ = writeln!(details, "Timeline");
        for (index, (at, description)) in self.timeline.iter().enumerate() {
            let _ = writeln!(
                details,
                "{}. {} {description}",
                index + 1,
                at.to_rfc3339()
            );
        }

        if let Some((code, message)) = &self.error {
            let _ = writeln!(details);
            let _ = writeln!(details, "Failure");
            let _ = writeln!(details, "Code: {code}");
            let _ = writeln!(details, "Message: {message}");
        }

        let _ = writeln!(details);
        let _ = writeln!(
            details,
            "Modrinth versions ({})",
            self.modrinth_ids.len()
        );
        for version_id in &self.modrinth_ids {
            let _ = writeln!(details, "- {version_id}");
        }

        let _ = writeln!(details);
        let _ =
            writeln!(details, "Selected files ({})", self.selected_paths.len());
        for path in &self.selected_paths {
            let _ = writeln!(details, "- {path}");
        }

        let total_size = self
            .files
            .iter()
            .fold(0_u64, |total, file| total.saturating_add(file.size));
        let _ = writeln!(details);
        let _ = writeln!(
            details,
            "Files ({}, {})",
            self.files.len(),
            format_bytes(total_size)
        );
        if let Some(requested) = &self.requested_files {
            let requested_size = self
                .files
                .iter()
                .filter(|file| requested.contains(&file.file_name))
                .fold(0_u64, |total, file| total.saturating_add(file.size));
            let _ = writeln!(
                details,
                "Requested for upload: {} ({})",
                requested.len(),
                format_bytes(requested_size)
            );
        }
        for file in &self.files {
            let upload = match &self.requested_files {
                Some(requested) if requested.contains(&file.file_name) => {
                    ", upload"
                }
                Some(_) => ", already stored",
                None => "",
            };
            let blocked = if file.executable {
                ", blocked as executable"
            } else {
                ""
            };
            let _ = writeln!(
                details,
                "- {} [{}{upload}{blocked}] {} sha512={}",
                file.file_name,
                file.file_type,
                format_bytes(file.size),
                file.sha512
            );
        }

        details
    }
}

fn optional_version(version: Option<i32>) -> String {
    version.map_or_else(|| "none".to_string(), |version| version.to_string())
}

#[tracing::instrument]
pub async fn get_shared_instance_publish_support_details(
    instance_id: &str,
) -> crate::Result<Option<String>> {
    let state = crate::State::get().await?;
    let details = PUBLISH_REPORTS.lock().ok().and_then(|reports| {
        reports.get(instance_id).map(PublishReport::render)
    });
    let Some(mut details) = details else {
        return Ok(None);
    };
    write_latest_log(&mut details, &state.directories).await;

    Ok(Some(censor_support_text(details, &state.pool).await?))
}
