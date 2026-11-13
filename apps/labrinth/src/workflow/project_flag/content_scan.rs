use std::{num::NonZero, time::Duration};

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use temporalio_common::error::ApplicationFailure;
use temporalio_sdk::activities::ActivityError;
use url::Url;
use xredis::RedisPool;

use super::model_scan::ModelScanner;
use crate::database::PgPool;
use crate::models::v3::ids::ProjectId;

#[derive(Debug, thiserror::Error)]
#[error("project inference quota reached")]
pub(super) struct InferenceQuotaReached {
    pub retry_after: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageScanInput {
    pub project_id: ProjectId,
    pub url: Url,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImageScanResult {
    pub explicit_probability: Option<f64>,
    pub ai_generated_probability: Option<f64>,
}

/// Increments a fixed-window request counter and checks its quota.
///
/// Returns `None` when allowed or `Some(retry_after)` when over quota. Denied
/// attempts also increment the counter, but do not extend the window. Counter
/// increment, initial expiry, and TTL lookup execute in one Redis transaction.
/// Redis failures return errors rather than allowing requests.
pub async fn check_quota_reached_and_consume(
    redis: &RedisPool,
    namespace: &str,
    key: &str,
    limit: NonZero<u64>,
    window: Duration,
) -> anyhow::Result<Option<Duration>> {
    let mut conn = redis
        .connect()
        .await
        .map_err(|error| anyhow::Error::from_boxed(error.into()))
        .context("connecting to quota Redis")?;
    let key = conn.key().entity(namespace, key);
    let (count, retry_after_ms): (i64, i64) = redis::pipe()
        .atomic()
        .cmd("INCR")
        .arg(&key)
        .cmd("PEXPIRE")
        .arg(&key)
        .arg(window.as_millis())
        .arg("NX")
        .ignore()
        .cmd("PTTL")
        .arg(&key)
        .query_async(&mut conn)
        .await
        .context("checking and consuming quota")?;

    ensure!(count > 0, "invalid quota counter from Redis");
    if count as u64 <= limit.get() {
        Ok(None)
    } else {
        let retry_after_ms = u64::try_from(retry_after_ms)
            .context("invalid quota retry duration from Redis")?;
        Ok(Some(Duration::from_millis(retry_after_ms)))
    }
}

pub(super) fn image_scan_activity_error(error: anyhow::Error) -> ActivityError {
    if let Some(quota) = error.downcast_ref::<InferenceQuotaReached>() {
        let retry_after = quota.retry_after;
        return ActivityError::application(
            ApplicationFailure::builder(error.into_boxed_dyn_error())
                .type_name("InferenceQuotaReached".to_owned())
                .next_retry_delay(retry_after)
                .build(),
        );
    }
    ActivityError::from(error)
}

pub(super) async fn scan_image(
    scanner: &ModelScanner,
    pool: &PgPool,
    input: ImageScanInput,
) -> Result<ImageScanResult> {
    let (explicit_version, ai_generated_version) =
        scanner.image_model_versions()?;
    let rows = sqlx::query!(
        r#"
        SELECT scan_kind, probability
        FROM project_flag_image_scans
        WHERE image_url = $1
          AND ((scan_kind = 'explicit' AND model_version = $2)
            OR (scan_kind = 'ai_generated' AND model_version = $3))
        "#,
        input.url.as_str(),
        explicit_version,
        ai_generated_version,
    )
    .fetch_all(pool)
    .await
    .context("loading project flag image scans")?;

    let mut cached = ImageScanResult::default();
    for row in rows {
        match row.scan_kind.as_str() {
            "explicit" => cached.explicit_probability = Some(row.probability),
            "ai_generated" => {
                cached.ai_generated_probability = Some(row.probability)
            }
            _ => {}
        }
    }
    let needs_explicit =
        explicit_version.is_some() && cached.explicit_probability.is_none();
    let needs_ai_generated = ai_generated_version.is_some()
        && cached.ai_generated_probability.is_none();
    if !needs_explicit && !needs_ai_generated {
        return Ok(cached);
    }

    let (bytes, content_type) = scanner.download_image(&input).await?;
    let explicit = async {
        if let Some(version) = explicit_version.filter(|_| needs_explicit) {
            let probability = scanner
                .scan_image_explicit(input.project_id, &bytes, &content_type)
                .await?
                .context("explicit image scanner is disabled")?;
            return persist_image_scan(
                pool,
                input.url.as_str(),
                "explicit",
                version,
                probability,
            )
            .await
            .map(Some);
        }
        Ok(cached.explicit_probability)
    };
    let ai_generated = async {
        if let Some(version) =
            ai_generated_version.filter(|_| needs_ai_generated)
        {
            let probability = scanner
                .scan_image_ai_generated(
                    input.project_id,
                    &bytes,
                    &content_type,
                )
                .await?
                .context("AI-generated image scanner is disabled")?;
            return persist_image_scan(
                pool,
                input.url.as_str(),
                "ai_generated",
                version,
                probability,
            )
            .await
            .map(Some);
        }
        Ok(cached.ai_generated_probability)
    };
    let (explicit, ai_generated) = tokio::join!(explicit, ai_generated);
    Ok(ImageScanResult {
        explicit_probability: explicit?,
        ai_generated_probability: ai_generated?,
    })
}

async fn persist_image_scan(
    pool: &PgPool,
    image_url: &str,
    scan_kind: &str,
    model_version: i64,
    probability: f64,
) -> Result<f64> {
    sqlx::query!(
        r#"
        INSERT INTO project_flag_image_scans
            (image_url, scan_kind, model_version, probability)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (image_url, scan_kind, model_version) DO NOTHING
        "#,
        image_url,
        scan_kind,
        model_version,
        probability,
    )
    .execute(pool)
    .await
    .context("persisting project flag image scan")?;

    sqlx::query_scalar!(
        r#"
        SELECT probability
        FROM project_flag_image_scans
        WHERE image_url = $1 AND scan_kind = $2 AND model_version = $3
        "#,
        image_url,
        scan_kind,
        model_version,
    )
    .fetch_one(pool)
    .await
    .context("loading persisted project flag image scan")
}
