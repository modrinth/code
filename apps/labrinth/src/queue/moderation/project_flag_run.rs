use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

use super::project_flag::{ProjectFlagEngine, ProjectRuleCall};
use eyre::{Result, WrapErr, eyre};
use futures::{StreamExt, TryStreamExt, stream};
use rdkafka::{
    Message,
    consumer::{Consumer, StreamConsumer},
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

/// Executes and persists run jobs; offset commits are not yet implemented.
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
    consumer
        .subscribe(&[PROJECT_FLAG_RUN_TOPIC])
        .wrap_err("subscribing to project flag run Kafka topic")?;

    tracing::warn!(
        topic = PROJECT_FLAG_RUN_TOPIC,
        consumer_group = PROJECT_FLAG_RUN_CONSUMER_GROUP,
        batch_size,
        "Started project flag run consumer; offset commits are disabled"
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
            eyre::Ok(())
        }
        .await;
        if let Err(error) = result {
            tracing::error!(?error, "Project flag run consumer failed");
            tokio::time::sleep(KAFKA_OPERATION_INTERVAL).await;
        }
    }
}

struct PreparedRule {
    id: ProjectFlagRuleId,
    call: ProjectRuleCall,
    pattern_names: BTreeMap<PatternName, PatternName>,
}

struct PreparedRun {
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
        let key = connection
            .key()
            .entity("project_flag_run_rules:v1", run_id.0);
        let snapshot: Option<Vec<u8>> = redis::cmd("GET")
            .arg(key)
            .query_async(&mut connection)
            .await
            .wrap_err("fetching project flag rule snapshot")?;
        let snapshot = snapshot.ok_or_else(|| {
            eyre!("rule snapshot missing for run {}", run_id.0)
        })?;
        let rules: Vec<DBProjectFlagRule> =
            serde_json::from_slice(&snapshot)
                .wrap_err("decoding project flag rule snapshot")?;
        snapshots.insert(run_id, rules);
    }
    drop(connection);

    let compile_engine = engine.clone();
    let runs = tokio::task::spawn_blocking(move || {
        snapshots
            .into_iter()
            .map(|(run_id, rules)| {
                let mut prepared = PreparedRun {
                    rules: Vec::new(),
                    patterns: BTreeMap::new(),
                };
                for rule in rules {
                    let call = compile_engine
                        .make_call(&rule.params)
                        .wrap_err_with(|| {
                            format!("compiling project flag rule {}", rule.id.0)
                        })?;
                    let mut pattern_names = BTreeMap::new();
                    for (name, regex) in &call.scan_files_for {
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

    let prepared =
        futures::future::try_join_all(events.into_iter().map(|event| {
            let contexts = &contexts;
            let files = &files;
            let runs = &runs;
            async move {
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
                let run = runs.get(&event.run_id).ok_or_else(|| {
                    eyre!("rule snapshot missing for run {}", event.run_id.0)
                })?;
                let files = files
                    .get(&project_id)
                    .map(Vec::as_slice)
                    .unwrap_or_default();
                let context = engine
                    .prepare_context(context, files, &run.patterns)
                    .await
                    .wrap_err_with(|| {
                        format!(
                            "preparing project {} for run {}",
                            event.project_id, event.run_id.0
                        )
                    })?;
                eyre::Ok((event, context))
            }
        }))
        .await?;

    let tasks = prepared
        .into_iter()
        .map(|(event, mut context)| {
            let engine = engine.clone();
            let runs = runs.clone();
            tokio::task::spawn_blocking(move || {
                let original_patterns = context
                    .scanned_files
                    .iter()
                    .map(|file| file.found_patterns.clone())
                    .collect::<Vec<_>>();
                let mut results = Vec::new();
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
                    let (flags, log, error) = match engine
                        .evaluate(&rule.call, &context)
                    {
                        Ok(result) => (result.flags, result.log, None),
                        Err(error) => {
                            (Vec::new(), Vec::new(), Some(format!("{error:#}")))
                        }
                    };
                    results.push(DBProjectFlagRunResult {
                        run_id: event.run_id,
                        project_id: event.project_id.into(),
                        rule_id: rule.id,
                        flags,
                        log,
                        error,
                    });
                }
                results
            })
        })
        .collect::<Vec<_>>();
    let mut results = Vec::new();
    for result in futures::future::join_all(tasks).await {
        results
            .extend(result.wrap_err("joining project flag batch evaluation")?);
    }
    let result_count = results.len();
    DBProjectFlagRunResult::upsert_many(results, pool).await?;
    tracing::info!(result_count, "Persisted project flag run batch results");
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectFlagRunEvent {
    pub project_id: ProjectId,
    pub run_id: ProjectFlagRunId,
}

/// Stores a permanent JSON rules snapshot under the run ID and atomically
/// publishes its project jobs. Consumers must use `read_committed` isolation.
/// Snapshots are retained on failure, including indeterminate commit outcomes.
/// Retrying a run is not guaranteed to be duplicate-free.
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
                            ProjectFlagRunEvent { project_id, run_id },
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
            producer
                .commit_transaction(KAFKA_DELIVERY_TIMEOUT)
                .wrap_err("committing project flag run Kafka transaction")?;
            eyre::Ok(())
        })();

        if let Err(error) = result {
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
    .wrap_err("joining project flag run dispatch")?
}
