use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use temporalio_sdk::{WorkflowContext, WorkflowResult};

use super::ProjectFlagActivities;
use super::model_scan::ModelScanner;
use super::scan_cache::{
    ProjectFlagScanKind, input_hash, load_scan, persist_scan, rules_input_hash,
};
use crate::database::PgPool;
use crate::database::models::{
    DBProject, DBProjectId,
    project_flag_item::{
        ContextField, DecisionQuestion, project_flag_ruleset_snapshot,
    },
};
use crate::models::v3::ids::ProjectId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectText {
    pub(crate) title: String,
    pub(crate) summary: String,
    pub(crate) description: String,
    pub(crate) links: serde_json::Value,
    pub(crate) text_fields: BTreeMap<String, serde_json::Value>,
}

impl ProjectText {
    pub(crate) fn from_project(
        project: &crate::models::projects::Project,
        versions: &[crate::models::projects::Version],
    ) -> Result<Self> {
        let mut versions = versions.iter().collect::<Vec<_>>();
        versions.sort_by_key(|version| version.id.0);
        let mut addresses = BTreeMap::new();
        if let Some(server) = &project.components.minecraft_java_server {
            addresses.insert("java", &server.address);
        }
        if let Some(server) = &project.components.minecraft_bedrock_server {
            addresses.insert("bedrock", &server.address);
        }
        let text_fields = BTreeMap::from([
            ("slug".into(), serde_json::json!(project.slug)),
            ("license".into(), serde_json::json!(project.license)),
            ("categories".into(), serde_json::json!(project.categories)),
            (
                "additional_categories".into(),
                serde_json::json!(project.additional_categories),
            ),
            (
                "gallery_titles".into(),
                serde_json::json!(
                    project
                        .gallery
                        .iter()
                        .map(|image| &image.name)
                        .collect::<Vec<_>>()
                ),
            ),
            (
                "gallery_descriptions".into(),
                serde_json::json!(
                    project
                        .gallery
                        .iter()
                        .map(|image| &image.description)
                        .collect::<Vec<_>>()
                ),
            ),
            (
                "version_names".into(),
                serde_json::json!(
                    versions
                        .iter()
                        .map(|version| &version.name)
                        .collect::<Vec<_>>()
                ),
            ),
            (
                "version_numbers".into(),
                serde_json::json!(
                    versions
                        .iter()
                        .map(|version| &version.version_number)
                        .collect::<Vec<_>>()
                ),
            ),
            (
                "version_changelogs".into(),
                serde_json::json!(
                    versions
                        .iter()
                        .map(|version| &version.changelog)
                        .collect::<Vec<_>>()
                ),
            ),
            (
                "filenames".into(),
                serde_json::json!(
                    versions
                        .iter()
                        .flat_map(|version| version
                            .files
                            .iter()
                            .map(|file| &file.filename))
                        .collect::<Vec<_>>()
                ),
            ),
            ("server_addresses".into(), serde_json::json!(addresses)),
        ]);
        Ok(Self {
            text_fields,
            title: project.name.clone(),
            summary: project.summary.clone(),
            description: project.description.clone(),
            links: serde_json::to_value(
                project.link_urls.iter().collect::<BTreeMap<_, _>>(),
            )
            .context("encoding project links")?,
        })
    }

    pub(crate) fn select(
        &self,
        question: &DecisionQuestion,
    ) -> serde_json::Value {
        let mut selected = serde_json::Map::new();
        for field in &question.include_context {
            let (key, value) = match field {
                ContextField::Title => ("title", serde_json::json!(self.title)),
                ContextField::Summary => {
                    ("summary", serde_json::json!(self.summary))
                }
                ContextField::Description => {
                    ("description", serde_json::json!(self.description))
                }
                ContextField::Links => ("links", self.links.clone()),
                field => {
                    let key = match field {
                        ContextField::Slug => "slug",
                        ContextField::License => "license",
                        ContextField::Categories => "categories",
                        ContextField::AdditionalCategories => {
                            "additional_categories"
                        }
                        ContextField::GalleryTitles => "gallery_titles",
                        ContextField::GalleryDescriptions => {
                            "gallery_descriptions"
                        }
                        ContextField::VersionNames => "version_names",
                        ContextField::VersionNumbers => "version_numbers",
                        ContextField::VersionChangelogs => "version_changelogs",
                        ContextField::Filenames => "filenames",
                        ContextField::ServerAddresses => "server_addresses",
                        _ => unreachable!(),
                    };
                    (
                        key,
                        self.text_fields
                            .get(key)
                            .cloned()
                            .unwrap_or(serde_json::Value::Null),
                    )
                }
            };
            selected.insert(key.to_owned(), value);
        }
        serde_json::Value::Object(selected)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTextScan {
    pub project_id: ProjectId,
    #[serde(default)]
    pub ruleset_id: Option<uuid::Uuid>,
    pub content: ProjectText,
    pub questions: Vec<DecisionQuestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextScanInput {
    pub project_id: ProjectId,
    #[serde(default)]
    pub ruleset_id: Option<uuid::Uuid>,
    pub content: ProjectText,
    pub questions: Vec<DecisionQuestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextScanResult {
    pub probability: f64,
}

pub(super) async fn scan_project_text<W>(
    ctx: &WorkflowContext<W>,
    input: super::ReconcileRulesInput,
) -> WorkflowResult<()> {
    let plan = ctx
        .execute_activity(
            ProjectFlagActivities::project_text,
            input,
            super::image_scan::scan_activity_options(),
        )
        .await?;
    let Some(plan) = plan else {
        return Ok(());
    };

    ctx.execute_activity(
        ProjectFlagActivities::scan_text,
        TextScanInput {
            project_id: plan.project_id,
            ruleset_id: plan.ruleset_id,
            content: plan.content,
            questions: plan.questions,
        },
        super::image_scan::scan_activity_options(),
    )
    .await?;
    Ok(())
}

pub(super) async fn project_text(
    pool: &PgPool,
    redis: &xredis::RedisPool,
    input: super::ReconcileRulesInput,
) -> Result<Option<ProjectTextScan>> {
    let Some(ruleset_id) = input.ruleset_id() else {
        return Ok(None);
    };
    let project_id = input.project_id();
    let project = DBProject::get_many_uncached(
        &[crate::models::ids::ProjectId::from(DBProjectId::from(
            project_id,
        ))],
        pool,
        redis,
    )
    .await
    .context("loading authoritative project decision context")?
    .into_iter()
    .next();
    let Some(project) = project else {
        return Ok(None);
    };
    let project = crate::models::projects::Project::from(project);
    let id = DBProjectId::from(project_id);
    let version_ids = sqlx::query_scalar!(
        r#"SELECT id FROM versions WHERE mod_id = $1 ORDER BY id"#,
        id.0,
    )
    .fetch_all(pool)
    .await
    .context("loading version IDs for project decision context")?
    .into_iter()
    .map(crate::database::models::DBVersionId)
    .collect::<Vec<_>>();
    let versions = crate::database::models::DBVersion::get_many_uncached(
        &version_ids,
        pool,
        redis,
    )
    .await
    .context("loading authoritative version decision context")?
    .into_iter()
    .map(crate::models::projects::Version::from)
    .collect::<Vec<_>>();
    let content = ProjectText::from_project(&project, &versions)?;
    let Some(snapshot) = project_flag_ruleset_snapshot(pool, ruleset_id)
        .await
        .map_err(|error| anyhow::Error::from_boxed(error.into()))
        .context("loading deployed project flag decision questions")?
    else {
        return Ok(None);
    };
    for rule in &snapshot.rules {
        rule.params.validate_decision_questions()?;
    }
    let questions = snapshot
        .rules
        .into_iter()
        .flat_map(|rule| rule.params.decision_questions.into_values())
        .map(|question| question.canonicalized())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    Ok(Some(ProjectTextScan {
        project_id,
        ruleset_id: Some(ruleset_id),
        content,
        questions,
    }))
}

pub(super) async fn text_input_is_current(
    pool: &PgPool,
    redis: &xredis::RedisPool,
    input: &TextScanInput,
) -> Result<bool> {
    let current = project_text(
        pool,
        redis,
        super::ReconcileRulesInput {
            project_id: input.project_id,
            ruleset_id: input.ruleset_id,
            approval_due: false,
        },
    )
    .await?;
    Ok(current.is_some_and(|current| text_plan_matches(&current, input)))
}

fn text_plan_matches(current: &ProjectTextScan, input: &TextScanInput) -> bool {
    let canonical = |questions: &[DecisionQuestion]| {
        questions
            .iter()
            .map(DecisionQuestion::canonicalized)
            .collect::<BTreeSet<_>>()
    };
    canonical(&current.questions) == canonical(&input.questions)
        && current.questions.iter().all(|question| {
            current.content.select(question) == input.content.select(question)
        })
}

pub(super) async fn scan_text(
    scanner: &ModelScanner,
    pool: &PgPool,
    input: TextScanInput,
) -> Result<Option<Vec<TextScanResult>>> {
    if input.ruleset_id.is_none() {
        return Ok(None);
    }
    let Some(version) = scanner.decision_model_version()? else {
        return Ok(None);
    };
    let mut results = Vec::new();
    let mut missing = Vec::new();
    let mut questions = input
        .questions
        .iter()
        .map(DecisionQuestion::canonicalized)
        .collect::<Vec<_>>();
    questions.sort();
    questions.dedup();
    for question in questions {
        let hash = text_input_hash(&input.content, &question)?;
        if let Some(result) =
            load_scan(pool, ProjectFlagScanKind::Text, hash.as_bytes(), version)
                .await?
        {
            results.push(decode_result(result)?);
        } else {
            missing.push((question, hash));
        }
    }
    if missing.is_empty() {
        return Ok(Some(results));
    }
    for (question, hash) in missing {
        let Some(probability) = scanner
            .scan_text_content(
                input.project_id,
                input.content.select(&question),
                &question.question,
            )
            .await?
        else {
            return Ok(None);
        };
        let result = serde_json::to_value(TextScanResult { probability })
            .context("encoding text scan result")?;
        let result = persist_scan(
            pool,
            ProjectFlagScanKind::Text,
            hash.as_bytes(),
            version,
            &result,
        )
        .await?;
        results.push(decode_result(result)?);
    }
    Ok(Some(results))
}

pub(crate) fn decode_result(
    value: serde_json::Value,
) -> Result<TextScanResult> {
    let result: TextScanResult =
        serde_json::from_value(value).context("decoding text scan result")?;
    ensure!(
        result.probability.is_finite()
            && (0.0..=1.0).contains(&result.probability),
        "invalid cached text scan probability"
    );
    Ok(result)
}

pub(crate) fn text_input_hash(
    content: &ProjectText,
    question: &DecisionQuestion,
) -> Result<blake3::Hash> {
    let content_key = rules_input_hash(&content.select(question))?
        .to_hex()
        .to_string();
    Ok(input_hash(
        &content_key,
        blake3::hash(question.question.as_bytes()).as_bytes(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn content() -> ProjectText {
        ProjectText {
            text_fields: BTreeMap::new(),
            title: "Title".into(),
            summary: "Summary".into(),
            description: "Private unless selected".into(),
            links: json!({"source": {"platform": "source", "donation": false, "url": "https://example.com"}}),
        }
    }

    fn question(fields: Vec<ContextField>) -> DecisionQuestion {
        DecisionQuestion {
            question: "Question?".into(),
            include_context: fields,
        }
    }

    #[test]
    fn selection_exposes_only_requested_fields_and_preserves_links() {
        let text = content();
        assert_eq!(
            text.select(&question(vec![ContextField::Title])),
            json!({"title": "Title"})
        );
        assert_eq!(
            text.select(&question(vec![ContextField::Links])),
            json!({"links": text.links})
        );
        assert_eq!(
            text.select(&question(vec![
                ContextField::Summary,
                ContextField::Description
            ])),
            json!({"summary": text.summary, "description": text.description})
        );
    }

    #[test]
    fn identity_ignores_unselected_changes_and_normalizes_fields() {
        let text = content();
        let q = question(vec![ContextField::Title]);
        let original = text_input_hash(&text, &q).unwrap();
        let mut changed = text.clone();
        changed.summary.push('!');
        changed.description.push('!');
        changed.links = json!({});
        assert_eq!(original, text_input_hash(&changed, &q).unwrap());
        changed.title.push('!');
        assert_ne!(original, text_input_hash(&changed, &q).unwrap());
        let a = question(vec![ContextField::Links, ContextField::Title]);
        let b = question(vec![
            ContextField::Title,
            ContextField::Links,
            ContextField::Title,
        ]);
        assert_eq!(a.canonicalized(), b.canonicalized());
        assert_eq!(
            text_input_hash(&text, &a).unwrap(),
            text_input_hash(&text, &b).unwrap()
        );
        assert_ne!(original, text_input_hash(&text, &a).unwrap());
        let mut prompt = q.clone();
        prompt.question.push('!');
        assert_ne!(original, text_input_hash(&text, &prompt).unwrap());
        let links = question(vec![ContextField::Links]);
        assert_ne!(
            text_input_hash(&text, &links).unwrap(),
            text_input_hash(&changed, &links).unwrap()
        );
        assert_eq!(
            [a.canonicalized(), b.canonicalized(), q]
                .into_iter()
                .collect::<BTreeSet<_>>()
                .len(),
            2
        );
    }

    #[test]
    fn current_plan_compares_selected_content_and_canonical_questions() {
        let current = ProjectTextScan {
            project_id: crate::database::models::DBProjectId(1).into(),
            ruleset_id: Some(uuid::Uuid::nil()),
            content: content(),
            questions: vec![question(vec![ContextField::Title])],
        };
        let mut input = TextScanInput {
            project_id: current.project_id,
            ruleset_id: current.ruleset_id,
            content: current.content.clone(),
            questions: current.questions.clone(),
        };
        input.content.summary.push('!');
        assert!(text_plan_matches(&current, &input));
        input.content.title.push('!');
        assert!(!text_plan_matches(&current, &input));
        input.content = current.content.clone();
        input.questions.push(question(vec![ContextField::Links]));
        assert!(!text_plan_matches(&current, &input));
    }

    #[test]
    fn granular_fields_share_inference_cache_and_staleness_projection() {
        let fields = [
            (ContextField::Slug, "slug"),
            (ContextField::License, "license"),
            (ContextField::Categories, "categories"),
            (ContextField::AdditionalCategories, "additional_categories"),
            (ContextField::GalleryTitles, "gallery_titles"),
            (ContextField::GalleryDescriptions, "gallery_descriptions"),
            (ContextField::VersionNames, "version_names"),
            (ContextField::VersionNumbers, "version_numbers"),
            (ContextField::VersionChangelogs, "version_changelogs"),
            (ContextField::Filenames, "filenames"),
            (ContextField::ServerAddresses, "server_addresses"),
        ];
        let mut text = content();
        for (_, key) in fields {
            text.text_fields.insert(key.into(), json!(["original"]));
        }
        for (field, key) in fields {
            let q = question(vec![field]);
            let state = text.select(&q);
            assert_eq!(state, json!({key: ["original"]}));
            let expected = input_hash(
                rules_input_hash(&state).unwrap().to_hex().as_ref(),
                blake3::hash(q.question.as_bytes()).as_bytes(),
            );
            assert_eq!(text_input_hash(&text, &q).unwrap(), expected);
            let current = ProjectTextScan {
                project_id: DBProjectId(1).into(),
                ruleset_id: Some(uuid::Uuid::nil()),
                content: text.clone(),
                questions: vec![q.clone()],
            };
            let mut input = TextScanInput {
                project_id: current.project_id,
                ruleset_id: current.ruleset_id,
                content: text.clone(),
                questions: vec![q.clone()],
            };
            input.content.title.push('!');
            assert!(text_plan_matches(&current, &input));
            assert_eq!(text_input_hash(&input.content, &q).unwrap(), expected);
            input
                .content
                .text_fields
                .insert(key.into(), json!(["changed"]));
            assert!(!text_plan_matches(&current, &input));
            assert_ne!(text_input_hash(&input.content, &q).unwrap(), expected);
        }
    }

    #[test]
    fn rejects_invalid_cached_probability() {
        for probability in [-0.1, 1.1] {
            assert!(
                decode_result(json!({"probability": probability})).is_err()
            );
        }
        assert!(decode_result(json!({"probability": 0.5})).is_ok());
    }
}
