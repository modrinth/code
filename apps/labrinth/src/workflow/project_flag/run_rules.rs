use std::{collections::BTreeMap, sync::Arc};

use eyre::{Result, WrapErr, eyre};

use super::run::RunSnapshot;
use crate::{
    database::{
        PgPool,
        models::{
            DBProjectFlagRuleId, DBProjectId, ProjectFlagRunId,
            project_flag_item::{
                DBProjectFlagModule, DBProjectFlagRule, DBProjectFlagRunResult,
                FlagOutcome, PatternName, ProjectFlagRuleResult,
            },
        },
    },
    models::projects::ProjectStatus,
    queue::moderation::project_flag::{
        FlagContext, ProjectFlagEngine, ProjectRuleCall,
    },
};

pub(super) async fn reconcile_rules(
    engine: &ProjectFlagEngine,
    pool: &PgPool,
    project_id: DBProjectId,
) -> Result<()> {
    let rules = DBProjectFlagRule::all(pool).await?;
    let modules = DBProjectFlagModule::list(i64::MAX, 0, &pool.clone().into())
        .await?
        .into_iter()
        .map(|module| (module.name, module.script))
        .collect();
    scan_batch(
        engine,
        pool,
        None,
        vec![project_id],
        RunSnapshot { rules, modules },
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
    .wrap_err("joining project flag rule compilation")?;
    let rules = Arc::new(rules);
    let (contexts, files) = tokio::try_join!(
        engine.load_contexts_for_rules(&project_ids, &ai_rules),
        engine.load_files(&project_ids),
    )?;
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
                let (context, scan_error) = match engine
                    .prepare_context(context.clone(), files, patterns)
                    .await
                {
                    Ok(context) => (context, None),
                    Err(error) => (context, Some(format!("{error:#}"))),
                };
                (project_id, context, scan_error)
            })
        }),
    )
    .await;
    let tasks =
        prepared
            .into_iter()
            .map(|(project_id, context, scan_error)| {
                let engine = engine.clone();
                let rules = rules.clone();
                async move {
                    let cache_hash = snapshot_hash
                        .map(|snapshot_hash| {
                            super::scan_cache::rules_input_hash(&(
                                snapshot_hash.as_bytes(),
                                &context,
                                &scan_error,
                            ))
                        })
                        .transpose()?;
                    if let Some(hash) = &cache_hash
                        && super::scan_cache::load_scan(
                            pool,
                            "rules",
                            hash.as_bytes(),
                            super::scan_cache::RULES_CACHE_VERSION,
                        )
                        .await
                        .map_err(|error| eyre!("{error:#}"))?
                        .is_some()
                    {
                        return Ok(None);
                    }
                    let result = tokio::task::spawn_blocking(move || {
                        evaluate_rules(
                            &engine, &rules, context, scan_error, run_id,
                            project_id,
                        )
                    })
                    .await
                    .wrap_err("joining project flag evaluation")?;
                    if let Some(hash) = cache_hash {
                        super::scan_cache::persist_scan(
                            pool,
                            "rules",
                            hash.as_bytes(),
                            super::scan_cache::RULES_CACHE_VERSION,
                            &serde_json::to_value(&result)
                                .wrap_err("serializing rule results")?,
                        )
                        .await
                        .map_err(|error| eyre!("{error:#}"))?;
                        Ok(None)
                    } else {
                        eyre::Ok(Some(result))
                    }
                }
            });
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
                proposed_status: None,
                error: Some("project no longer exists".to_owned()),
            }),
    );
    let result_count = results.len();
    DBProjectFlagRunResult::upsert_many(results, pool).await?;
    tracing::info!(
        ?run_id,
        result_count,
        "Persisted project flag rule results"
    );
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
            Err(error) => (Err(eyre!(error.clone())), Vec::new()),
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
    let proposed_status = proposed_project_status(
        context.project.status,
        results.values(),
        scan_error.is_some(),
    );
    DBProjectFlagRunResult {
        run_id,
        project_id,
        proposed_status,
        rules: results,
        error: scan_error,
    }
}

/// Computes a preview only; callers must not treat this as a status mutation.
pub(crate) fn proposed_project_status<'a>(
    current_status: ProjectStatus,
    rules: impl IntoIterator<Item = &'a ProjectFlagRuleResult>,
    has_error: bool,
) -> Option<ProjectStatus> {
    if current_status != ProjectStatus::Processing || has_error {
        return None;
    }
    let mut proposed = ProjectStatus::Approved;
    for rule in rules {
        if rule.error.is_some() {
            return None;
        }
        for flag in &rule.flags {
            match flag.flag_outcome {
                FlagOutcome::Withheld => proposed = ProjectStatus::Withheld,
                FlagOutcome::ManualReview
                    if proposed != ProjectStatus::Withheld =>
                {
                    proposed = ProjectStatus::Processing;
                }
                FlagOutcome::ManualReview => {}
            }
        }
    }
    Some(proposed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::models::project_flag_item::ProjectFlag;

    fn rule(outcomes: &[FlagOutcome], error: bool) -> ProjectFlagRuleResult {
        ProjectFlagRuleResult {
            flags: outcomes
                .iter()
                .map(|outcome| ProjectFlag {
                    key: "test".into(),
                    reason: None,
                    flag_outcome: *outcome,
                    retain_if_inactive: false,
                    value: serde_json::json!({}),
                })
                .collect(),
            log: Vec::new(),
            error: error.then(|| "evaluation failed".into()),
        }
    }

    #[test]
    fn processing_decisions_and_error_precedence() {
        use FlagOutcome::{ManualReview, Withheld};
        for (outcomes, expected) in [
            (vec![], ProjectStatus::Approved),
            (vec![ManualReview], ProjectStatus::Processing),
            (vec![Withheld, ManualReview], ProjectStatus::Withheld),
            (vec![ManualReview, Withheld], ProjectStatus::Withheld),
        ] {
            let rules = [rule(&outcomes, false)];
            assert_eq!(
                proposed_project_status(
                    ProjectStatus::Processing,
                    &rules,
                    false
                ),
                Some(expected)
            );
            assert_eq!(
                proposed_project_status(
                    ProjectStatus::Processing,
                    &rules,
                    true
                ),
                None
            );
            for status in ProjectStatus::iterator()
                .filter(|s| *s != ProjectStatus::Processing)
            {
                assert_eq!(
                    proposed_project_status(status, &rules, false),
                    None
                );
            }
        }
        let rules = [rule(&[Withheld], false), rule(&[], true)];
        assert_eq!(
            proposed_project_status(ProjectStatus::Processing, &rules, false),
            None
        );
        let rules = [rule(&[], true), rule(&[Withheld], false)];
        assert_eq!(
            proposed_project_status(ProjectStatus::Processing, &rules, false),
            None
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
        assert_eq!(result.proposed_status, None);
    }
}
