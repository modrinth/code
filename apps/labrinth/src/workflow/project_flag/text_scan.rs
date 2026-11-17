use std::{collections::BTreeSet, time::Duration};

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use temporalio_sdk::{ActivityOptions, WorkflowContext, WorkflowResult};

use super::model_scan::ModelScanner;
use super::scan_cache::{input_hash, load_scan, persist_scan};
use super::{ProjectFlagActivities, ReconcileProjectFlagsWorkflow};
use crate::database::PgPool;
use crate::database::models::{
    DBProjectId, project_flag_item::DBProjectFlagRule,
};
use crate::models::v3::ids::ProjectId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectText {
    title: String,
    summary: String,
    description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTextScan {
    pub project_id: ProjectId,
    pub content: ProjectText,
    pub questions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextScanInput {
    pub project_id: ProjectId,
    pub content: ProjectText,
    pub question: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextScanResult {
    pub probability: f64,
}

pub(super) async fn scan_project_text(
    ctx: &WorkflowContext<ReconcileProjectFlagsWorkflow>,
    project_id: ProjectId,
) -> WorkflowResult<()> {
    let plan = ctx
        .execute_activity(
            ProjectFlagActivities::project_text,
            project_id,
            ActivityOptions::start_to_close_timeout(Duration::from_secs(60)),
        )
        .await?;
    let Some(plan) = plan else {
        return Ok(());
    };

    futures::future::try_join_all(plan.questions.into_iter().map(|question| {
        ctx.execute_activity(
            ProjectFlagActivities::scan_text,
            TextScanInput {
                project_id: plan.project_id,
                content: plan.content.clone(),
                question,
            },
            ActivityOptions::start_to_close_timeout(Duration::from_secs(60)),
        )
    }))
    .await?;
    Ok(())
}

pub(super) async fn project_text(
    pool: &PgPool,
    project_id: ProjectId,
) -> Result<Option<ProjectTextScan>> {
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
    let rules = DBProjectFlagRule::all(pool)
        .await
        .map_err(|error| anyhow::Error::from_boxed(error.into()))
        .context("loading project flag decision questions")?;
    let questions = rules
        .into_iter()
        .flat_map(|rule| rule.params.decision_questions.into_values())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    Ok(Some(ProjectTextScan {
        project_id,
        content: ProjectText {
            title: project.name,
            summary: project.summary,
            description: project.description,
        },
        questions,
    }))
}

pub(super) async fn scan_text(
    scanner: &ModelScanner,
    pool: &PgPool,
    input: TextScanInput,
) -> Result<Option<TextScanResult>> {
    let Some(version) = scanner.decision_model_version()? else {
        return Ok(None);
    };
    let text = serde_json::to_string(&input.content)
        .context("encoding project text")?;
    let content_key = blake3::hash(text.as_bytes()).to_hex().to_string();
    let context_hash = blake3::hash(input.question.as_bytes());
    let input_hash = input_hash(&content_key, context_hash.as_bytes());
    if let Some(result) =
        load_scan(pool, "text", input_hash.as_bytes(), version).await?
    {
        return decode_result(result).map(Some);
    }

    let Some(probability) = scanner
        .scan_text_content(input.project_id, &text, &input.question)
        .await?
    else {
        return Ok(None);
    };
    let result = serde_json::to_value(TextScanResult { probability })
        .context("encoding text scan result")?;
    let result =
        persist_scan(pool, "text", input_hash.as_bytes(), version, &result)
            .await?;
    decode_result(result).map(Some)
}

fn decode_result(value: serde_json::Value) -> Result<TextScanResult> {
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
