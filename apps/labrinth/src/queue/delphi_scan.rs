use std::time::Duration;

use actix_web::web;
use eyre::{Result, WrapErr, eyre};
use rdkafka::{
    Message, Offset,
    consumer::{CommitMode, Consumer, StreamConsumer},
    message::BorrowedMessage,
    producer::FutureRecord,
    util::Timeout,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    database::{
        PgPool, PgTransaction,
        advisory_lock::AdvisoryLock,
        models::{DBFileId, DelphiReportId},
    },
    env::ENV,
    models::ids::FileId,
    routes::internal::delphi::{self, DelphiRunParameters},
    util::{
        http::HTTP_CLIENT,
        kafka::{
            DELPHI_FILE_SCAN_TASK, KAFKA_OPERATION_INTERVAL, KafkaClientState,
            KafkaEvent,
        },
    },
};

pub const DELPHI_FILE_SCAN_TOPIC: &str = "private.labrinth.delphi-file-scan.v1";
pub const DELPHI_FILE_SCAN_FAILED_TOPIC: &str =
    "private.labrinth.delphi-file-scan.failed.v1";

const SCAN_POLL_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DelphiFileScanEvent {
    pub file_id: FileId,
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Serialize)]
struct FailedDelphiFileScanEvent {
    file_id: FileId,
    identifier: Uuid,
}

#[derive(Debug, Deserialize)]
struct DelphiFileScanMessage {
    event_metadata: DelphiFileScanEventMetadata,
    file_id: FileId,
    #[serde(default)]
    force: bool,
}

#[derive(Debug, Deserialize)]
struct DelphiFileScanEventMetadata {
    event_id: Uuid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "delphi_file_scan_outcome", rename_all = "snake_case")]
pub(crate) enum DelphiFileScanOutcome {
    Succeeded,
    SubmissionFailed,
    TimedOut,
}

enum ScanResult {
    AlreadySucceeded,
    Succeeded,
    Failed {
        outcome: DelphiFileScanOutcome,
        delphi_version: Option<i32>,
        error: String,
    },
}

pub async fn enqueue_file(
    transaction: &mut PgTransaction<'_>,
    kafka_client: &KafkaClientState,
    file_id: DBFileId,
) -> Result<()> {
    enqueue_file_inner(transaction, kafka_client, file_id, false).await
}

pub async fn force_enqueue_file(
    transaction: &mut PgTransaction<'_>,
    kafka_client: &KafkaClientState,
    file_id: DBFileId,
) -> Result<()> {
    enqueue_file_inner(transaction, kafka_client, file_id, true).await
}

async fn enqueue_file_inner(
    transaction: &mut PgTransaction<'_>,
    kafka_client: &KafkaClientState,
    file_id: DBFileId,
    force: bool,
) -> Result<()> {
    // Kafka can deliver the event before the transaction inserting the file
    // commits. The consumer takes the same lock before checking the file, so it
    // observes either the committed file or the completed rollback.
    AdvisoryLock::DelphiFile(file_id)
        .acquire(transaction)
        .await
        .wrap_err("locking file for Delphi scan enqueue")?;

    let event = KafkaEvent::new(
        DELPHI_FILE_SCAN_TOPIC,
        DelphiFileScanEvent {
            file_id: file_id.into(),
            force,
        },
    );
    let key = event.data.file_id.to_string();
    let payload = serde_json::to_vec(&event)
        .wrap_err("serializing Delphi file scan event")?;
    let record = FutureRecord::to(DELPHI_FILE_SCAN_TOPIC)
        .key(&key)
        .payload(&payload);

    tokio::time::timeout(
        KAFKA_OPERATION_INTERVAL,
        kafka_client
            .client
            .send(record, Timeout::After(KAFKA_OPERATION_INTERVAL)),
    )
    .await
    .wrap_err("timing out Delphi file scan event publication")?
    .map_err(|(err, _)| eyre!(err))
    .wrap_err("publishing Delphi file scan event")?;

    Ok(())
}

pub async fn record_succeeded_scan(
    transaction: &mut PgTransaction<'_>,
    scan_id: Uuid,
    file_id: DBFileId,
    delphi_version: i32,
    report_id: DelphiReportId,
) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO delphi_file_scans (
            id,
            file_id,
            delphi_version,
            report_id,
            outcome
        )
        VALUES ($1, $2, $3, $4, 'succeeded')
        ON CONFLICT (id) DO NOTHING
        "#,
        scan_id,
        file_id.0,
        delphi_version,
        report_id as DelphiReportId,
    )
    .execute(&mut *transaction)
    .await
    .wrap_err("recording successful Delphi file scan")?;

    Ok(())
}

pub async fn run(
    pool: PgPool,
    kafka_client: web::Data<KafkaClientState>,
) -> Result<()> {
    let consumer = &kafka_client.delphi_file_scan_consumer;
    consumer
        .subscribe(&[DELPHI_FILE_SCAN_TOPIC])
        .wrap_err("subscribing to Delphi file scan Kafka topic")?;

    info!(
        kafka.topic = DELPHI_FILE_SCAN_TOPIC,
        kafka.consumer_group = DELPHI_FILE_SCAN_TASK,
        "Started Delphi file scan Kafka consumer"
    );

    loop {
        let message = consumer
            .recv()
            .await
            .wrap_err("receiving Delphi file scan Kafka message")?;

        if let Err(err) =
            process_message(&pool, &kafka_client, consumer, &message).await
        {
            tracing::error!(
                kafka.topic = message.topic(),
                kafka.partition = message.partition(),
                kafka.offset = message.offset(),
                "Failed to process Delphi file scan event: {err:?}"
            );

            consumer
                .seek(
                    message.topic(),
                    message.partition(),
                    Offset::Offset(message.offset()),
                    Timeout::After(KAFKA_OPERATION_INTERVAL),
                )
                .wrap_err("seeking failed Delphi file scan Kafka message")?;
            tokio::time::sleep(KAFKA_OPERATION_INTERVAL).await;
        }
    }
}

async fn process_message(
    pool: &PgPool,
    kafka_client: &KafkaClientState,
    consumer: &StreamConsumer,
    message: &BorrowedMessage<'_>,
) -> Result<()> {
    let Some(payload) = message.payload() else {
        tracing::error!(
            kafka.topic = message.topic(),
            kafka.partition = message.partition(),
            kafka.offset = message.offset(),
            "Skipping Delphi file scan event without payload"
        );
        return commit_message(consumer, message);
    };

    let event = match serde_json::from_slice::<DelphiFileScanMessage>(payload) {
        Ok(event) => event,
        Err(err) => {
            tracing::error!(
                kafka.topic = message.topic(),
                kafka.partition = message.partition(),
                kafka.offset = message.offset(),
                "Skipping malformed Delphi file scan event: {err:?}"
            );
            return commit_message(consumer, message);
        }
    };

    let scan_id = event.event_metadata.event_id;
    let file_id = DBFileId::from(event.file_id);
    match scan_outcome(pool, scan_id).await? {
        Some(DelphiFileScanOutcome::Succeeded) => {
            info!(%event.file_id, %scan_id, "Skipping successful Delphi file scan");
            return commit_message(consumer, message);
        }
        Some(_) => {
            info!(%event.file_id, %scan_id, "Republishing failed Delphi file scan");
            publish_failed_file(kafka_client, event.file_id, scan_id).await?;
            return commit_message(consumer, message);
        }
        None => {}
    }

    if !wait_for_file_transaction(pool, file_id).await? {
        info!(%event.file_id, %scan_id, "Discarding Delphi scan for missing file");
        return commit_message(consumer, message);
    }

    match scan_file(pool, event.file_id, scan_id, event.force).await? {
        ScanResult::AlreadySucceeded => {
            info!(
                %event.file_id,
                %scan_id,
                "Skipping file already scanned by this Delphi version"
            );
        }
        ScanResult::Succeeded => {
            info!(%event.file_id, %scan_id, "Completed Delphi file scan");
        }
        ScanResult::Failed {
            outcome,
            delphi_version,
            error,
        } => {
            let mut transaction = pool
                .begin()
                .await
                .wrap_err("beginning failed Delphi file scan transaction")?;
            AdvisoryLock::DelphiScan(scan_id)
                .acquire(&mut transaction)
                .await
                .wrap_err("locking failed Delphi scan transition")?;

            let inserted = record_failed_scan(
                &mut transaction,
                scan_id,
                file_id,
                delphi_version,
                outcome,
                &error,
            )
            .await?;

            transaction
                .commit()
                .await
                .wrap_err("committing failed Delphi file scan")?;

            if inserted {
                publish_failed_file(kafka_client, event.file_id, scan_id)
                    .await?;
            } else {
                info!(
                    %event.file_id,
                    %scan_id,
                    "Delphi file scan became terminal while recording failure"
                );
            }
        }
    }

    commit_message(consumer, message)
}

async fn wait_for_file_transaction(
    pool: &PgPool,
    file_id: DBFileId,
) -> Result<bool> {
    let mut transaction = pool
        .begin()
        .await
        .wrap_err("beginning Delphi file existence transaction")?;

    AdvisoryLock::DelphiFile(file_id)
        .acquire(&mut transaction)
        .await
        .wrap_err("waiting for file upload transaction")?;

    let exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM files WHERE id = $1) AS \"exists!\"",
        file_id.0,
    )
    .fetch_one(&mut transaction)
    .await
    .wrap_err("checking whether Delphi scan file exists")?;

    transaction
        .commit()
        .await
        .wrap_err("committing Delphi file existence transaction")?;

    Ok(exists)
}

async fn scan_file(
    pool: &PgPool,
    file_id: FileId,
    scan_id: Uuid,
    force: bool,
) -> Result<ScanResult> {
    let deadline = tokio::time::Instant::now()
        + Duration::from_secs(ENV.DELPHI_SCAN_TIMEOUT);
    let delphi_version = match tokio::time::timeout_at(
        deadline,
        fetch_delphi_version(),
    )
    .await
    {
        Ok(Ok(version)) => Some(version),
        Ok(Err(err)) => {
            warn!(%file_id, %scan_id, "Failed to fetch Delphi version: {err:?}");
            None
        }
        Err(_) => return Ok(scan_timed_out(file_id, scan_id, None)),
    };

    if !force
        && let Some(delphi_version) = delphi_version
        && successful_report_exists(pool, file_id, delphi_version).await?
    {
        return Ok(ScanResult::AlreadySucceeded);
    }

    info!(%file_id, %scan_id, "Submitting file to Delphi");
    match tokio::time::timeout_at(
        deadline,
        delphi::run(
            pool,
            DelphiRunParameters { file_id },
            scan_id,
            &HTTP_CLIENT,
        ),
    )
    .await
    {
        Ok(Ok(_)) => {}
        Ok(Err(err)) => {
            warn!(%file_id, %scan_id, "Failed to submit file to Delphi: {err:?}");
            return Ok(ScanResult::Failed {
                outcome: DelphiFileScanOutcome::SubmissionFailed,
                delphi_version,
                error: format!("{err:#}"),
            });
        }
        Err(_) => {
            return Ok(scan_timed_out(file_id, scan_id, delphi_version));
        }
    }

    if wait_for_succeeded_scan(pool, scan_id, deadline).await? {
        Ok(ScanResult::Succeeded)
    } else {
        Ok(scan_timed_out(file_id, scan_id, delphi_version))
    }
}

async fn successful_report_exists(
    pool: &PgPool,
    file_id: FileId,
    delphi_version: i32,
) -> Result<bool> {
    sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM delphi_reports
            WHERE file_id = $1 AND delphi_version = $2
        ) AS "exists!"
        "#,
        file_id.0 as i64,
        delphi_version,
    )
    .fetch_one(pool)
    .await
    .wrap_err("checking whether file was scanned by this Delphi version")
}

fn scan_timed_out(
    file_id: FileId,
    scan_id: Uuid,
    delphi_version: Option<i32>,
) -> ScanResult {
    let timeout = ENV.DELPHI_SCAN_TIMEOUT;
    let error = format!("delphi scan timed out after {timeout} seconds");
    warn!(%file_id, %scan_id, %error);
    ScanResult::Failed {
        outcome: DelphiFileScanOutcome::TimedOut,
        delphi_version,
        error,
    }
}

pub(crate) async fn fetch_delphi_version() -> Result<i32> {
    let response = HTTP_CLIENT
        .get(format!("{}/version", ENV.DELPHI_URL))
        .send()
        .await
        .and_then(|response| response.error_for_status())
        .wrap_err("fetching Delphi version")?;
    let version = response
        .text()
        .await
        .wrap_err("reading Delphi version response body")?;

    version.trim().parse().wrap_err_with(|| {
        eyre!("invalid Delphi version response body: {version}")
    })
}

async fn wait_for_succeeded_scan(
    pool: &PgPool,
    scan_id: Uuid,
    deadline: tokio::time::Instant,
) -> Result<bool> {
    let wait = async {
        loop {
            if scan_succeeded(pool, scan_id).await? {
                return Ok(true);
            }
            tokio::time::sleep(SCAN_POLL_INTERVAL).await;
        }
    };

    match tokio::time::timeout_at(deadline, wait).await {
        Ok(result) => result,
        Err(_) => Ok(false),
    }
}

async fn scan_succeeded(pool: &PgPool, scan_id: Uuid) -> Result<bool> {
    sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM delphi_file_scans
            WHERE id = $1
                AND outcome = 'succeeded'
        ) AS "succeeded!"
        "#,
        scan_id,
    )
    .fetch_one(pool)
    .await
    .wrap_err("checking for completed Delphi scan")
}

pub(crate) async fn scan_outcome<'e>(
    exec: impl crate::database::Executor<'e, Database = sqlx::Postgres>,
    scan_id: Uuid,
) -> Result<Option<DelphiFileScanOutcome>> {
    sqlx::query_scalar!(
        r#"
        SELECT outcome AS "outcome: DelphiFileScanOutcome"
        FROM delphi_file_scans
        WHERE id = $1
        "#,
        scan_id,
    )
    .fetch_optional(exec)
    .await
    .wrap_err("checking whether Delphi scan is terminal")
}

async fn record_failed_scan(
    transaction: &mut PgTransaction<'_>,
    scan_id: Uuid,
    file_id: DBFileId,
    delphi_version: Option<i32>,
    outcome: DelphiFileScanOutcome,
    error: &str,
) -> Result<bool> {
    let result = sqlx::query!(
        r#"
        INSERT INTO delphi_file_scans (
            id,
            file_id,
            delphi_version,
            outcome,
            error
        )
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (id) DO NOTHING
        "#,
        scan_id,
        file_id.0,
        delphi_version,
        outcome as DelphiFileScanOutcome,
        error,
    )
    .execute(&mut *transaction)
    .await
    .wrap_err("recording failed Delphi file scan")?;

    Ok(result.rows_affected() == 1)
}

async fn publish_failed_file(
    kafka_client: &KafkaClientState,
    file_id: FileId,
    identifier: Uuid,
) -> Result<()> {
    let event = KafkaEvent::new(
        DELPHI_FILE_SCAN_FAILED_TOPIC,
        FailedDelphiFileScanEvent {
            file_id,
            identifier,
        },
    );
    let key = file_id.to_string();
    let payload = serde_json::to_vec(&event)
        .wrap_err("serializing failed Delphi file scan event")?;
    let record = FutureRecord::to(DELPHI_FILE_SCAN_FAILED_TOPIC)
        .key(&key)
        .payload(&payload);

    tokio::time::timeout(
        KAFKA_OPERATION_INTERVAL,
        kafka_client
            .client
            .send(record, Timeout::After(KAFKA_OPERATION_INTERVAL)),
    )
    .await
    .wrap_err("timing out failed Delphi file scan event publication")?
    .map_err(|(err, _)| eyre!(err))
    .wrap_err("publishing failed Delphi file scan event")?;

    Ok(())
}

fn commit_message(
    consumer: &StreamConsumer,
    message: &BorrowedMessage<'_>,
) -> Result<()> {
    consumer
        .commit_message(message, CommitMode::Sync)
        .wrap_err("committing Delphi file scan Kafka message")
}
