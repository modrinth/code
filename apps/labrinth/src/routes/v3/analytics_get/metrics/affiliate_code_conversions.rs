use futures::StreamExt;
use serde::{Deserialize, Serialize};

use crate::{
    database::{
        PgPool,
        models::{DBAffiliateCodeId, DBUserId},
    },
    models::ids::AffiliateCodeId,
    routes::ApiError,
    util::error::Context,
};

use super::super::{TimeSlice, add_to_time_slice};
use super::{
    AffiliateCodeAnalytics, AffiliateCodeMetrics, AnalyticsData, Metrics,
};

/// Fields for [`super::ReturnMetrics::affiliate_code_conversions`].
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum AffiliateCodeConversionsField {
    /// Affiliate code ID.
    AffiliateCodeId,
}

/// Filters for [`super::ReturnMetrics::affiliate_code_conversions`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AffiliateCodeConversionsFilters {
    /// Affiliate code IDs to include.
    #[serde(default)]
    pub affiliate_code_id: Vec<AffiliateCodeId>,
}

/// [`super::ReturnMetrics::affiliate_code_conversions`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AffiliateCodeConversions {
    /// Total conversions for this bucket.
    pub conversions: u64,
}

pub(crate) async fn fetch(
    pool: &PgPool,
    time_slices: &mut [TimeSlice],
    req: &super::super::GetRequest,
    user_id: DBUserId,
    num_time_slices: usize,
    metrics: &Metrics<
        AffiliateCodeConversionsField,
        AffiliateCodeConversionsFilters,
    >,
) -> Result<(), ApiError> {
    let filter_affiliate_code_ids = metrics
        .filter_by
        .affiliate_code_id
        .iter()
        .map(|id| DBAffiliateCodeId::from(*id).0)
        .collect::<Vec<_>>();
    let mut rows = sqlx::query!(
        r#"
        SELECT
            WIDTH_BUCKET(
                EXTRACT(EPOCH FROM usa.created_at)::bigint,
                EXTRACT(EPOCH FROM $1::timestamp with time zone AT TIME ZONE 'UTC')::bigint,
                EXTRACT(EPOCH FROM $2::timestamp with time zone AT TIME ZONE 'UTC')::bigint,
                $3::integer
            ) AS "bucket?",
            CASE WHEN $5 THEN affiliate_code ELSE 0 END AS "affiliate_code?",
            COUNT(*) AS "conversions?"
        FROM users_subscriptions_affiliations usa
        INNER JOIN affiliate_codes ac ON ac.id = usa.affiliate_code
        INNER JOIN users_subscriptions us ON us.id = usa.subscription_id
        INNER JOIN charges c ON c.subscription_id = us.id
        WHERE
            ac.affiliate = $4
            AND usa.created_at >= $1
            AND usa.created_at < $2
            AND c.status = 'succeeded'
            AND (cardinality($6::bigint[]) = 0 OR affiliate_code = ANY($6))
        GROUP BY 1, 2
        "#,
        req.time_range.start,
        req.time_range.end,
        num_time_slices as i64,
        user_id as DBUserId,
        metrics
            .bucket_by
            .contains(&AffiliateCodeConversionsField::AffiliateCodeId),
        &filter_affiliate_code_ids,
    )
    .fetch(pool);
    while let Some(row) = rows.next().await.transpose()? {
        let bucket = row
            .bucket
            .wrap_internal_err("bucket should be non-null - query bug!")?;
        let bucket = usize::try_from(bucket).wrap_internal_err_with(|| {
            eyre::eyre!(
                "bucket value {bucket} does not fit into `usize` - query bug!"
            )
        })?;

        let source_affiliate_code = AffiliateCodeId::from(DBAffiliateCodeId(
            row.affiliate_code.unwrap_or_default(),
        ));
        let conversions = u64::try_from(row.conversions.unwrap_or_default())
            .unwrap_or(u64::MAX);

        add_to_time_slice(
            time_slices,
            bucket,
            AnalyticsData::AffiliateCode(AffiliateCodeAnalytics {
                source_affiliate_code,
                metrics: AffiliateCodeMetrics::Conversions(
                    AffiliateCodeConversions { conversions },
                ),
            }),
        )?;
    }

    Ok(())
}
