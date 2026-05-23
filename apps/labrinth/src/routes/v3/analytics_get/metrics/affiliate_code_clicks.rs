use const_format::formatcp;
use serde::{Deserialize, Serialize};

use crate::{
    database::models::DBAffiliateCodeId, models::ids::AffiliateCodeId,
    routes::ApiError,
};

use super::super::{
    ClickhouseFilterParam, ClickhouseQueryParams, QueryClickhouseContext,
    query_clickhouse,
};
use super::{
    AffiliateCodeAnalytics, AffiliateCodeMetrics, AnalyticsData, Metrics,
};

const TIME_RANGE_START: &str = "{time_range_start: UInt64}";
const TIME_RANGE_END: &str = "{time_range_end: UInt64}";
const TIME_SLICES: &str = "{time_slices: UInt64}";

/// Fields for [`super::ReturnMetrics::affiliate_code_clicks`].
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum AffiliateCodeClicksField {
    /// Affiliate code ID.
    AffiliateCodeId,
}

/// Filters for [`super::ReturnMetrics::affiliate_code_clicks`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AffiliateCodeClicksFilters {
    /// Affiliate code IDs to include.
    #[serde(default)]
    pub affiliate_code_id: Vec<AffiliateCodeId>,
}

/// [`super::ReturnMetrics::affiliate_code_clicks`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AffiliateCodeClicks {
    /// Total clicks for this bucket.
    pub clicks: u64,
}

#[derive(Debug, clickhouse::Row, serde::Deserialize)]
struct AffiliateCodeClickRow {
    bucket: u64,
    affiliate_code_id: DBAffiliateCodeId,
    clicks: u64,
}

const AFFILIATE_CODE_CLICKS: &str = {
    const USE_AFFILIATE_CODE_ID: &str = "{use_affiliate_code_id: Bool}";
    const FILTER_AFFILIATE_CODE_ID: &str =
        "{filter_affiliate_code_id: Array(UInt64)}";

    formatcp!(
        "SELECT
            widthBucket(toUnixTimestamp(recorded), {TIME_RANGE_START}, {TIME_RANGE_END}, {TIME_SLICES}) AS bucket,
            if({USE_AFFILIATE_CODE_ID}, affiliate_code_id, 0) AS affiliate_code_id,
            COUNT(*) AS clicks
        FROM affiliate_code_clicks
        WHERE
            recorded BETWEEN {TIME_RANGE_START} AND {TIME_RANGE_END}
            -- make sure that the REAL affiliate code id is included,
            -- not the possibly-zero one,
            -- by using `affiliate_code_clicks.affiliate_code_id` instead of `project_id`
            AND (empty({FILTER_AFFILIATE_CODE_ID}) OR affiliate_code_id IN {FILTER_AFFILIATE_CODE_ID})
        GROUP BY bucket, affiliate_code_id"
    )
};

pub(crate) async fn fetch(
    cx: &mut QueryClickhouseContext<'_>,
    metrics: &Metrics<AffiliateCodeClicksField, AffiliateCodeClicksFilters>,
) -> Result<(), ApiError> {
    use AffiliateCodeClicksField as F;
    let uses = |field| metrics.bucket_by.contains(&field);

    query_clickhouse::<AffiliateCodeClickRow>(
        cx,
        AFFILIATE_CODE_CLICKS,
        ClickhouseQueryParams::empty(),
        &[("use_affiliate_code_id", uses(F::AffiliateCodeId))],
        vec![ClickhouseFilterParam::AffiliateCodeId(
            "filter_affiliate_code_id",
            &metrics.filter_by.affiliate_code_id,
        )],
        |_| true,
        |row| row.bucket,
        |row| {
            AnalyticsData::AffiliateCode(AffiliateCodeAnalytics {
                source_affiliate_code: row.affiliate_code_id.into(),
                metrics: AffiliateCodeMetrics::Clicks(AffiliateCodeClicks {
                    clicks: row.clicks,
                }),
            })
        },
    )
    .await
}
