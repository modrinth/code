use std::{collections::BTreeMap, sync::Arc};

use anyhow::{Context, Result, anyhow};

use super::run::RunSnapshot;
use super::scan_cache::ProjectFlagScanKind;
use crate::{
    database::{
        PgPool,
        models::{
            DBProjectFlagRuleId, DBProjectId, ProjectFlagRunId,
            project_flag_item::{
                DBProjectFlagRunResult, PatternName, ProjectFlagRuleResult,
                project_flag_ruleset_snapshot,
            },
        },
    },
    models::projects::ProjectReviewOutcome,
    queue::moderation::project_flag::{
        FlagContext, ProjectFlagEngine, ProjectRuleCall,
    },
};

#[derive(Debug, thiserror::Error)]
#[error("project flag reconciliation inputs changed")]
pub(crate) struct ReconciliationInputsChanged;

pub(super) async fn reconcile_rules(
    engine: &ProjectFlagEngine,
    pool: &PgPool,
    project_id: DBProjectId,
    ruleset_id: Option<uuid::Uuid>,
    approval_due: bool,
    approval_workflow: Option<temporalio_client::Client>,
) -> Result<()> {
    if !super::project_in_review(pool, project_id.into()).await? {
        return Ok(());
    }
    let Some(ruleset_id) = ruleset_id else {
        return Ok(());
    };
    let snapshot = project_flag_ruleset_snapshot(pool, ruleset_id)
        .await?
        .ok_or_else(|| anyhow!("deployed project flag ruleset missing"))?;
    scan_batch_inner(
        engine,
        pool,
        None,
        vec![project_id],
        snapshot,
        Some(ruleset_id),
        approval_due,
        None,
        approval_workflow.as_ref(),
    )
    .await
}

struct PreparedRule {
    id: DBProjectFlagRuleId,
    call: Result<ProjectRuleCall, String>,
    pattern_names: BTreeMap<PatternName, PatternName>,
}

pub(super) async fn scan_batch(
    engine: &ProjectFlagEngine,
    pool: &PgPool,
    run_id: Option<ProjectFlagRunId>,
    project_ids: Vec<DBProjectId>,
    snapshot: RunSnapshot,
    temporal: &temporalio_client::Client,
    ctx: &temporalio_sdk::activities::ActivityContext,
) -> Result<()> {
    scan_batch_inner(
        engine,
        pool,
        run_id,
        project_ids,
        snapshot,
        None,
        false,
        Some((temporal, ctx)),
        None,
    )
    .await
}

async fn scan_batch_inner(
    engine: &ProjectFlagEngine,
    pool: &PgPool,
    run_id: Option<ProjectFlagRunId>,
    project_ids: Vec<DBProjectId>,
    snapshot: RunSnapshot,
    deployed_ruleset_id: Option<uuid::Uuid>,
    approval_due: bool,
    guard: Option<(
        &temporalio_client::Client,
        &temporalio_sdk::activities::ActivityContext,
    )>,
    approval_workflow: Option<&temporalio_client::Client>,
) -> Result<()> {
    let snapshot_hash = if run_id.is_none() {
        Some(super::scan_cache::rules_input_hash(&snapshot)?)
    } else {
        None
    };
    let ai_rules = snapshot.rules.clone();
    let engine = engine.with_modules(snapshot.modules);
    let compile_engine = engine.clone();
    let (rules, patterns) = tokio::task::spawn_blocking(move || {
        let mut rules = Vec::new();
        let mut patterns = BTreeMap::new();
        for rule in snapshot.rules {
            let call = compile_engine
                .make_call(&rule.params)
                .map_err(|error| format!("{error:#}"));
            let mut pattern_names = BTreeMap::new();
            for (name, regex) in rule.params.scan_files_for {
                let internal = PatternName(format!("{}:{}", rule.id.0, name.0));
                patterns.insert(internal.clone(), regex);
                pattern_names.insert(internal, name);
            }
            rules.push(PreparedRule {
                id: rule.id,
                call,
                pattern_names,
            });
        }
        (rules, patterns)
    })
    .await
    .context("joining project flag rule compilation")?;
    let rules = Arc::new(rules);
    let (contexts, files) = tokio::try_join!(
        async {
            if deployed_ruleset_id.is_some() {
                engine
                    .load_reconciliation_contexts(&project_ids, &ai_rules)
                    .await
            } else {
                let mut remaining = project_ids.clone();
                loop {
                    let loaded = engine
                        .load_contexts_for_rules(&remaining, &ai_rules)
                        .await;
                    match loaded {
                        Ok(contexts) => break Ok(contexts),
                        Err(error) if run_id.is_some() => {
                            let ids = remaining
                                .iter()
                                .map(|id| id.0)
                                .collect::<Vec<_>>();
                            let surviving = sqlx::query_scalar!(
                                r#"SELECT id FROM mods WHERE id = ANY($1)"#,
                                &ids,
                            ).fetch_all(pool).await.context("checking deleted projects after context load failure")?;
                            let previous = remaining.len();
                            remaining.retain(|id| surviving.contains(&id.0));
                            if remaining.len() == previous {
                                break Err(error);
                            }
                            tracing::info!(
                                ?run_id,
                                skipped = previous - remaining.len(),
                                "Skipping deleted projects after context load failure"
                            );
                        }
                        Err(error) => break Err(error),
                    }
                }
            }
        },
        engine.load_files(&project_ids),
    )?;
    if deployed_ruleset_id.is_some()
        && project_ids.iter().all(|id| {
            contexts
                .get(id)
                .is_none_or(|context| !context.project.in_review)
        })
    {
        return Ok(());
    }
    if run_id.is_none() {
        for project_id in &project_ids {
            let context = contexts
                .get(project_id)
                .ok_or_else(|| anyhow!("project no longer exists"))?;
            engine.validate_reconciliation_ai(context, &ai_rules)?;
        }
    }
    let prepared = futures::future::join_all(
        project_ids.iter().copied().filter_map(|project_id| {
            let context = contexts.get(&project_id)?.clone();
            let engine = &engine;
            let patterns = &patterns;
            let files = files
                .get(&project_id)
                .map(Vec::as_slice)
                .unwrap_or_default();
            Some(async move {
                let mut manifest = files
                    .iter()
                    .map(|file| {
                        (
                            file.file_id.0,
                            file.version_id.0,
                            &file.filename,
                            &file.url,
                            file.size,
                        )
                    })
                    .collect::<Vec<_>>();
                manifest.sort_by_key(|file| file.0);
                let manifest_hash =
                    super::scan_cache::rules_input_hash(&manifest);
                let (context, scan_error) = match engine
                    .prepare_context(context.clone(), files, patterns)
                    .await
                {
                    Ok(context) => (context, None),
                    Err(error) => (context, Some(format!("{error:#}"))),
                };
                (project_id, context, scan_error, manifest_hash)
            })
        }),
    )
    .await;
    if run_id.is_none()
        && let Some(error) =
            prepared.iter().find_map(|(_, _, error, _)| error.as_ref())
    {
        return Err(anyhow!("preparing reconciliation file scans: {error}"));
    }
    let tasks = prepared.into_iter().map(
        |(project_id, context, scan_error, manifest_hash)| {
            let engine = engine.clone();
            let rules = rules.clone();
            async move {
                let manifest_hash = manifest_hash?;
                let cache_hash = snapshot_hash
                    .map(|snapshot_hash| {
                        super::scan_cache::rules_input_hash(&(
                            snapshot_hash.as_bytes(),
                            &context,
                            &scan_error,
                            manifest_hash.as_bytes(),
                        ))
                    })
                    .transpose()?;
                let cached = if let Some(hash) = &cache_hash {
                    super::scan_cache::load_scan(
                        pool,
                        ProjectFlagScanKind::Rules,
                        hash.as_bytes(),
                        super::scan_cache::RULES_CACHE_VERSION,
                    )
                    .await
                    .map_err(|error| anyhow!("{error:#}"))?
                } else {
                    None
                };
                let cache_hit = cached.is_some();
                let result = if let Some(cached) = cached {
                    serde_json::from_value(cached).context(
                        "deserializing cached reconciliation results",
                    )?
                } else {
                    let evaluation_context = context.clone();
                    let evaluation_engine = engine.clone();
                    tokio::task::spawn_blocking(move || {
                        evaluate_rules(
                            &evaluation_engine,
                            &rules,
                            evaluation_context,
                            scan_error,
                            run_id,
                            project_id,
                        )
                    })
                    .await
                    .context("joining project flag evaluation")?
                };
                if let Some(hash) = cache_hash {
                    validate_reconciliation_result(&result)?;
                    if !cache_hit {
                        super::scan_cache::persist_scan(
                            pool,
                            ProjectFlagScanKind::Rules,
                            hash.as_bytes(),
                            super::scan_cache::RULES_CACHE_VERSION,
                            &serde_json::to_value(&result)
                                .context("serializing rule results")?,
                        )
                        .await
                        .map_err(|error| anyhow!("{error:#}"))?;
                    }
                    if let Some(ruleset_id) = deployed_ruleset_id {
                        engine
                            .apply_reconciliation(
                                pool,
                                &context,
                                &result,
                                ruleset_id,
                                manifest_hash,
                                approval_due,
                                approval_workflow,
                            )
                            .await?;
                    }
                    Ok(None)
                } else {
                    anyhow::Ok(Some(result))
                }
            }
        },
    );
    let mut results = futures::future::try_join_all(tasks)
        .await?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    results.extend(
        project_ids
            .into_iter()
            .filter(|id| run_id.is_some() && !contexts.contains_key(id))
            .map(|project_id| DBProjectFlagRunResult {
                run_id,
                project_id,
                rules: BTreeMap::new(),
                proposed_review_outcome: None,
                error: Some("project no longer exists".to_owned()),
            }),
    );
    let result_count = results.len();
    DBProjectFlagRunResult::upsert_many_guarded(results, pool, guard).await?;
    tracing::info!(
        ?run_id,
        result_count,
        "Persisted project flag rule results"
    );
    Ok(())
}

fn validate_reconciliation_result(
    result: &DBProjectFlagRunResult,
) -> Result<()> {
    if let Some(error) = &result.error {
        return Err(anyhow!("project flag reconciliation failed: {error}"));
    }
    for (rule, result) in &result.rules {
        if let Some(error) = &result.error {
            return Err(anyhow!("project flag rule `{rule}` failed: {error}"));
        }
    }
    Ok(())
}

fn evaluate_rules(
    engine: &ProjectFlagEngine,
    rules: &[PreparedRule],
    mut context: FlagContext,
    scan_error: Option<String>,
    run_id: Option<ProjectFlagRunId>,
    project_id: DBProjectId,
) -> DBProjectFlagRunResult {
    let original_patterns = context
        .scanned_files
        .iter()
        .map(|file| file.found_patterns.clone())
        .collect::<Vec<_>>();
    let mut results = BTreeMap::new();
    for rule in rules {
        context.ai.decisions = context
            .ai
            .decisions_by_rule
            .get(
                &crate::models::ids::ProjectFlagRuleId::from(rule.id)
                    .to_string(),
            )
            .cloned()
            .unwrap_or_default();
        for (file, names) in
            context.scanned_files.iter_mut().zip(&original_patterns)
        {
            file.found_patterns = names
                .iter()
                .filter_map(|name| rule.pattern_names.get(name).cloned())
                .collect();
        }
        let (evaluation, log) = match &rule.call {
            Ok(call) => engine.evaluate_with_scan_error(
                call,
                &context,
                scan_error.as_deref(),
            ),
            Err(error) => (Err(anyhow!(error.clone())), Vec::new()),
        };
        let (flags, error) = match evaluation {
            Ok(flags) => (flags, None),
            Err(error) => (Vec::new(), Some(format!("{error:#}"))),
        };
        results.insert(
            rule.id.0.to_string(),
            ProjectFlagRuleResult { flags, log, error },
        );
    }
    let proposed_review_outcome = determine_project_review_outcome(
        context.project.in_review,
        results.values(),
        scan_error.is_some(),
    );
    DBProjectFlagRunResult {
        run_id,
        project_id,
        proposed_review_outcome,
        rules: results,
        error: scan_error,
    }
}

/// Computes the rule decision; applying approval still requires eligibility checks.
pub(crate) fn determine_project_review_outcome<'a>(
    in_review: bool,
    rules: impl IntoIterator<Item = &'a ProjectFlagRuleResult>,
    has_error: bool,
) -> Option<ProjectReviewOutcome> {
    use crate::database::models::project_flag_item::{
        effective_project_flag_outcome, effective_project_flags,
    };
    let rules = rules.into_iter().collect::<Vec<_>>();
    let effective = effective_project_flags(
        rules.iter().flat_map(|rule| &rule.flags),
        std::iter::empty(),
    );
    effective_project_flag_outcome(
        in_review,
        &effective,
        has_error || rules.iter().any(|rule| rule.error.is_some()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::models::project_flag_item::{
        FlagOutcome, ProjectFlag,
    };

    fn rule(outcomes: &[FlagOutcome], error: bool) -> ProjectFlagRuleResult {
        ProjectFlagRuleResult {
            flags: outcomes
                .iter()
                .enumerate()
                .map(|(index, outcome)| ProjectFlag {
                    key: format!("test-{index}"),
                    reason: None,
                    flag_outcome: *outcome,
                    retain_when_resolved: false,
                    value: serde_json::json!({}),
                })
                .collect(),
            log: Vec::new(),
            error: error.then(|| "evaluation failed".into()),
        }
    }

    #[test]
    fn reconciliation_rejects_cached_scan_and_rule_errors() {
        let mut result = DBProjectFlagRunResult {
            run_id: None,
            project_id: DBProjectId(1),
            proposed_review_outcome: Some(ProjectReviewOutcome::Approved),
            rules: BTreeMap::new(),
            error: None,
        };
        assert!(validate_reconciliation_result(&result).is_ok());
        result.error = Some("file scan failed".into());
        assert!(validate_reconciliation_result(&result).is_err());
        result.error = None;
        result.rules.insert("1".into(), rule(&[], true));
        assert!(validate_reconciliation_result(&result).is_err());
        result.rules.insert("1".into(), rule(&[], false));
        assert!(validate_reconciliation_result(&result).is_ok());
    }

    #[test]
    fn processing_decisions_and_error_precedence() {
        use FlagOutcome::{ManualReview, Withheld};
        for (outcomes, expected) in [
            (vec![], Some(ProjectReviewOutcome::Approved)),
            (vec![ManualReview], None),
            (
                vec![Withheld, ManualReview],
                Some(ProjectReviewOutcome::Withheld),
            ),
            (
                vec![ManualReview, Withheld],
                Some(ProjectReviewOutcome::Withheld),
            ),
        ] {
            let rules = [rule(&outcomes, false)];
            assert_eq!(
                determine_project_review_outcome(true, &rules, false),
                expected
            );
            assert_eq!(
                determine_project_review_outcome(true, &rules, true),
                None
            );
            assert_eq!(
                determine_project_review_outcome(false, &rules, false),
                None
            );
        }
        let rules = [rule(&[Withheld], false), rule(&[], true)];
        assert_eq!(determine_project_review_outcome(true, &rules, false), None);
        let rules = [rule(&[], true), rule(&[Withheld], false)];
        assert_eq!(determine_project_review_outcome(true, &rules, false), None);
    }

    #[test]
    fn resolved_retained_flags_preserve_evidence_and_block_approval() {
        use crate::database::models::project_flag_item::PersistedProjectFlag;
        for (outcome, expected) in [
            (FlagOutcome::ManualReview, None),
            (FlagOutcome::Withheld, Some(ProjectReviewOutcome::Withheld)),
        ] {
            let original = serde_json::json!({
                "key": "retained", "reason": "original reason",
                "flag_outcome": outcome, "retain_when_resolved": true,
                "evidence": {"file": "original.jar"}, "active": false,
            });
            let resolved: PersistedProjectFlag =
                serde_json::from_value(original.clone()).unwrap();
            assert!(!resolved.active);
            assert!(resolved.flag.retain_when_resolved);
            assert_eq!(serde_json::to_value(&resolved).unwrap(), original);
            let effective = ProjectFlagRuleResult {
                flags: vec![resolved.flag],
                log: Vec::new(),
                error: None,
            };
            assert_eq!(
                determine_project_review_outcome(true, [&effective], false),
                expected
            );
        }
    }

    #[test]
    fn effective_violations_merge_current_evidence_and_sticky_retention() {
        use crate::database::models::project_flag_item::{
            effective_project_flag_outcome, effective_project_flags,
        };
        let retained: ProjectFlag = serde_json::from_value(serde_json::json!({
            "key": "same", "reason": "original", "flag_outcome": "withheld",
            "retain_when_resolved": true, "evidence": {"file": "original.jar"},
        }))
        .unwrap();
        let mut current = retained.clone();
        current.reason = Some("current".into());
        current.flag_outcome = FlagOutcome::ManualReview;
        current.retain_when_resolved = false;
        current.value =
            serde_json::json!({"evidence": {"file": "current.jar"}});
        let effective =
            effective_project_flags([&current, &current], [&retained]);
        assert_eq!(effective.len(), 1);
        assert!(effective[0].active);
        assert!(effective[0].flag.retain_when_resolved);
        assert_eq!(effective[0].flag.reason, current.reason);
        assert_eq!(effective[0].flag.value, current.value);
        assert_eq!(
            effective_project_flag_outcome(true, &effective, false),
            None
        );
        let resolved = effective_project_flags([], [&effective[0].flag]);
        assert!(!resolved[0].active);
        assert_eq!(resolved[0].flag.value, current.value);
        let resolved_original = effective_project_flags([], [&retained]);
        assert_eq!(resolved_original[0].flag.value, retained.value);
        assert_eq!(
            effective_project_flag_outcome(true, &resolved_original, false),
            Some(ProjectReviewOutcome::Withheld)
        );
        assert_eq!(
            effective_project_flag_outcome(true, &resolved_original, true),
            None
        );
        assert_eq!(
            effective_project_flag_outcome(false, &resolved_original, false),
            None
        );
        let combined =
            effective_project_flags([&current], [&retained, &retained]);
        assert_eq!(combined.len(), 1);
        let mut other = retained.clone();
        other.key = "other".into();
        let combined = effective_project_flags([&current], [&retained, &other]);
        assert_eq!(
            effective_project_flag_outcome(true, &combined, false),
            Some(ProjectReviewOutcome::Withheld)
        );
        let expired = effective_project_flags([], [&current]);
        assert!(expired.is_empty());
        assert_eq!(
            effective_project_flag_outcome(true, &expired, false),
            Some(ProjectReviewOutcome::Approved)
        );
    }

    #[test]
    fn legacy_flags_and_cached_results_default_optional_fields() {
        let flag: ProjectFlag =
            serde_json::from_value(serde_json::json!({"key": "test"})).unwrap();
        assert_eq!(flag.reason, None);
        assert_eq!(flag.flag_outcome, FlagOutcome::ManualReview);
        let flag: ProjectFlag = serde_json::from_value(serde_json::json!({"key": "test", "reason": "explanation", "extra": true})).unwrap();
        assert_eq!(flag.reason.as_deref(), Some("explanation"));
        assert_eq!(flag.value, serde_json::json!({"extra": true}));
        let result: DBProjectFlagRunResult = serde_json::from_value(serde_json::json!({"run_id": null, "project_id": 1, "rule_results": {}, "error": null})).unwrap();
        assert_eq!(result.proposed_review_outcome, None);
    }
}
