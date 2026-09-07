use std::fmt;
use std::fs::File;
use std::io::{BufReader, BufWriter, IsTerminal};
use std::path::Path;

use eyre::{Result, WrapErr, eyre};
use labrinth::models::projects::Project;
use rustrict::{Censor, Type};
use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};

const REPORT_FORMAT_VERSION: u32 = 1;
const CENSORED: char = '\0';
const ANSI_START: &str = "\x1b[1;97;41m";
const ANSI_END: &str = "\x1b[0m";

#[derive(Serialize, Deserialize)]
struct ProfanityReport {
    format_version: u32,
    matcher: String,
    projects_scanned: usize,
    projects_with_profanity: usize,
    fields_with_profanity: usize,
    fields_triggering_nags: usize,
    projects: Vec<ProjectProfanityReport>,
}

impl ProfanityReport {
    fn new() -> Self {
        Self {
            format_version: REPORT_FORMAT_VERSION,
            matcher: "rustrict: profane and moderate-or-higher".to_owned(),
            projects_scanned: 0,
            projects_with_profanity: 0,
            fields_with_profanity: 0,
            fields_triggering_nags: 0,
            projects: Vec::new(),
        }
    }

    fn add_project(&mut self, project: &Project) {
        self.projects_scanned += 1;
        let mut fields = Vec::new();
        push_field(
            &mut fields,
            ProfanityFieldKind::ProjectName,
            None,
            &project.name,
        );
        push_field(
            &mut fields,
            ProfanityFieldKind::ProjectSummary,
            None,
            &project.summary,
        );
        push_field(
            &mut fields,
            ProfanityFieldKind::ProjectDescription,
            None,
            &project.description,
        );
        for (gallery_index, gallery_item) in project.gallery.iter().enumerate()
        {
            if let Some(name) = &gallery_item.name {
                push_field(
                    &mut fields,
                    ProfanityFieldKind::GalleryName,
                    Some(gallery_index),
                    name,
                );
            }
            if let Some(description) = &gallery_item.description {
                push_field(
                    &mut fields,
                    ProfanityFieldKind::GalleryDescription,
                    Some(gallery_index),
                    description,
                );
            }
        }

        if fields.is_empty() {
            return;
        }

        self.projects_with_profanity += 1;
        self.fields_with_profanity += fields.len();
        self.fields_triggering_nags +=
            fields.iter().filter(|field| field.triggers_nag).count();
        self.projects.push(ProjectProfanityReport {
            id: project.id.to_string(),
            name: project.name.clone(),
            fields,
        });
    }
}

#[derive(Serialize, Deserialize)]
struct ProjectProfanityReport {
    id: String,
    name: String,
    fields: Vec<ProfanityFieldReport>,
}

#[derive(Serialize, Deserialize)]
struct ProfanityFieldReport {
    kind: ProfanityFieldKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    gallery_index: Option<usize>,
    text: String,
    matches: Vec<ProfanityMatch>,
    triggers_nag: bool,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ProfanityFieldKind {
    ProjectName,
    ProjectSummary,
    ProjectDescription,
    GalleryName,
    GalleryDescription,
}

impl ProfanityFieldKind {
    fn label(self, gallery_index: Option<usize>) -> String {
        match (self, gallery_index) {
            (Self::ProjectName, _) => "Project name".to_owned(),
            (Self::ProjectSummary, _) => "Project summary".to_owned(),
            (Self::ProjectDescription, _) => "Project description".to_owned(),
            (Self::GalleryName, Some(index)) => {
                format!("Gallery image {} name", index + 1)
            }
            (Self::GalleryDescription, Some(index)) => {
                format!("Gallery image {} description", index + 1)
            }
            (Self::GalleryName, None) => "Gallery image name".to_owned(),
            (Self::GalleryDescription, None) => {
                "Gallery image description".to_owned()
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ProfanityMatch {
    start_char: usize,
    end_char: usize,
}

fn push_field(
    fields: &mut Vec<ProfanityFieldReport>,
    kind: ProfanityFieldKind,
    gallery_index: Option<usize>,
    text: &str,
) {
    let matches = profanity_matches(text);
    if matches.is_empty() {
        return;
    }

    let triggers_nag = match kind {
        ProfanityFieldKind::ProjectDescription => matches.len() > 2,
        _ => true,
    };
    fields.push(ProfanityFieldReport {
        kind,
        gallery_index,
        text: text.to_owned(),
        matches,
        triggers_nag,
    });
}

pub(super) fn write_report(
    dataset_path: &Path,
    report_path: &Path,
) -> Result<()> {
    let report = scan_dataset(dataset_path)?;
    let output = File::create(report_path)
        .wrap_err_with(|| format!("creating `{}`", report_path.display()))?;
    serde_json::to_writer_pretty(BufWriter::new(output), &report)
        .wrap_err_with(|| format!("writing `{}`", report_path.display()))?;

    eprintln!(
        "wrote {} projects with profanity across {} fields to `{}`",
        report.projects_with_profanity,
        report.fields_with_profanity,
        report_path.display(),
    );
    Ok(())
}

pub(super) fn render_report(
    report_path: &Path,
    project_limit: usize,
    force_brackets: bool,
) -> Result<()> {
    let input = File::open(report_path)
        .wrap_err_with(|| format!("opening `{}`", report_path.display()))?;
    let report: ProfanityReport =
        serde_json::from_reader(BufReader::new(input))
            .wrap_err_with(|| format!("reading `{}`", report_path.display()))?;
    if report.format_version != REPORT_FORMAT_VERSION {
        return Err(eyre!(
            "unsupported profanity report format version `{}`",
            report.format_version
        ));
    }

    let use_color = !force_brackets
        && std::io::stdout().is_terminal()
        && std::env::var_os("NO_COLOR").is_none();
    println!("=== Project profanity report ===");
    println!("Matcher:                  {}", report.matcher);
    println!("Projects scanned:         {}", report.projects_scanned);
    println!(
        "Projects with profanity: {}",
        report.projects_with_profanity
    );
    println!("Fields with profanity:    {}", report.fields_with_profanity);
    println!(
        "Fields triggering nags:   {}",
        report.fields_triggering_nags
    );
    if use_color {
        println!("Detected text uses a bold white-on-red background.");
    } else {
        println!("Detected text is enclosed in ⟦double brackets⟧.");
    }

    let projects = if project_limit == 0 {
        report.projects.as_slice()
    } else {
        &report.projects[..project_limit.min(report.projects.len())]
    };
    for (project_index, project) in projects.iter().enumerate() {
        println!(
            "\n--- Project {} of {} ---",
            project_index + 1,
            projects.len()
        );
        println!("Project: {}", sanitize_text(&project.name));
        println!("ID:      {}", project.id);
        println!("URL:     https://modrinth.com/project/{}", project.id);

        for field in &project.fields {
            println!(
                "\n[{}; {} match{}; nag: {}]",
                field.kind.label(field.gallery_index),
                field.matches.len(),
                if field.matches.len() == 1 { "" } else { "es" },
                if field.triggers_nag { "yes" } else { "no" },
            );
            println!(
                "{}",
                highlight_matches(&field.text, &field.matches, use_color)
            );
        }
    }
    Ok(())
}

pub(super) fn highlight_text(text: &str, use_color: bool) -> String {
    let matches = profanity_matches(text);
    highlight_matches(text, &matches, use_color)
}

fn scan_dataset(path: &Path) -> Result<ProfanityReport> {
    let input = File::open(path)
        .wrap_err_with(|| format!("opening `{}`", path.display()))?;
    let mut deserializer =
        serde_json::Deserializer::from_reader(BufReader::new(input));
    let report = ProfanityDatasetSeed
        .deserialize(&mut deserializer)
        .wrap_err_with(|| format!("reading `{}`", path.display()))?;
    deserializer
        .end()
        .wrap_err_with(|| format!("reading `{}`", path.display()))?;
    Ok(report)
}

fn profanity_matches(text: &str) -> Vec<ProfanityMatch> {
    let threshold = Type::PROFANE & Type::MODERATE_OR_HIGHER;
    let mut censor = Censor::from_str(text);
    censor
        .with_ignore_self_censoring(true)
        .with_censor_threshold(threshold)
        .with_censor_first_character_threshold(threshold)
        .with_censor_replacement(CENSORED);
    let censored = censor.censor().chars().collect::<Vec<_>>();

    let mut matches = Vec::new();
    let mut start = None;
    let mut character_count = 0;
    for (index, _) in text.chars().enumerate() {
        character_count = index + 1;
        let is_censored = censored.get(index) == Some(&CENSORED);
        match (start, is_censored) {
            (None, true) => start = Some(index),
            (Some(start_char), false) => {
                matches.push(ProfanityMatch {
                    start_char,
                    end_char: index,
                });
                start = None;
            }
            _ => {}
        }
    }
    if let Some(start_char) = start {
        matches.push(ProfanityMatch {
            start_char,
            end_char: character_count,
        });
    }
    matches
}

fn highlight_matches(
    text: &str,
    matches: &[ProfanityMatch],
    use_color: bool,
) -> String {
    let mut output = String::with_capacity(text.len());
    let mut matches = matches.iter().peekable();
    let mut highlighting = false;
    for (index, character) in text.chars().enumerate() {
        while matches
            .peek()
            .is_some_and(|matched| index >= matched.end_char)
        {
            matches.next();
        }
        let is_censored = matches.peek().is_some_and(|matched| {
            index >= matched.start_char && index < matched.end_char
        });
        if is_censored != highlighting {
            output.push_str(if is_censored {
                if use_color { ANSI_START } else { "⟦" }
            } else if use_color {
                ANSI_END
            } else {
                "⟧"
            });
            highlighting = is_censored;
        }
        push_safe_character(&mut output, character);
    }
    if highlighting {
        output.push_str(if use_color { ANSI_END } else { "⟧" });
    }
    output
}

fn sanitize_text(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    for character in text.chars() {
        push_safe_character(&mut output, character);
    }
    output
}

fn push_safe_character(output: &mut String, character: char) {
    match character {
        '\n' | '\t' => output.push(character),
        '\r' => {}
        character if character.is_control() => output.push('�'),
        character => output.push(character),
    }
}

struct ProfanityDatasetSeed;

impl<'de> DeserializeSeed<'de> for ProfanityDatasetSeed {
    type Value = ProfanityReport;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(ProfanityDatasetVisitor)
    }
}

struct ProfanityDatasetVisitor;

impl<'de> Visitor<'de> for ProfanityDatasetVisitor {
    type Value = ProfanityReport;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a moderation project dataset")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut report = None;
        while let Some(key) = map.next_key::<String>()? {
            if key == "projects" {
                if report.is_some() {
                    return Err(serde::de::Error::duplicate_field("projects"));
                }
                report = Some(map.next_value_seed(ProfanityProjectsSeed)?);
            } else {
                map.next_value::<IgnoredAny>()?;
            }
        }
        report.ok_or_else(|| serde::de::Error::missing_field("projects"))
    }
}

struct ProfanityProjectsSeed;

impl<'de> DeserializeSeed<'de> for ProfanityProjectsSeed {
    type Value = ProfanityReport;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ProfanityProjectsVisitor)
    }
}

struct ProfanityProjectsVisitor;

impl<'de> Visitor<'de> for ProfanityProjectsVisitor {
    type Value = ProfanityReport;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an array of projects")
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut report = ProfanityReport::new();
        while let Some(project) = sequence.next_element::<Project>()? {
            report.add_project(&project);
        }
        Ok(report)
    }
}
