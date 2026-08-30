use std::collections::BTreeMap;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use clap::Parser;
use eyre::{Result, WrapErr, eyre};
use labrinth::models::projects::{Project, Version};
use labrinth::validate::project::{
	ProjectNagKind, ProjectNagSeverity, validate,
};
use serde::Serialize;
use serde::de::{
	DeserializeOwned, DeserializeSeed, IgnoredAny, MapAccess, SeqAccess,
	Visitor,
};
use strum::IntoEnumIterator;
use url::Url;

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
		required_unless_present = "file",
		conflicts_with = "file"
	)]
	project_id: Option<String>,

	/// Validate all projects in a moderation project dataset
	#[arg(long, value_name = "PATH", conflicts_with = "token")]
	file: Option<PathBuf>,

	/// A Modrinth token to send as bearer authentication
	#[arg(long, value_name = "TOKEN")]
	token: Option<String>,
}

#[derive(Serialize)]
struct BatchSummary {
	projects: usize,
	projects_with_at_least_one_required_nag: usize,
	version_details_available: bool,
	nag_counts: BTreeMap<ProjectNagKind, usize>,
}

impl BatchSummary {
	fn new() -> Self {
		Self {
			projects: 0,
			projects_with_at_least_one_required_nag: 0,
			version_details_available: false,
			nag_counts: ProjectNagKind::iter().map(|kind| (kind, 0)).collect(),
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
		for nag in nags {
			*self.nag_counts.entry(nag.kind).or_default() += 1;
		}
	}
}

#[tokio::main]
async fn main() -> Result<()> {
	color_eyre::install().wrap_err("installing color-eyre")?;
	let args = Args::parse();

	if let Some(path) = args.file {
		let summary = summarize_file(&path)?;
		print_json(&summary)?;
		return Ok(());
	}

	let project_id = args
		.project_id
		.as_deref()
		.ok_or_else(|| eyre!("a project ID or `--file` is required"))?;
	check_api_project(project_id, args.token.as_deref()).await
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

fn summarize_file(path: &Path) -> Result<BatchSummary> {
	let file = File::open(path)
		.wrap_err_with(|| format!("opening `{}`", path.display()))?;
	let mut deserializer =
		serde_json::Deserializer::from_reader(BufReader::new(file));
	let summary = DatasetSeed
		.deserialize(&mut deserializer)
		.wrap_err_with(|| format!("reading `{}`", path.display()))?;
	deserializer
		.end()
		.wrap_err_with(|| format!("reading `{}`", path.display()))?;
	Ok(summary)
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

struct DatasetSeed;

impl<'de> DeserializeSeed<'de> for DatasetSeed {
	type Value = BatchSummary;

	fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_map(DatasetVisitor)
	}
}

struct DatasetVisitor;

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
				summary = Some(map.next_value_seed(ProjectsSeed)?);
			} else {
				map.next_value::<IgnoredAny>()?;
			}
		}

		summary.ok_or_else(|| serde::de::Error::missing_field("projects"))
	}
}

struct ProjectsSeed;

impl<'de> DeserializeSeed<'de> for ProjectsSeed {
	type Value = BatchSummary;

	fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_seq(ProjectsVisitor)
	}
}

struct ProjectsVisitor;

impl<'de> Visitor<'de> for ProjectsVisitor {
	type Value = BatchSummary;

	fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		formatter.write_str("an array of projects")
	}

	fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
	where
		A: SeqAccess<'de>,
	{
		let mut summary = BatchSummary::new();
		while let Some(project) = sequence.next_element::<Project>()? {
			summary.add_project(&project);
		}
		Ok(summary)
	}
}
