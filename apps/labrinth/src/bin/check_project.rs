use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs::File;
use std::io::{BufReader, IsTerminal};
use std::path::{Path, PathBuf};

use clap::Parser;
use eyre::{Result, WrapErr, eyre};
use labrinth::models::projects::{Project, Version};
use labrinth::validate::project::{
    ProjectNagKind, ProjectNagSeverity, is_project_description_non_english,
    is_project_summary_non_english, validate,
};
use serde::de::{
    DeserializeOwned, DeserializeSeed, IgnoredAny, MapAccess, SeqAccess,
    Visitor,
};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use url::Url;

#[path = "check_project/profanity.rs"]
mod profanity;

const API_BASE_URL: &str = "https://api.modrinth.com/v3/";

#[derive(Parser)]
#[command(
    version,
    about = "Validate a project from the Modrinth API or a project dataset"
)]
struct Args {
    /// The ID of the project to validate
    #[arg(
		value_name = "PROJECT_ID",
		required_unless_present_any = ["file", "read_profanity_report"],
		conflicts_with_all = ["file", "read_profanity_report"]
	)]
    project_id: Option<String>,

    /// Validate all projects in a moderation project dataset
    #[arg(
		long,
		value_name = "PATH",
		conflicts_with_all = ["token", "read_profanity_report"]
	)]
    file: Option<PathBuf>,

    /// Print every validator kind with the IDs of projects that trigger it
    #[arg(
		long,
		requires = "file",
		conflicts_with_all = [
			"nag_project_ids",
			"description_language_project_ids",
			"show_description_profanity",
			"write_profanity_report"
		]
	)]
    all_nag_project_ids: bool,

    /// A Modrinth token to send as bearer authentication
    #[arg(long, value_name = "TOKEN")]
    token: Option<String>,

    /// Show this many descriptions flagged for profanity
    #[arg(long, value_name = "COUNT", default_value_t = 0, requires = "file")]
    show_description_profanity: usize,

    /// Print project IDs triggering the summary language nag
    #[arg(long, requires = "file")]
    nag_project_ids: bool,

    /// Print project IDs triggering the description language nag
    #[arg(long, requires = "file", conflicts_with = "nag_project_ids")]
    description_language_project_ids: bool,

    /// Scan every profanity-bearing field into a reusable JSON report
    #[arg(
        long,
        value_name = "PATH",
        requires = "file",
        conflicts_with = "show_description_profanity"
    )]
    write_profanity_report: Option<PathBuf>,

    /// Render a previously generated profanity report
    #[arg(long, value_name = "PATH", conflicts_with = "file")]
    read_profanity_report: Option<PathBuf>,

    /// Render at most this many projects from a profanity report (0 means all)
    #[arg(
        long,
        value_name = "COUNT",
        default_value_t = 0,
        requires = "read_profanity_report"
    )]
    profanity_report_limit: usize,

    /// Use bracket markers instead of ANSI colors when rendering a report
    #[arg(long, requires = "read_profanity_report")]
    profanity_report_brackets: bool,
}

#[derive(Serialize)]
struct BatchSummary {
    projects: usize,
    projects_with_at_least_one_required_nag: usize,
    version_details_available: bool,
    nag_counts: BTreeMap<ProjectNagKind, usize>,
    #[serde(skip)]
    description_profanity_sample_limit: usize,
    #[serde(skip)]
    description_profanity_samples: Vec<DescriptionProfanitySample>,
}

struct DescriptionProfanitySample {
    id: String,
    name: String,
    description: String,
}

impl BatchSummary {
    fn new(description_profanity_sample_limit: usize) -> Self {
        Self {
            projects: 0,
            projects_with_at_least_one_required_nag: 0,
            version_details_available: false,
            nag_counts: ProjectNagKind::iter().map(|kind| (kind, 0)).collect(),
            description_profanity_sample_limit,
            description_profanity_samples: Vec::new(),
        }
    }

    fn add_project(&mut self, project: &Project) {
        let nags = validate(project, &[]);
        self.projects += 1;
        if nags
            .iter()
            .any(|nag| nag.severity == ProjectNagSeverity::Required)
        {
            self.projects_with_at_least_one_required_nag += 1;
        }
        if self.description_profanity_samples.len()
            < self.description_profanity_sample_limit
            && nags.iter().any(|nag| {
                matches!(
                    nag.kind,
                    ProjectNagKind::ProjectDescriptionSlur
                        | ProjectNagKind::ProjectDescriptionProfanity
                )
            })
        {
            self.description_profanity_samples.push(
                DescriptionProfanitySample {
                    id: project.id.to_string(),
                    name: project.name.clone(),
                    description: project.description.clone(),
                },
            );
        }
        for nag in nags {
            *self.nag_counts.entry(nag.kind).or_default() += 1;
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install().wrap_err("installing color-eyre")?;
    let args = Args::parse();

    if let Some(report_path) = args.read_profanity_report.as_deref() {
        return profanity::render_report(
            report_path,
            args.profanity_report_limit,
            args.profanity_report_brackets,
        );
    }

    if let Some(report_path) = args.write_profanity_report.as_deref() {
        let dataset_path = args.file.as_deref().ok_or_else(|| {
            eyre!("`--write-profanity-report` requires `--file`")
        })?;
        return profanity::write_report(dataset_path, report_path);
    }

    if let Some(path) = args.file.as_deref() {
        if args.nag_project_ids {
            return print_json(&find_language_project_ids(
                path,
                ProjectNagKind::ProjectSummaryNonEnglish,
                is_project_summary_non_english,
            )?);
        }
        if args.description_language_project_ids {
            return print_json(&find_language_project_ids(
                path,
                ProjectNagKind::ProjectDescriptionNonEnglish,
                is_project_description_non_english,
            )?);
        }
        if args.all_nag_project_ids {
            return print_json(&find_all_nag_project_ids(path)?);
        }
        let summary = summarize_file(path, args.show_description_profanity)?;
        print_description_profanity_samples(
            &summary.description_profanity_samples,
        );
        print_json(&summary)?;
        return Ok(());
    }

    let project_id = args
        .project_id
        .as_deref()
        .ok_or_else(|| eyre!("a project ID or `--file` is required"))?;
    check_api_project(project_id, args.token.as_deref()).await
}

#[derive(Deserialize)]
struct ProjectDataset {
    projects: Vec<Project>,
}

fn find_all_nag_project_ids(
    path: &Path,
) -> Result<BTreeMap<ProjectNagKind, Vec<String>>> {
    let file = File::open(path)
        .wrap_err_with(|| format!("opening `{}`", path.display()))?;
    let dataset: ProjectDataset = serde_json::from_reader(BufReader::new(file))
        .wrap_err_with(|| format!("reading `{}`", path.display()))?;
    let worker_count = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1);
    let chunk_size = dataset.projects.len().div_ceil(worker_count).max(1);
    let worker_results = std::thread::scope(|scope| {
        let workers = dataset
            .projects
            .chunks(chunk_size)
            .map(|projects| {
                scope.spawn(move || {
                    let mut project_ids =
                        BTreeMap::<ProjectNagKind, Vec<String>>::new();
                    for project in projects {
                        let kinds = validate(project, &[])
                            .into_iter()
                            .map(|nag| nag.kind)
                            .collect::<BTreeSet<_>>();
                        for kind in kinds {
                            project_ids
                                .entry(kind)
                                .or_default()
                                .push(project.id.to_string());
                        }
                    }
                    project_ids
                })
            })
            .collect::<Vec<_>>();
        workers
            .into_iter()
            .map(|worker| {
                worker
                    .join()
                    .map_err(|_| eyre!("project validation worker panicked"))
            })
            .collect::<Result<Vec<_>>>()
    })?;

    let mut project_ids = ProjectNagKind::iter()
        .map(|kind| (kind, Vec::new()))
        .collect::<BTreeMap<_, _>>();
    for worker_result in worker_results {
        for (kind, ids) in worker_result {
            project_ids.entry(kind).or_default().extend(ids);
        }
    }

    Ok(project_ids)
}

fn find_language_project_ids(
    path: &Path,
    kind: ProjectNagKind,
    is_non_english: fn(&Project) -> bool,
) -> Result<BTreeMap<ProjectNagKind, Vec<String>>> {
    let file = File::open(path)
        .wrap_err_with(|| format!("opening `{}`", path.display()))?;
    let dataset: ProjectDataset = serde_json::from_reader(BufReader::new(file))
        .wrap_err_with(|| format!("reading `{}`", path.display()))?;
    let worker_count = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1);
    let chunk_size = dataset.projects.len().div_ceil(worker_count).max(1);
    let ids = std::thread::scope(|scope| -> Result<Vec<String>> {
        let workers = dataset
            .projects
            .chunks(chunk_size)
            .map(|projects| {
                scope.spawn(move || {
                    projects
                        .iter()
                        .filter(|project| is_non_english(project))
                        .map(|project| project.id.to_string())
                        .collect::<Vec<_>>()
                })
            })
            .collect::<Vec<_>>();
        workers
            .into_iter()
            .map(|worker| {
                worker
                    .join()
                    .map_err(|_| eyre!("language detection worker panicked"))
            })
            .collect::<Result<Vec<_>>>()
            .map(|ids| ids.into_iter().flatten().collect())
    })?;
    let project_ids = BTreeMap::from([(kind, ids)]);

    Ok(project_ids)
}

async fn check_api_project(
    project_id: &str,
    token: Option<&str>,
) -> Result<()> {
    let client = reqwest::Client::builder()
        .user_agent(concat!(
            "labrinth-check-project/",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .wrap_err("building HTTP client")?;

    let project_url = api_url(&["project", project_id])?;
    let mut versions_url = api_url(&["project", project_id, "version"])?;
    versions_url
        .query_pairs_mut()
        .append_pair("include_changelog", "false");

    let (project, versions) = tokio::try_join!(
        fetch::<Project>(&client, project_url, token),
        fetch::<Vec<Version>>(&client, versions_url, token),
    )?;
    print_json(&validate(&project, &versions))
}

fn summarize_file(
    path: &Path,
    description_profanity_sample_limit: usize,
) -> Result<BatchSummary> {
    let file = File::open(path)
        .wrap_err_with(|| format!("opening `{}`", path.display()))?;
    let mut deserializer =
        serde_json::Deserializer::from_reader(BufReader::new(file));
    let summary = DatasetSeed {
        description_profanity_sample_limit,
    }
    .deserialize(&mut deserializer)
    .wrap_err_with(|| format!("reading `{}`", path.display()))?;
    deserializer
        .end()
        .wrap_err_with(|| format!("reading `{}`", path.display()))?;
    Ok(summary)
}

fn print_description_profanity_samples(samples: &[DescriptionProfanitySample]) {
    if samples.is_empty() {
        return;
    }

    let use_color = std::io::stderr().is_terminal()
        && std::env::var_os("NO_COLOR").is_none();
    eprintln!("\n=== Description profanity samples ===");
    if use_color {
        eprintln!(
            "Detected text is shown with a bold white-on-red background."
        );
    } else {
        eprintln!("Detected text is enclosed in ⟦double brackets⟧.");
    }

    for (index, sample) in samples.iter().enumerate() {
        eprintln!("\n--- Sample {} of {} ---", index + 1, samples.len());
        eprintln!("Project: {}", sample.name);
        eprintln!("ID:      {}", sample.id);
        eprintln!("URL:     https://modrinth.com/project/{}", sample.id);
        eprintln!(
            "\n{}",
            profanity::highlight_text(&sample.description, use_color)
        );
    }
    eprintln!("\n=== End description profanity samples ===\n");
}

fn print_json(value: &impl Serialize) -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(value).wrap_err("serializing output")?
    );
    Ok(())
}

fn api_url(path: &[&str]) -> Result<Url> {
    let mut url = Url::parse(API_BASE_URL).wrap_err("parsing API base URL")?;
    url.path_segments_mut()
        .map_err(|_| eyre!("API base URL cannot be a base URL"))?
        .extend(path);
    Ok(url)
}

async fn fetch<T: DeserializeOwned>(
    client: &reqwest::Client,
    url: Url,
    token: Option<&str>,
) -> Result<T> {
    let mut request = client.get(url.clone());
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }

    let response = request
        .send()
        .await
        .wrap_err_with(|| format!("requesting `{url}`"))?
        .error_for_status()
        .wrap_err_with(|| format!("requesting `{url}`"))?;

    response
        .json()
        .await
        .wrap_err_with(|| format!("deserializing response from `{url}`"))
}

struct DatasetSeed {
    description_profanity_sample_limit: usize,
}

impl<'de> DeserializeSeed<'de> for DatasetSeed {
    type Value = BatchSummary;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(DatasetVisitor {
            description_profanity_sample_limit: self
                .description_profanity_sample_limit,
        })
    }
}

struct DatasetVisitor {
    description_profanity_sample_limit: usize,
}

impl<'de> Visitor<'de> for DatasetVisitor {
    type Value = BatchSummary;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a moderation project dataset")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut summary = None;
        while let Some(key) = map.next_key::<String>()? {
            if key == "projects" {
                if summary.is_some() {
                    return Err(serde::de::Error::duplicate_field("projects"));
                }
                summary = Some(map.next_value_seed(ProjectsSeed {
                    description_profanity_sample_limit:
                        self.description_profanity_sample_limit,
                })?);
            } else {
                map.next_value::<IgnoredAny>()?;
            }
        }

        summary.ok_or_else(|| serde::de::Error::missing_field("projects"))
    }
}

struct ProjectsSeed {
    description_profanity_sample_limit: usize,
}

impl<'de> DeserializeSeed<'de> for ProjectsSeed {
    type Value = BatchSummary;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ProjectsVisitor {
            description_profanity_sample_limit: self
                .description_profanity_sample_limit,
        })
    }
}

struct ProjectsVisitor {
    description_profanity_sample_limit: usize,
}

impl<'de> Visitor<'de> for ProjectsVisitor {
    type Value = BatchSummary;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an array of projects")
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut summary =
            BatchSummary::new(self.description_profanity_sample_limit);
        while let Some(project) = sequence.next_element::<Project>()? {
            summary.add_project(&project);
        }
        Ok(summary)
    }
}
