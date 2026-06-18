use futures::StreamExt;
use rust_decimal::Decimal;
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

/// Fields for [`super::ReturnMetrics::affiliate_code_revenue`].
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum AffiliateCodeRevenueField {
    /// Affiliate code ID.
    AffiliateCodeId,
}

/// Filters for [`super::ReturnMetrics::affiliate_code_revenue`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AffiliateCodeRevenueFilters {
    /// Affiliate code IDs to include.
    #[serde(default)]
    pub affiliate_code_id: Vec<AffiliateCodeId>,
}

/// [`super::ReturnMetrics::affiliate_code_revenue`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AffiliateCodeRevenue {
    /// Total revenue for this bucket.
    pub revenue: Decimal,
}

pub(crate) async fn fetch(
    pool: &PgPool,
    time_slices: &mut [TimeSlice],
    req: &super::super::GetRequest,
    user_id: DBUserId,
    num_time_slices: usize,
    metrics: &Metrics<AffiliateCodeRevenueField, AffiliateCodeRevenueFilters>,
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
                EXTRACT(EPOCH FROM created)::bigint,
                EXTRACT(EPOCH FROM $1::timestamp with time zone AT TIME ZONE 'UTC')::bigint,
                EXTRACT(EPOCH FROM $2::timestamp with time zone AT TIME ZONE 'UTC')::bigint,
                $3::integer
            ) AS "bucket?",
            CASE WHEN $5 THEN affiliate_code_source ELSE 0 END AS "affiliate_code_source?",
            SUM(amount) AS "amount_sum?"
        FROM payouts_values
        WHERE
            user_id = $4
            AND payouts_values.affiliate_code_source IS NOT NULL
            AND created >= $1
            AND created < $2
            AND (cardinality($6::bigint[]) = 0 OR affiliate_code_source = ANY($6))
        GROUP BY 1, 2
        "#,
        req.time_range.start,
        req.time_range.end,
        num_time_slices as i64,
        user_id as DBUserId,
        metrics
            .bucket_by
            .contains(&AffiliateCodeRevenueField::AffiliateCodeId),
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
            row.affiliate_code_source.unwrap_or_default(),
        ));
        let revenue = row.amount_sum.unwrap_or_default();

        add_to_time_slice(
            time_slices,
            bucket,
            AnalyticsData::AffiliateCode(AffiliateCodeAnalytics {
                source_affiliate_code,
                metrics: AffiliateCodeMetrics::Revenue(AffiliateCodeRevenue {
                    revenue,
                }),
            }),
        )?;
    }

    Ok(())
}
