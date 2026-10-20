use eyre::{Result, WrapErr, eyre};
use rdkafka::{producer::FutureRecord, util::Timeout};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::{
        PgTransaction,
        models::{DBFileId, DelphiReportId},
    },
    env::ENV,
    models::ids::FileId,
    util::{
        http::HTTP_CLIENT,
        kafka::{
            KAFKA_OPERATION_INTERVAL, KafkaClientState, KafkaEvent,
            sentry_trace_headers,
        },
    },
};

pub const DELPHI_FILE_SCAN_TOPIC: &str = "private.labrinth.delphi-file-scan.v1";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DelphiFileScanEvent {
    pub file_id: FileId,
    #[serde(default)]
    pub force: bool,
    pub decompile: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "delphi_file_scan_outcome", rename_all = "snake_case")]
pub(crate) enum DelphiFileScanOutcome {
    Succeeded,
    SubmissionFailed,
    TimedOut,
}

/// Publishes scan events for files inserted by an upload. Must only be called
/// after the inserting transaction commits, since Delphi resolves the file
/// as soon as it receives the event.
pub async fn publish_new_files(
    kafka_client: &KafkaClientState,
    file_ids: &[DBFileId],
) {
    for &file_id in file_ids {
        if let Err(err) =
            publish_file_scan(kafka_client, file_id, false, true).await
        {
            tracing::error!(
                file_id = %FileId::from(file_id),
                "Failed to enqueue new file for Delphi scan: {err:?}"
            );
        }
    }
}

pub async fn publish_file_scan(
    kafka_client: &KafkaClientState,
    file_id: DBFileId,
    force: bool,
    decompile: bool,
) -> Result<()> {
    let event = KafkaEvent::new(
        DELPHI_FILE_SCAN_TOPIC,
        DelphiFileScanEvent {
            file_id: file_id.into(),
            force,
            decompile,
        },
    );
    let key = event.data.file_id.to_string();
    let payload = serde_json::to_vec(&event)
        .wrap_err("serializing Delphi file scan event")?;
    let record = FutureRecord::to(DELPHI_FILE_SCAN_TOPIC)
        .key(&key)
        .payload(&payload)
        .headers(sentry_trace_headers());

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
