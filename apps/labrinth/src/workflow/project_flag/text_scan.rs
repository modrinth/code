use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use temporalio_sdk::{WorkflowContext, WorkflowResult};

use super::ProjectFlagActivities;
use super::model_scan::ModelScanner;
use super::scan_cache::{
    ProjectFlagScanKind, input_hash, load_scan, persist_scan,
};
use crate::database::PgPool;
use crate::database::models::{
    DBProjectId, project_flag_item::project_flag_ruleset_snapshot,
};
use crate::models::v3::ids::ProjectId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectText {
    pub(crate) title: String,
    pub(crate) summary: String,
    pub(crate) description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTextScan {
    pub project_id: ProjectId,
    #[serde(default)]
    pub ruleset_id: Option<uuid::Uuid>,
    pub content: ProjectText,
    pub questions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextScanInput {
    pub project_id: ProjectId,
    #[serde(default)]
    pub ruleset_id: Option<uuid::Uuid>,
    pub content: ProjectText,
    pub questions: Vec<String>,
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
    input: super::ReconcileRulesInput,
) -> Result<Option<ProjectTextScan>> {
    let Some(ruleset_id) = input.ruleset_id() else {
        return Ok(None);
    };
    let project_id = input.project_id();
    let id = DBProjectId::from(project_id);
    let project = sqlx::query!(
        r#"
        SELECT name, summary, description
        FROM mods
        WHERE id = $1
        "#,
        id.0,
    )
    .fetch_optional(pool)
    .await
    .context("loading project text for reconciliation")?;
    let Some(project) = project else {
        return Ok(None);
    };
    let Some(snapshot) = project_flag_ruleset_snapshot(pool, ruleset_id)
        .await
        .map_err(|error| anyhow::Error::from_boxed(error.into()))
        .context("loading deployed project flag decision questions")?
    else {
        return Ok(None);
    };
    let questions = snapshot
        .rules
        .into_iter()
        .flat_map(|rule| rule.params.decision_questions.into_values())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    Ok(Some(ProjectTextScan {
        project_id,
        ruleset_id: Some(ruleset_id),
        content: ProjectText {
            title: project.name,
            summary: project.summary,
            description: project.description,
        },
        questions,
    }))
}

pub(super) async fn text_input_is_current(
    pool: &PgPool,
    input: &TextScanInput,
) -> Result<bool> {
    let current = project_text(
        pool,
        super::ReconcileRulesInput {
            project_id: input.project_id,
            ruleset_id: input.ruleset_id,
            approval_due: false,
            approval_token: None,
        },
    )
    .await?;
    Ok(current.is_some_and(|current| text_plan_matches(&current, input)))
}

fn text_plan_matches(current: &ProjectTextScan, input: &TextScanInput) -> bool {
    current.content == input.content
        && current.questions.iter().collect::<BTreeSet<_>>()
            == input.questions.iter().collect::<BTreeSet<_>>()
}

pub(super) async fn scan_text(
    scanner: &ModelScanner,
    pool: &PgPool,
    input: TextScanInput,
) -> Result<Option<BTreeMap<String, TextScanResult>>> {
    if input.ruleset_id.is_none() {
        return Ok(None);
    }
    let Some(version) = scanner.decision_model_version()? else {
        return Ok(None);
    };
    let mut results = BTreeMap::new();
    let mut missing = Vec::new();
    let mut questions = input.questions;
    questions.sort();
    questions.dedup();
    for question in questions {
        let hash = text_input_hash(&input.content, &question)?;
        if let Some(result) =
            load_scan(pool, ProjectFlagScanKind::Text, hash.as_bytes(), version)
                .await?
        {
            results.insert(question, decode_result(result)?);
        } else {
            missing.push((question, hash));
        }
    }
    if missing.is_empty() {
        return Ok(Some(results));
    }
    let text = serde_json::to_string(&input.content)
        .context("encoding project text")?;
    let questions = missing
        .iter()
        .map(|(question, _)| question.clone())
        .collect::<Vec<_>>();
    let Some(probabilities) = scanner
        .scan_text_content(input.project_id, &text, &questions)
        .await?
    else {
        return Ok(None);
    };
    ensure!(
        probabilities.len() == missing.len(),
        "text scan answer count mismatch"
    );
    for ((question, hash), probability) in
        missing.into_iter().zip(probabilities)
    {
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
        results.insert(question, decode_result(result)?);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn obsolete_text_or_questions_do_not_match_current_plan() {
        let current = ProjectTextScan {
            project_id: crate::database::models::DBProjectId(1).into(),
            ruleset_id: Some(uuid::Uuid::nil()),
            content: ProjectText {
                title: "Title".into(),
                summary: "Summary".into(),
                description: "Description".into(),
            },
            questions: vec!["A?".into(), "B?".into()],
        };
        let mut input = TextScanInput {
            project_id: current.project_id,
            ruleset_id: current.ruleset_id,
            content: current.content.clone(),
            questions: vec!["B?".into(), "A?".into(), "A?".into()],
        };
        assert!(text_plan_matches(&current, &input));
        input.content.summary.push('!');
        assert!(!text_plan_matches(&current, &input));
        input.content = current.content.clone();
        input.questions.push("C?".into());
        assert!(!text_plan_matches(&current, &input));
    }

    #[test]
    fn text_identity_tracks_every_content_field() {
        let mut text = ProjectText {
            title: "Title".into(),
            summary: "Summary".into(),
            description: "Description".into(),
        };
        let original = serde_json::to_string(&text).unwrap();
        text.title.push('!');
        assert_ne!(original, serde_json::to_string(&text).unwrap());
        text.title.pop();
        text.summary.push('!');
        assert_ne!(original, serde_json::to_string(&text).unwrap());
        text.summary.pop();
        text.description.push('!');
        assert_ne!(original, serde_json::to_string(&text).unwrap());
    }

    #[test]
    fn rejects_invalid_cached_probability() {
        assert!(
            decode_result(serde_json::json!({ "probability": 1.1 })).is_err()
        );
        assert!(
            decode_result(serde_json::json!({ "probability": -0.1 })).is_err()
        );
        assert!(
            decode_result(serde_json::json!({ "probability": 0.5 })).is_ok()
        );
    }
}

pub(crate) fn text_input_hash(
    content: &ProjectText,
    question: &str,
) -> Result<blake3::Hash> {
    let text =
        serde_json::to_string(content).context("encoding project text")?;
    let content_key = blake3::hash(text.as_bytes()).to_hex().to_string();
    Ok(input_hash(
        &content_key,
        blake3::hash(question.as_bytes()).as_bytes(),
    ))
}

#[cfg(test)]
mod cache_identity_tests {
    use super::*;

    #[test]
    fn different_prompts_cannot_share_named_answers() {
        let text = ProjectText {
            title: "Title".into(),
            summary: "Summary".into(),
            description: "Description".into(),
        };
        assert_ne!(
            text_input_hash(&text, "Is this advertising?").unwrap(),
            text_input_hash(&text, "Is this explicit?").unwrap()
        );
        let mut changed = text.clone();
        changed.summary.push('!');
        assert_ne!(
            text_input_hash(&text, "Question").unwrap(),
            text_input_hash(&changed, "Question").unwrap()
        );
    }
}
