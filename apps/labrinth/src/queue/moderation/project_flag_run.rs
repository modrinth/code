use eyre::{Result, WrapErr, eyre};
use futures::{StreamExt, TryStreamExt, stream};
use rdkafka::producer::{FutureRecord, Producer};
use serde::{Deserialize, Serialize};
use xredis::RedisPool;

use crate::{
    database::models::{
        DBProjectId, ProjectFlagRunId, project_flag_item::DBProjectFlagRule,
    },
    models::ids::ProjectId,
    util::kafka::{
        KAFKA_DELIVERY_TIMEOUT, KAFKA_OPERATION_INTERVAL, KafkaClientState,
        KafkaEvent, create_project_flag_run_producer, sentry_trace_headers,
    },
};

pub const PROJECT_FLAG_RUN_TOPIC: &str = "private.labrinth.project-flag-run.v1";

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
