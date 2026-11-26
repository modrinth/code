use std::{num::NonZero, time::Duration};

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use temporalio_common::error::ApplicationFailure;
use temporalio_sdk::{
    ActivityOptions, WorkflowContext, WorkflowResult, activities::ActivityError,
};
use url::Url;
use xredis::RedisPool;

use super::ProjectFlagActivities;
use super::model_scan::ModelScanner;
use super::scan_cache::{
    ProjectFlagScanKind, input_hash, load_scan, persist_scan,
};
use crate::database::PgPool;
use crate::database::models::DBProjectId;
use crate::models::v3::ids::ProjectId;

pub(super) async fn scan_project_images<W>(
    ctx: &WorkflowContext<W>,
    project_id: ProjectId,
) -> WorkflowResult<()> {
    let images = ctx
        .execute_activity(
            ProjectFlagActivities::project_images,
            project_id,
            scan_activity_options(),
        )
        .await?;

    let results = futures::future::join_all(images.into_iter().map(|image| {
        ctx.execute_activity(
            ProjectFlagActivities::scan_image,
            image,
            scan_activity_options(),
        )
    }))
    .await;
    for result in results {
        result?;
    }

    Ok(())
}

pub(super) fn scan_activity_options() -> ActivityOptions {
    ActivityOptions::with_start_to_close_timeout(Duration::from_secs(60))
        .retry_policy(
            temporalio_common::RetryPolicy::builder()
                .maximum_attempts(5)
                .build(),
        )
        .build()
}

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
    if super::model_scan::is_permanent_scan_error(&error) {
        return ActivityError::application(ApplicationFailure::non_retryable(
            error.into_boxed_dyn_error(),
        ));
    }
    ActivityError::from(error)
}

pub(crate) async fn project_images(
    pool: &PgPool,
    project_id: ProjectId,
) -> Result<Vec<ImageScanInput>> {
    let id = DBProjectId::from(project_id);
    let project = sqlx::query!(
        r#"
        SELECT raw_icon_url, icon_url
        FROM mods
        WHERE id = $1
        "#,
        id.0,
    )
    .fetch_optional(pool)
    .await
    .context("loading project icon for reconciliation")?;
    let Some(project) = project else {
        return Ok(Vec::new());
    };

    let gallery = sqlx::query!(
        r#"
        SELECT raw_image_url
        FROM mods_gallery
        WHERE mod_id = $1
        ORDER BY ordering, id
        "#,
        id.0,
    )
    .fetch_all(pool)
    .await
    .context("loading project gallery for reconciliation")?;
    let icon = project.raw_icon_url.or(project.icon_url);
    gallery
        .iter()
        .map(|image| image.raw_image_url.as_str())
        .chain(icon.as_deref())
        .map(|url| {
            Ok(ImageScanInput {
                project_id,
                url: Url::parse(url).context("parsing project image URL")?,
            })
        })
        .collect()
}

pub(super) async fn scan_image(
    scanner: &ModelScanner,
    pool: &PgPool,
    input: ImageScanInput,
) -> Result<ImageScanResult> {
    let (explicit_version, ai_generated_version) =
        scanner.image_model_versions()?;
    let input_hash = image_input_hash(
        input.url.as_str(),
        ProjectFlagScanKind::Explicit,
        &scanner.config,
    );
    let ai_input_hash = image_input_hash(
        input.url.as_str(),
        ProjectFlagScanKind::AiGenerated,
        &scanner.config,
    );
    let mut cached = ImageScanResult::default();
    for (scan_kind, hash, version, probability) in [
        (
            ProjectFlagScanKind::Explicit,
            input_hash,
            explicit_version,
            &mut cached.explicit_probability,
        ),
        (
            ProjectFlagScanKind::AiGenerated,
            ai_input_hash,
            ai_generated_version,
            &mut cached.ai_generated_probability,
        ),
    ] {
        if let Some(version) = version
            && let Some(result) =
                load_scan(pool, scan_kind, hash.as_bytes(), version).await?
        {
            *probability = Some(image_scan_probability(&result)?);
        }
    }
    let needs_explicit =
        explicit_version.is_some() && cached.explicit_probability.is_none();
    let needs_ai_generated = ai_generated_version.is_some()
        && cached.ai_generated_probability.is_none();
    if !needs_explicit && !needs_ai_generated {
        return Ok(cached);
    }

    let (bytes, content_type) = scanner.read_image(&input).await?;
    let explicit = async {
        if let Some(version) = explicit_version.filter(|_| needs_explicit) {
            let probability = scanner
                .scan_image_explicit(input.project_id, &bytes, content_type)
                .await?
                .context("explicit image scanner is disabled")?;
            return persist_image_scan(
                pool,
                input_hash.as_bytes(),
                ProjectFlagScanKind::Explicit,
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
                .scan_image_ai_generated(input.project_id, &bytes, content_type)
                .await?
                .context("AI-generated image scanner is disabled")?;
            return persist_image_scan(
                pool,
                ai_input_hash.as_bytes(),
                ProjectFlagScanKind::AiGenerated,
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

pub(crate) fn image_scan_probability(
    result: &serde_json::Value,
) -> Result<f64> {
    let probability = result["probability"]
        .as_f64()
        .context("missing or invalid image scan probability")?;
    ensure!(
        (0.0..=1.0).contains(&probability),
        "image scan probability is out of range"
    );
    Ok(probability)
}

async fn persist_image_scan(
    pool: &PgPool,
    input_hash: &[u8],
    scan_kind: ProjectFlagScanKind,
    model_version: i64,
    probability: f64,
) -> Result<f64> {
    let result = persist_scan(
        pool,
        scan_kind,
        input_hash,
        model_version,
        &serde_json::json!({ "probability": probability }),
    )
    .await?;
    image_scan_probability(&result)
}

/// Shares cache identity between inference and cache-only rule context loading.
pub(crate) fn image_input_hash(
    url: &str,
    kind: ProjectFlagScanKind,
    config: &crate::env::ProjectFlags,
) -> blake3::Hash {
    let prompt = if kind == ProjectFlagScanKind::Explicit {
        config.infer_explicit_prompt.as_bytes()
    } else {
        &[]
    };
    input_hash(url, prompt)
}

#[cfg(test)]
mod cache_identity_tests {
    use super::*;

    #[test]
    fn scan_retries_are_bounded() {
        assert_eq!(
            scan_activity_options()
                .retry_policy
                .unwrap()
                .maximum_attempts(),
            5
        );
    }

    #[test]
    fn explicit_identity_tracks_prompt_and_url() {
        let mut config = crate::env::ProjectFlags::default();
        let original = image_input_hash(
            "https://cdn.example/a.png",
            ProjectFlagScanKind::Explicit,
            &config,
        );
        let ai = image_input_hash(
            "https://cdn.example/a.png",
            ProjectFlagScanKind::AiGenerated,
            &config,
        );
        config.infer_explicit_prompt.push('!');
        assert_ne!(
            original,
            image_input_hash(
                "https://cdn.example/a.png",
                ProjectFlagScanKind::Explicit,
                &config
            )
        );
        assert_ne!(
            original,
            image_input_hash(
                "https://cdn.example/b.png",
                ProjectFlagScanKind::Explicit,
                &config
            )
        );
        assert_eq!(
            ai,
            image_input_hash(
                "https://cdn.example/a.png",
                ProjectFlagScanKind::AiGenerated,
                &config
            )
        );
    }
}
