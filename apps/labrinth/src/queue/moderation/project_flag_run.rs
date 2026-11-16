use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

use super::project_flag::{ProjectFlagEngine, ProjectRuleCall};
use eyre::{Result, WrapErr, eyre};
use futures::{StreamExt, TryStreamExt, stream};
use rdkafka::{
    Message, Offset, TopicPartitionList,
    consumer::{CommitMode, Consumer, StreamConsumer},
    error::KafkaError,
    producer::{FutureRecord, Producer},
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use xredis::RedisPool;

use crate::{
    database::models::{
        DBProjectId, ProjectFlagRuleId, ProjectFlagRunId,
        project_flag_item::{
            DBProjectFlagRule, DBProjectFlagRunResult, PatternName,
            ProjectFlagRuleResult,
        },
    },
    database::{PgPool, ReadOnlyPgPool},
    env::ENV,
    file_hosting::FileHost,
    models::ids::ProjectId,
    util::kafka::{
        KAFKA_DELIVERY_TIMEOUT, KAFKA_OPERATION_INTERVAL, KafkaClientState,
        KafkaEvent, consumer_config, create_project_flag_run_producer,
        sentry_trace_headers,
    },
};

pub const PROJECT_FLAG_RUN_TOPIC: &str = "private.labrinth.project-flag-run.v1";
pub const PROJECT_FLAG_RUN_CONSUMER_GROUP: &str = "project-flag-runs";
const SCAN_BATCH_DELAY: Duration = Duration::from_secs(1);

/// Persists each batch before acknowledging its Kafka offsets.
pub async fn consume_project_flag_runs(
    pool: PgPool,
    ro_pool: ReadOnlyPgPool,
    redis: RedisPool,
    file_host: Arc<dyn FileHost>,
) -> Result<()> {
    let engine = ProjectFlagEngine::new(ro_pool, redis.clone(), file_host)?;
    let batch_size = ENV.PROJECT_FLAG_SCAN_BATCH_SIZE.get();
    let consumer: StreamConsumer =
        consumer_config(PROJECT_FLAG_RUN_CONSUMER_GROUP)
            .set("isolation.level", "read_committed")
            .set("enable.auto.offset.store", "false")
            .create()
            .wrap_err("creating project flag run Kafka consumer")?;
    let consumer = Arc::new(consumer);
    consumer
        .subscribe(&[PROJECT_FLAG_RUN_TOPIC])
        .wrap_err("subscribing to project flag run Kafka topic")?;

    tracing::warn!(
        topic = PROJECT_FLAG_RUN_TOPIC,
        consumer_group = PROJECT_FLAG_RUN_CONSUMER_GROUP,
        batch_size,
        "Started project flag run consumer"
    );

    let mut messages = Vec::with_capacity(batch_size);
    loop {
        messages.clear();
        let result = async {
            messages.push(consumer.recv().await.wrap_err(
                "receiving initial project flag run Kafka message",
            )?);
            let deadline = tokio::time::Instant::now() + SCAN_BATCH_DELAY;
            while messages.len() < batch_size {
                match tokio::time::timeout_at(deadline, consumer.recv()).await {
                    Ok(message) => messages.push(message.wrap_err(
                        "receiving project flag run Kafka batch message",
                    )?),
                    Err(_) => break,
                }
            }

            let events = messages
                .iter()
                .map(|message| {
                    let payload = message.payload().ok_or_else(|| {
                        eyre!("project flag run Kafka message has no payload")
                    })?;
                    serde_json::from_slice::<ProjectFlagRunEvent>(payload)
                        .wrap_err("decoding project flag run Kafka message")
                })
                .collect::<Result<Vec<_>>>()?;

            scan_batch(&engine, &redis, &pool, events).await?;
            let mut offsets = BTreeMap::new();
            for message in &messages {
                offsets
                    .entry(message.partition())
                    .and_modify(|offset: &mut i64| {
                        *offset = (*offset).max(message.offset() + 1)
                    })
                    .or_insert(message.offset() + 1);
            }
            let mut positions = TopicPartitionList::new();
            for (partition, offset) in offsets {
                positions
                    .add_partition_offset(
                        PROJECT_FLAG_RUN_TOPIC,
                        partition,
                        Offset::Offset(offset),
                    )
                    .wrap_err("building project flag run consumer offsets")?;
            }
            let consumer = consumer.clone();
            tokio::task::spawn_blocking(move || {
                consumer.commit(&positions, CommitMode::Sync)
            })
            .await
            .wrap_err("joining project flag run offset commit")?
            .wrap_err("committing project flag run consumer offsets")?;
            eyre::Ok(())
        }
        .await;
        if let Err(error) = result {
            tracing::error!(?error, "Project flag run consumer failed");
            return Err(error);
        }
    }
}

struct PreparedRule {
    id: ProjectFlagRuleId,
    call: Result<ProjectRuleCall, String>,
    pattern_names: BTreeMap<PatternName, PatternName>,
}

struct PreparedRun {
    error: Option<String>,
    rules: Vec<PreparedRule>,
    patterns: BTreeMap<PatternName, Regex>,
}

async fn scan_batch(
    engine: &ProjectFlagEngine,
    redis: &RedisPool,
    pool: &PgPool,
    events: Vec<ProjectFlagRunEvent>,
) -> Result<()> {
    let mut snapshots = HashMap::new();
    let mut connection = redis
        .connect()
        .await
        .wrap_err("connecting to Redis for project flag rule snapshots")?;
    for run_id in events.iter().map(|event| event.run_id) {
        if snapshots.contains_key(&run_id) {
            continue;
        }
        let Some(id) = run_id else {
            let rules = DBProjectFlagRule::all(pool)
                .await
                .wrap_err("loading rules for standalone project scans")?;
            snapshots.insert(None, Ok(rules));
            continue;
        };
        let key = connection.key().entity("project_flag_run_rules:v1", id.0);
        let snapshot: Option<Vec<u8>> = redis::cmd("GET")
            .arg(key)
            .query_async(&mut connection)
            .await
            .wrap_err("fetching project flag rule snapshot")?;
        let rules = snapshot
            .ok_or_else(|| format!("rule snapshot missing for run {}", id.0))
            .and_then(|snapshot| {
                serde_json::from_slice::<Vec<DBProjectFlagRule>>(&snapshot)
                    .map_err(|error| {
                        format!("decoding project flag rule snapshot: {error}")
                    })
            });
        snapshots.insert(run_id, rules);
    }
    drop(connection);

    let compile_engine = engine.clone();
    let runs = tokio::task::spawn_blocking(move || {
        snapshots
            .into_iter()
            .map(|(run_id, rules)| {
                let mut prepared = PreparedRun {
                    error: None,
                    rules: Vec::new(),
                    patterns: BTreeMap::new(),
                };
                let rules = match rules {
                    Ok(rules) => rules,
                    Err(error) => {
                        prepared.error = Some(error);
                        return eyre::Ok((run_id, prepared));
                    }
                };
                for rule in rules {
                    let call = compile_engine
                        .make_call(&rule.params)
                        .map_err(|error| format!("{error:#}"));
                    let mut pattern_names = BTreeMap::new();
                    for (name, regex) in &rule.params.scan_files_for {
                        let internal =
                            PatternName(format!("{}:{}", rule.id.0, name.0));
                        prepared
                            .patterns
                            .insert(internal.clone(), regex.clone());
                        pattern_names.insert(internal, name.clone());
                    }
                    prepared.rules.push(PreparedRule {
                        id: rule.id,
                        call,
                        pattern_names,
                    });
                }
                eyre::Ok((run_id, prepared))
            })
            .collect::<Result<HashMap<_, _>>>()
    })
    .await
    .wrap_err("joining project flag rule compilation")??;
    let runs = Arc::new(runs);
    let project_ids = events
        .iter()
        .map(|event| DBProjectId::from(event.project_id))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let (contexts, files) = tokio::try_join!(
        engine.load_contexts(&project_ids),
        engine.load_files(&project_ids),
    )?;

    let consumed_jobs = events
        .iter()
        .filter_map(|event| {
            event.run_id.map(|run_id| (run_id, event.project_id.into()))
        })
        .collect::<Vec<_>>();
    let events = events.into_iter().filter(|event| {
        let exists = contexts.contains_key(&DBProjectId::from(event.project_id));
        if !exists {
            tracing::info!(project_id = %event.project_id, "Skipping deleted project flag scan");
        }
        exists
    });
    let prepared = futures::future::join_all(events.map(|event| {
        let contexts = &contexts;
        let files = &files;
        let runs = &runs;
        async move {
            let prepared = async {
                let project_id = DBProjectId::from(event.project_id);
                let context = contexts
                    .get(&project_id)
                    .ok_or_else(|| {
                        eyre!(
                            "project context missing for {}",
                            event.project_id
                        )
                    })?
                    .clone();
                let run = runs
                    .get(&event.run_id)
                    .ok_or_else(|| eyre!("project scan rules not found"))?;
                if let Some(error) = &run.error {
                    return Err(eyre!(error.clone()));
                }
                let files = files
                    .get(&project_id)
                    .map(Vec::as_slice)
                    .unwrap_or_default();
                let scanned_context = engine
                    .prepare_context(context.clone(), files, &run.patterns)
                    .await
                    .wrap_err_with(|| {
                        format!(
                            "preparing project {} for flag scanning",
                            event.project_id
                        )
                    });
                let (context, scan_error) = match scanned_context {
                    Ok(context) => (context, None),
                    Err(error) => (context, Some(format!("{error:#}"))),
                };
                eyre::Ok((context, scan_error))
            }
            .await;
            (event, prepared)
        }
    }))
    .await;

    let tasks = prepared
        .into_iter()
        .map(|(event, context)| {
            let engine = engine.clone();
            let runs = runs.clone();
            tokio::task::spawn_blocking(move || {
                let (mut context, scan_error) = match context {
                    Ok(context) => context,
                    Err(error) => {
                        return DBProjectFlagRunResult {
                            run_id: event.run_id,
                            project_id: event.project_id.into(),
                            rules: BTreeMap::new(),
                            error: Some(format!("{error:#}")),
                        };
                    }
                };
                let original_patterns = context
                    .scanned_files
                    .iter()
                    .map(|file| file.found_patterns.clone())
                    .collect::<Vec<_>>();
                let mut rules = BTreeMap::new();
                for rule in &runs[&event.run_id].rules {
                    for (file, names) in
                        context.scanned_files.iter_mut().zip(&original_patterns)
                    {
                        file.found_patterns = names
                            .iter()
                            .filter_map(|name| {
                                rule.pattern_names.get(name).cloned()
                            })
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
                    rules.insert(
                        rule.id.0.to_string(),
                        ProjectFlagRuleResult { flags, log, error },
                    );
                }
                DBProjectFlagRunResult {
                    run_id: event.run_id,
                    project_id: event.project_id.into(),
                    rules,
                    error: None,
                }
            })
        })
        .collect::<Vec<_>>();
    let mut results = Vec::new();
    for result in futures::future::join_all(tasks).await {
        results.push(result.wrap_err("joining project flag batch evaluation")?);
    }
    let result_count = results.len();
    DBProjectFlagRunResult::upsert_many(results, &consumed_jobs, pool).await?;
    tracing::info!(result_count, "Persisted project flag run batch results");
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectFlagRunEvent {
    pub project_id: ProjectId,
    pub run_id: Option<ProjectFlagRunId>,
}

#[derive(Debug, thiserror::Error)]
#[error("project flag run dispatch outcome is unknown")]
pub struct ProjectFlagDispatchUncertain;

/// Stores a permanent JSON rules snapshot under the run ID and atomically
/// publishes its project jobs. Consumers must use `read_committed` isolation.
/// Snapshots are retained on failure, including indeterminate commit outcomes.
/// `ProjectFlagDispatchUncertain` errors must leave the run active so committed
/// jobs can persist their results. Commit retries reuse the same transaction;
/// retrying an entire run is not guaranteed to be duplicate-free.
pub async fn dispatch_project_flag_run(
    run_id: ProjectFlagRunId,
    rules: Vec<DBProjectFlagRule>,
    project_ids: Vec<DBProjectId>,
    redis: &RedisPool,
    kafka: &KafkaClientState,
) -> Result<()> {
    let snapshot = serde_json::to_vec(&rules)
        .wrap_err("serializing project flag run rules")?;
    tokio::time::timeout(KAFKA_DELIVERY_TIMEOUT, async {
        let mut connection = redis
            .connect()
            .await
            .wrap_err("connecting to Redis for project flag run")?;
        let key = connection
            .key()
            .entity("project_flag_run_rules:v1", run_id.0);
        redis::cmd("SET")
            .arg(&key)
            .arg(&snapshot)
            .arg("NX")
            .query_async::<Option<String>>(&mut connection)
            .await
            .wrap_err("storing project flag run rules")?
            .ok_or_else(|| eyre!("project flag run rules already exist"))?;
        eyre::Ok(())
    })
    .await
    .wrap_err("timed out storing project flag run rules")??;

    if project_ids.is_empty() {
        return Ok(());
    }

    let headers = sentry_trace_headers();
    let runtime = tokio::runtime::Handle::current();
    let mut guard = kafka.project_flag_run_producer.clone().lock_owned().await;
    tokio::task::spawn_blocking(move || {
        let producer = match guard.take() {
            Some(producer) => producer,
            None => create_project_flag_run_producer()?,
        };

        let mut commit_uncertain = false;
        let result = (|| {
            producer
                .begin_transaction()
                .wrap_err("beginning project flag run Kafka transaction")?;
            runtime.block_on(async {
                tokio::time::timeout(KAFKA_DELIVERY_TIMEOUT, async {
                    let producer = &producer;
                    let headers = &headers;
                    stream::iter(project_ids)
                        .map(|project_id| async move {
                        let project_id = ProjectId::from(project_id);
                        let event = KafkaEvent::new(
                            PROJECT_FLAG_RUN_TOPIC,
                            ProjectFlagRunEvent { project_id, run_id: Some(run_id) },
                        );
                        let payload = serde_json::to_vec(&event)
                            .wrap_err("serializing project flag run event")?;
                        let project_key = project_id.to_string();
                        producer
                            .send(
                                FutureRecord::to(PROJECT_FLAG_RUN_TOPIC)
                                    .key(&project_key)
                                    .payload(&payload)
                                    .headers(headers.clone()),
                                KAFKA_OPERATION_INTERVAL,
                            )
                            .await
                            .map_err(|(error, _)| eyre!(error))
                            .wrap_err("publishing project flag run event")?;
                            eyre::Ok(())
                        })
                        .buffer_unordered(100)
                        .try_collect::<()>()
                        .await
                })
                .await
                .wrap_err("timed out publishing project flag run events")?
            })?;
            commit_uncertain = true;
            for attempt in 0..3 {
                match producer.commit_transaction(KAFKA_DELIVERY_TIMEOUT) {
                    Ok(()) => {
                        commit_uncertain = false;
                        break;
                    }
                    Err(KafkaError::Transaction(error))
                        if error.txn_requires_abort() =>
                    {
                        commit_uncertain = false;
                        return Err(eyre!(error))
                            .wrap_err("committing project flag run Kafka transaction");
                    }
                    Err(KafkaError::Transaction(error))
                        if error.is_retriable() && attempt < 2 =>
                    {
                        tracing::warn!(%error, "Retrying project flag run Kafka commit");
                    }
                    Err(error) => {
                        return Err(eyre!(error))
                            .wrap_err("committing project flag run Kafka transaction");
                    }
                }
            }
            eyre::Ok(())
        })();

        if let Err(error) = result {
            if commit_uncertain {
                return Err(error.wrap_err(ProjectFlagDispatchUncertain));
            }
            if let Err(abort_error) =
                producer.abort_transaction(KAFKA_DELIVERY_TIMEOUT)
            {
                return Err(error.wrap_err(format!(
                    "aborting project flag run Kafka transaction failed: {abort_error}"
                )));
            }
            return Err(error);
        }
        *guard = Some(producer);
        Ok(())
    })
    .await
    .wrap_err("joining project flag run dispatch")
    .wrap_err(ProjectFlagDispatchUncertain)?
}
