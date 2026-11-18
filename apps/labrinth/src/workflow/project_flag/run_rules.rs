use std::{collections::BTreeMap, sync::Arc};

use eyre::{Result, WrapErr, eyre};

use super::run::RunSnapshot;
use crate::{
    database::{
        PgPool,
        models::{
            DBProjectId, ProjectFlagRuleId, ProjectFlagRunId,
            project_flag_item::{
                DBProjectFlagModule, DBProjectFlagRule, DBProjectFlagRunResult,
                PatternName, ProjectFlagRuleResult,
            },
        },
    },
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
    id: ProjectFlagRuleId,
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
            .get(&rule.id.0.to_string())
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
    DBProjectFlagRunResult {
        run_id,
        project_id,
        rules: results,
        error: None,
    }
}
